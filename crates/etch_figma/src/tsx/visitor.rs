use std::collections::HashMap;
use figma_api::models::{
    FrameNode, VectorNode, TextNode, RectangleNode, EllipseNode, LineNode, StarNode,
    GroupNode, ComponentNode, InstanceNode, SectionNode, RegularPolygonNode,
    BooleanOperationNode, ComponentSetNode, TableNode, TransformGroupNode,
    TableCellNode, ShapeWithTextNode, TextPathNode, StickyNode, ConnectorNode,
    EmbedNode, LinkUnfurlNode, SliceNode, WashiTapeNode, WidgetNode
};
use swc_ecma_ast::{JSXElement, JSXElementChild};

use crate::walker::{NodeVisitor, NodeContext};
use super::ToJsx;
use crate::tailwind_ext::TailwindStyles;
use super::svg_strategy::{RenderingStrategy};
use super::figma_svg_export::FigmaSvgExporter;
use super::svg_strategy::SvgConfig;

/// Unified visitor that converts all Figma node types to hierarchical JSX elements
/// This builds the proper parent-child relationships from Figma
pub struct TsxVisitor {
    /// Generated JSX elements by node ID for all node types
    jsx_elements: HashMap<String, JSXElement>,
    /// Generated styles by node ID (for debugging/inspection)
    node_styles: HashMap<String, TailwindStyles>,
    /// Track node types for debugging
    node_types: HashMap<String, String>,
    /// Track parent-child relationships
    parent_child_map: HashMap<String, Vec<String>>,
    /// Track which nodes are root nodes (no parent) - stores node IDs
    root_nodes: Vec<String>,
    /// Stack to track current parent during traversal
    parent_stack: Vec<String>,
    /// Map node IDs to their semantic names
    node_names: HashMap<String, String>,
    /// Track which nodes are marked for export (have export settings)
    exportable_nodes: HashMap<String, bool>,
    /// SVG exporter for vector content
    svg_exporter: FigmaSvgExporter,
    /// Current file key for SVG exports
    file_key: Option<String>,
    /// Track rendering strategies for nodes
    rendering_strategies: HashMap<String, RenderingStrategy>,
}

impl TsxVisitor {
    pub fn new() -> Self {
        Self {
            jsx_elements: HashMap::new(),
            node_styles: HashMap::new(),
            node_types: HashMap::new(),
            parent_child_map: HashMap::new(),
            root_nodes: Vec::new(),
            parent_stack: Vec::new(),
            node_names: HashMap::new(),
            exportable_nodes: HashMap::new(),
            svg_exporter: FigmaSvgExporter::new(SvgConfig::default()),
            file_key: None,
            rendering_strategies: HashMap::new(),
        }
    }

    pub fn with_svg_config(config: SvgConfig) -> Self {
        Self {
            jsx_elements: HashMap::new(),
            node_styles: HashMap::new(),
            node_types: HashMap::new(),
            parent_child_map: HashMap::new(),
            root_nodes: Vec::new(),
            parent_stack: Vec::new(),
            node_names: HashMap::new(),
            exportable_nodes: HashMap::new(),
            svg_exporter: FigmaSvgExporter::new(config),
            file_key: None,
            rendering_strategies: HashMap::new(),
        }
    }

    pub fn set_file_key(&mut self, file_key: String) {
        self.file_key = Some(file_key);
    }
    
    /// Get all generated JSX elements (flat map for debugging)
    pub fn jsx_elements(&self) -> &HashMap<String, JSXElement> {
        &self.jsx_elements
    }
    
    /// Get the root JSX elements with full hierarchy built
    pub fn root_jsx_elements(&self) -> HashMap<String, JSXElement> {
        let mut root_elements = HashMap::new();
        
        for root_id in &self.root_nodes {
            if let Some(element) = self.build_hierarchical_jsx(root_id) {
                root_elements.insert(root_id.clone(), element);
            }
        }
        
        root_elements
    }
    
    /// Get the generated styles for debugging/inspection
    pub fn node_styles(&self) -> &HashMap<String, TailwindStyles> {
        &self.node_styles
    }
    
    /// Get node types for debugging
    pub fn node_types(&self) -> &HashMap<String, String> {
        &self.node_types
    }
    
    /// Get root node IDs
    pub fn root_nodes(&self) -> &Vec<String> {
        &self.root_nodes
    }
    
    /// Get node names mapping
    pub fn node_names(&self) -> &HashMap<String, String> {
        &self.node_names
    }
    
    /// Get root JSX elements with semantic names as keys
    pub fn root_jsx_elements_by_name(&self) -> HashMap<String, (String, JSXElement)> {
        let mut root_elements = HashMap::new();
        
        for root_id in &self.root_nodes {
            if let Some(element) = self.build_hierarchical_jsx(root_id) {
                let semantic_name = self.node_names.get(root_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Component_{}", root_id));
                root_elements.insert(semantic_name, (root_id.clone(), element));
            }
        }
        
        root_elements
    }
    
    /// Get only exportable root JSX elements (those with export settings)
    pub fn exportable_root_jsx_elements_by_name(&self) -> HashMap<String, (String, JSXElement)> {
        let mut root_elements = HashMap::new();
        
        for root_id in &self.root_nodes {
            // Only include nodes that are marked as exportable
            if *self.exportable_nodes.get(root_id).unwrap_or(&false) {
                if let Some(element) = self.build_hierarchical_jsx(root_id) {
                    let semantic_name = self.node_names.get(root_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Component_{}", root_id));
                    root_elements.insert(semantic_name, (root_id.clone(), element));
                }
            }
        }
        
        root_elements
    }
    
    /// Get exportable nodes mapping
    pub fn exportable_nodes(&self) -> &HashMap<String, bool> {
        &self.exportable_nodes
    }
    
    /// Helper method to store JSX element with hierarchy tracking
    fn store_jsx_element(&mut self, node_id: String, node_name: String, jsx_element: JSXElement, node_type: &str, styles: Option<TailwindStyles>) {
        // Store the base JSX element
        self.jsx_elements.insert(node_id.clone(), jsx_element);
        self.node_types.insert(node_id.clone(), node_type.to_string());
        self.node_names.insert(node_id.clone(), node_name);
        
        if let Some(styles) = styles {
            self.node_styles.insert(node_id.clone(), styles);
        }
        
        // Track parent-child relationships
        if let Some(parent_id) = self.parent_stack.last() {
            // This node has a parent
            self.parent_child_map
                .entry(parent_id.clone())
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        } else {
            // This is a root node
            self.root_nodes.push(node_id.clone());
        }
    }
    
    /// Build hierarchical JSX element with all children populated
    fn build_hierarchical_jsx(&self, node_id: &str) -> Option<JSXElement> {
        let mut element = self.jsx_elements.get(node_id)?.clone();
        
        // Get children for this node
        if let Some(child_ids) = self.parent_child_map.get(node_id) {
            let mut children = Vec::new();
            
            for child_id in child_ids {
                if let Some(child_element) = self.build_hierarchical_jsx(child_id) {
                    children.push(JSXElementChild::JSXElement(Box::new(child_element)));
                }
            }
            
            element.children = children;
        }
        
        Some(element)
    }
    
    /// Enter a container node (push to parent stack)
    fn push_parent(&mut self, node_id: &str) {
        self.parent_stack.push(node_id.to_string());
    }
    
    /// Exit a container node (pop from parent stack)
    fn pop_parent(&mut self) {
        self.parent_stack.pop();
    }
}

impl NodeVisitor for TsxVisitor {
    // Container nodes (have children)
    fn visit_frame(&mut self, frame: &FrameNode, _context: &NodeContext) {
        use crate::tailwind_ext::TailwindStyleExt;
        let styles = frame.to_tailwind();
        let jsx_element = frame.to_jsx();
        
        // Check if this frame has export settings (is marked for export in Figma)
        let is_exportable = frame.export_settings.as_ref()
            .map(|settings| !settings.is_empty())
            .unwrap_or(false);
        
        self.exportable_nodes.insert(frame.id.clone(), is_exportable);
        self.store_jsx_element(frame.id.clone(), frame.name.clone(), jsx_element, "Frame", Some(styles));
    }
    
    fn visit_group(&mut self, group: &GroupNode, _context: &NodeContext) {
        use crate::tailwind_ext::TailwindStyleExt;
        let styles = group.to_tailwind();
        let jsx_element = group.to_jsx();
        self.store_jsx_element(group.id.clone(), group.name.clone(), jsx_element, "Group", Some(styles));
    }
    
    fn visit_component(&mut self, component: &ComponentNode, _context: &NodeContext) {
        // Components are similar to frames but with component-specific behavior
        let jsx_element = component.to_jsx();
        self.store_jsx_element(component.id.clone(), component.name.clone(), jsx_element, "Component", None);
    }
    
    fn visit_component_set(&mut self, component_set: &ComponentSetNode, _context: &NodeContext) {
        // Component sets are containers for component variants
        let jsx_element = component_set.to_jsx();
        self.store_jsx_element(component_set.id.clone(), component_set.name.clone(), jsx_element, "ComponentSet", None);
    }
    
    fn visit_instance(&mut self, instance: &InstanceNode, _context: &NodeContext) {
        // Instances reference components
        let jsx_element = instance.to_jsx();
        self.store_jsx_element(instance.id.clone(), instance.name.clone(), jsx_element, "Instance", None);
    }
    
    fn visit_section(&mut self, section: &SectionNode, _context: &NodeContext) {
        let jsx_element = section.to_jsx();
        self.store_jsx_element(section.id.clone(), section.name.clone(), jsx_element, "Section", None);
    }
    
    fn visit_boolean_operation(&mut self, boolean_op: &BooleanOperationNode, _context: &NodeContext) {
        let jsx_element = boolean_op.to_jsx();
        self.store_jsx_element(boolean_op.id.clone(), boolean_op.name.clone(), jsx_element, "BooleanOperation", None);
    }
    
    fn visit_table(&mut self, table: &TableNode, _context: &NodeContext) {
        let jsx_element = table.to_jsx();
        self.store_jsx_element(table.id.clone(), table.name.clone(), jsx_element, "Table", None);
    }
    
    fn visit_transform_group(&mut self, transform_group: &TransformGroupNode, _context: &NodeContext) {
        let jsx_element = transform_group.to_jsx();
        self.store_jsx_element(transform_group.id.clone(), transform_group.name.clone(), jsx_element, "TransformGroup", None);
    }
    
    // Leaf nodes (no children)
    fn visit_table_cell(&mut self, table_cell: &TableCellNode, _context: &NodeContext) {
        let jsx_element = table_cell.to_jsx();
        self.store_jsx_element(table_cell.id.clone(), table_cell.name.clone(), jsx_element, "TableCell", None);
    }
    
    fn visit_text(&mut self, text: &TextNode, _context: &NodeContext) {
        use crate::tailwind_ext::TailwindStyleExt;
        let styles = text.to_tailwind();
        let jsx_element = text.to_jsx();
        self.store_jsx_element(text.id.clone(), text.name.clone(), jsx_element, "Text", Some(styles));
    }
    
    fn visit_vector(&mut self, vector: &VectorNode, _context: &NodeContext) {
        // Determine rendering strategy
        let strategy = RenderingStrategy::SvgInline; // Default for vector nodes
        self.rendering_strategies.insert(vector.id.clone(), strategy);
        
        // For now, use the existing implementation
        // In a full implementation, this would use the SVG exporter
        let jsx_element = create_inline_vector_jsx_element(vector);
        self.store_jsx_element(vector.id.clone(), vector.name.clone(), jsx_element, "Vector", None);
    }
    
    fn visit_rectangle(&mut self, rectangle: &RectangleNode, _context: &NodeContext) {
        let jsx_element = rectangle.to_jsx();
        self.store_jsx_element(rectangle.id.clone(), rectangle.name.clone(), jsx_element, "Rectangle", None);
    }
    
    fn visit_ellipse(&mut self, ellipse: &EllipseNode, _context: &NodeContext) {
        let jsx_element = ellipse.to_jsx();
        self.store_jsx_element(ellipse.id.clone(), ellipse.name.clone(), jsx_element, "Ellipse", None);
    }
    
    fn visit_line(&mut self, line: &LineNode, _context: &NodeContext) {
        let jsx_element = line.to_jsx();
        self.store_jsx_element(line.id.clone(), line.name.clone(), jsx_element, "Line", None);
    }
    
    fn visit_star(&mut self, star: &StarNode, _context: &NodeContext) {
        let jsx_element = star.to_jsx();
        self.store_jsx_element(star.id.clone(), star.name.clone(), jsx_element, "Star", None);
    }
    
    fn visit_regular_polygon(&mut self, polygon: &RegularPolygonNode, _context: &NodeContext) {
        let jsx_element = polygon.to_jsx();
        self.store_jsx_element(polygon.id.clone(), polygon.name.clone(), jsx_element, "RegularPolygon", None);
    }
    
    fn visit_shape_with_text(&mut self, shape_with_text: &ShapeWithTextNode, _context: &NodeContext) {
        let jsx_element = shape_with_text.to_jsx();
        self.store_jsx_element(shape_with_text.id.clone(), shape_with_text.name.clone(), jsx_element, "ShapeWithText", None);
    }
    
    fn visit_text_path(&mut self, text_path: &TextPathNode, _context: &NodeContext) {
        let jsx_element = text_path.to_jsx();
        self.store_jsx_element(text_path.id.clone(), text_path.name.clone(), jsx_element, "TextPath", None);
    }
    
    fn visit_sticky(&mut self, sticky: &StickyNode, _context: &NodeContext) {
        let jsx_element = sticky.to_jsx();
        self.store_jsx_element(sticky.id.clone(), sticky.name.clone(), jsx_element, "Sticky", None);
    }
    
    fn visit_connector(&mut self, connector: &ConnectorNode, _context: &NodeContext) {
        let jsx_element = connector.to_jsx();
        self.store_jsx_element(connector.id.clone(), connector.name.clone(), jsx_element, "Connector", None);
    }
    
    fn visit_embed(&mut self, embed: &EmbedNode, _context: &NodeContext) {
        let jsx_element = embed.to_jsx();
        self.store_jsx_element(embed.id.clone(), embed.name.clone(), jsx_element, "Embed", None);
    }
    
    fn visit_link_unfurl(&mut self, link_unfurl: &LinkUnfurlNode, _context: &NodeContext) {
        let jsx_element = link_unfurl.to_jsx();
        self.store_jsx_element(link_unfurl.id.clone(), link_unfurl.name.clone(), jsx_element, "LinkUnfurl", None);
    }
    
    fn visit_slice(&mut self, slice: &SliceNode, _context: &NodeContext) {
        let jsx_element = slice.to_jsx();
        self.store_jsx_element(slice.id.clone(), slice.name.clone(), jsx_element, "Slice", None);
    }
    
    fn visit_widget(&mut self, widget: &WidgetNode, _context: &NodeContext) {
        let jsx_element = widget.to_jsx();
        self.store_jsx_element(widget.id.clone(), widget.name.clone(), jsx_element, "Widget", None);
    }
    
    /// Called before traversing children - push node to parent stack
    fn enter_container(&mut self, node: &figma_api::models::SubcanvasNode, _context: &NodeContext) {
        use crate::SubcanvasNodeExt;
        if let Some(node_id) = node.id() {
            self.push_parent(node_id);
        }
    }
    
    /// Called after traversing children - pop node from parent stack
    fn exit_container(&mut self, _node: &figma_api::models::SubcanvasNode, _context: &NodeContext) {
        self.pop_parent();
    }
}

// We need to implement ToJsx for all the node types that don't have it yet
// Let's add default implementations for the missing ones

// Default JSX implementations for nodes we haven't implemented yet
macro_rules! impl_default_jsx {
    ($node_type:ty, $tag:expr) => {
        impl ToJsx for $node_type {
            fn to_jsx(&self) -> JSXElement {
                use swc_common::{DUMMY_SP, SyntaxContext};
                use swc_ecma_ast::{
                    JSXOpeningElement, JSXClosingElement, JSXElementName, JSXAttr, JSXAttrName,
                    JSXAttrValue, JSXAttrOrSpread, IdentName, Ident, Str, Lit
                };

                let mut attrs = Vec::new();
                
                attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(IdentName {
                        span: DUMMY_SP,
                        sym: "data-name".into(),
                    }),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: self.name.clone().into(),
                        raw: None,
                    }))),
                }));
                
                attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(IdentName {
                        span: DUMMY_SP,
                        sym: "data-type".into(),
                    }),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: $tag.into(),
                        raw: None,
                    }))),
                }));
                
                JSXElement {
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "div".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                        attrs,
                        self_closing: false,
                        type_args: None,
                    },
                    closing: Some(JSXClosingElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "div".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                    }),
                    children: vec![],
                }
            }
        }
    };
}

// Implement default JSX for all missing node types
impl_default_jsx!(ComponentNode, "component");
impl_default_jsx!(ComponentSetNode, "component-set");
impl_default_jsx!(InstanceNode, "instance");
impl_default_jsx!(SectionNode, "section");
impl_default_jsx!(BooleanOperationNode, "boolean-operation");
impl_default_jsx!(TableNode, "table");
impl_default_jsx!(TransformGroupNode, "transform-group");
impl_default_jsx!(TableCellNode, "table-cell");
impl_default_jsx!(EllipseNode, "ellipse");
impl_default_jsx!(LineNode, "line");
impl_default_jsx!(StarNode, "star");
impl_default_jsx!(RegularPolygonNode, "regular-polygon");
impl_default_jsx!(ShapeWithTextNode, "shape-with-text");
impl_default_jsx!(TextPathNode, "text-path");
impl_default_jsx!(StickyNode, "sticky");
impl_default_jsx!(ConnectorNode, "connector");
impl_default_jsx!(EmbedNode, "embed");
impl_default_jsx!(LinkUnfurlNode, "link-unfurl");
impl_default_jsx!(SliceNode, "slice");
impl_default_jsx!(WashiTapeNode, "washi-tape");
impl_default_jsx!(WidgetNode, "widget");

/// Create an inline SVG element for vector nodes with path data directly embedded
fn create_inline_vector_jsx_element(vector: &VectorNode) -> JSXElement {
    use swc_common::{DUMMY_SP, SyntaxContext};
    use swc_ecma_ast::{
        JSXOpeningElement, JSXClosingElement, JSXElementName, JSXAttr, JSXAttrName,
        JSXAttrValue, JSXAttrOrSpread, IdentName, Ident, Str, Lit, JSXElementChild
    };

    let mut attrs = Vec::new();
    
    // Add className for SVG styling
    attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(IdentName {
            span: DUMMY_SP,
            sym: "className".into(),
        }),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: "inline-block".into(),
            raw: None,
        }))),
    }));
    
    // Add data attributes
    attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(IdentName {
            span: DUMMY_SP,
            sym: "data-name".into(),
        }),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: vector.name.clone().into(),
            raw: None,
        }))),
    }));
    
    attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(IdentName {
            span: DUMMY_SP,
            sym: "data-vector".into(),
        }),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: "true".into(),
            raw: None,
        }))),
    }));
    
    // Add size attributes if available
    if let Some(size) = &vector.size {
        attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "width".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: size.x.to_string().into(),
                raw: None,
            }))),
        }));
        
        attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "height".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: size.y.to_string().into(),
                raw: None,
            }))),
        }));
        
        attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "viewBox".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: format!("0 0 {} {}", size.x, size.y).into(),
                raw: None,
            }))),
        }));
    }
    
    // Extract path data from fill geometry and create path element
    let mut children = Vec::new();
    
    if let Some(fill_geometry) = &vector.fill_geometry {
        for path in fill_geometry {
            let mut path_attrs = vec![
                JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(IdentName {
                        span: DUMMY_SP,
                        sym: "d".into(),
                    }),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: path.path.clone().into(),
                        raw: None,
                    }))),
                }),
            ];
            
            // Add fill color if available
            if !vector.fills.is_empty() {
                if let Some(fill_color) = extract_simple_color(&vector.fills[0]) {
                    path_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(IdentName {
                            span: DUMMY_SP,
                            sym: "fill".into(),
                        }),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: fill_color.into(),
                            raw: None,
                        }))),
                    }));
                } else {
                    // Default fill
                    path_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(IdentName {
                            span: DUMMY_SP,
                            sym: "fill".into(),
                        }),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: "currentColor".into(),
                            raw: None,
                        }))),
                    }));
                }
            }
            
            let path_element = JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "path".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: path_attrs,
                    self_closing: true,
                    type_args: None,
                },
                closing: None,
                children: vec![],
            };
            
            children.push(JSXElementChild::JSXElement(Box::new(path_element)));
        }
    }
    
    // If no fill geometry, create a placeholder path
    if children.is_empty() {
        let placeholder_path = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "path".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![
                    JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(IdentName {
                            span: DUMMY_SP,
                            sym: "d".into(),
                        }),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: "M0,0".into(), // Minimal placeholder path
                            raw: None,
                        }))),
                    }),
                    JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(IdentName {
                            span: DUMMY_SP,
                            sym: "fill".into(),
                        }),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: "currentColor".into(),
                            raw: None,
                        }))),
                    }),
                ],
                self_closing: true,
                type_args: None,
            },
            closing: None,
            children: vec![],
        };
        children.push(JSXElementChild::JSXElement(Box::new(placeholder_path)));
    }
    
    // Create the SVG container
    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "svg".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs,
            self_closing: false,
            type_args: None,
        },
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "svg".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
        children,
    }
}

/// Extract a simple color from a Paint enum (simplified version)
fn extract_simple_color(_paint: &figma_api::models::Paint) -> Option<String> {
    // For now, we'll just return a placeholder since we need to understand the Paint structure better
    // This can be enhanced once we know the actual Paint enum structure
    Some("#000000".to_string())
}
