use std::collections::HashMap;
use figma_api::models::{
    FrameNode, VectorNode, TextNode, RectangleNode, EllipseNode, LineNode, StarNode,
    GroupNode, ComponentNode, InstanceNode, SectionNode, RegularPolygonNode,
    BooleanOperationNode, ComponentSetNode, TableNode, TransformGroupNode,
    TableCellNode, ShapeWithTextNode, TextPathNode, StickyNode, ConnectorNode,
    EmbedNode, LinkUnfurlNode, SliceNode, WashiTapeNode, WidgetNode
};

use crate::core::walker::{NodeVisitor, NodeContext};
use crate::extensions::tailwind::TailwindStyles;

/// HTML visitor that converts all Figma node types to HTML elements
/// This builds the proper parent-child relationships from Figma
#[derive(Default)]
pub struct HtmlVisitor {
    /// Generated HTML elements by node ID for all node types
    html_elements: HashMap<String, String>,
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
}

impl HtmlVisitor {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get all generated HTML elements (flat map for debugging)
    pub fn html_elements(&self) -> &HashMap<String, String> {
        &self.html_elements
    }
    
    /// Get the root HTML elements with full hierarchy built
    pub fn root_html_elements(&self) -> HashMap<String, String> {
        let mut root_elements = HashMap::new();
        
        for root_id in &self.root_nodes {
            if let Some(element) = self.build_hierarchical_html(root_id) {
                root_elements.insert(root_id.clone(), element);
            }
        }
        
        root_elements
    }
    
    /// Get root HTML elements with semantic names as keys
    pub fn root_html_elements_by_name(&self) -> HashMap<String, (String, String)> {
        let mut root_elements = HashMap::new();
        
        for root_id in &self.root_nodes {
            if let Some(element) = self.build_hierarchical_html(root_id) {
                let semantic_name = self.node_names.get(root_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Component_{}", root_id));
                root_elements.insert(semantic_name, (root_id.clone(), element));
            }
        }
        
        root_elements
    }
    
    /// Get only exportable root HTML elements (those with export settings)
    pub fn exportable_root_html_elements_by_name(&self) -> HashMap<String, (String, String)> {
        let mut root_elements = HashMap::new();
        
        for root_id in &self.root_nodes {
            // Only include nodes that are marked as exportable
            if *self.exportable_nodes.get(root_id).unwrap_or(&false) {
                if let Some(element) = self.build_hierarchical_html(root_id) {
                    let semantic_name = self.node_names.get(root_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Component_{}", root_id));
                    root_elements.insert(semantic_name, (root_id.clone(), element));
                }
            }
        }
        
        root_elements
    }
    
    /// Helper method to store HTML element with hierarchy tracking
    fn store_html_element(&mut self, node_id: String, node_name: String, html_element: String, node_type: &str, styles: Option<TailwindStyles>) {
        // Store the base HTML element
        self.html_elements.insert(node_id.clone(), html_element);
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
    
    /// Build hierarchical HTML element with all children populated
    fn build_hierarchical_html(&self, node_id: &str) -> Option<String> {
        let mut element = self.html_elements.get(node_id)?.clone();
        
        // Get children for this node
        if let Some(child_ids) = self.parent_child_map.get(node_id) {
            let mut children_html = String::new();
            
            for child_id in child_ids {
                if let Some(child_element) = self.build_hierarchical_html(child_id) {
                    children_html.push_str(&child_element);
                }
            }
            
            // Insert children before closing tag
            if let Some(closing_tag_pos) = element.rfind("</") {
                element.insert_str(closing_tag_pos, &children_html);
            }
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

impl NodeVisitor for HtmlVisitor {
    // Container nodes (have children)
    fn visit_frame(&mut self, frame: &FrameNode, _context: &NodeContext) {
        use crate::extensions::tailwind::TailwindStyleExt;
        let styles = frame.to_tailwind();
        let html_element = frame.to_html();
        
        // Check if this frame has export settings (is marked for export in Figma)
        let is_exportable = frame.export_settings.as_ref()
            .map(|settings| !settings.is_empty())
            .unwrap_or(false);
        
        self.exportable_nodes.insert(frame.id.clone(), is_exportable);
        self.store_html_element(frame.id.clone(), frame.name.clone(), html_element, "Frame", Some(styles));
    }
    
    fn visit_group(&mut self, group: &GroupNode, _context: &NodeContext) {
        use crate::extensions::tailwind::TailwindStyleExt;
        let styles = group.to_tailwind();
        let html_element = group.to_html();
        self.store_html_element(group.id.clone(), group.name.clone(), html_element, "Group", Some(styles));
    }
    
    fn visit_component(&mut self, component: &ComponentNode, _context: &NodeContext) {
        let html_element = component.to_html();
        self.store_html_element(component.id.clone(), component.name.clone(), html_element, "Component", None);
    }
    
    fn visit_component_set(&mut self, component_set: &ComponentSetNode, _context: &NodeContext) {
        let html_element = component_set.to_html();
        self.store_html_element(component_set.id.clone(), component_set.name.clone(), html_element, "ComponentSet", None);
    }
    
    fn visit_instance(&mut self, instance: &InstanceNode, _context: &NodeContext) {
        let html_element = instance.to_html();
        self.store_html_element(instance.id.clone(), instance.name.clone(), html_element, "Instance", None);
    }
    
    fn visit_section(&mut self, section: &SectionNode, _context: &NodeContext) {
        let html_element = section.to_html();
        self.store_html_element(section.id.clone(), section.name.clone(), html_element, "Section", None);
    }
    
    fn visit_boolean_operation(&mut self, boolean_op: &BooleanOperationNode, _context: &NodeContext) {
        let html_element = boolean_op.to_html();
        self.store_html_element(boolean_op.id.clone(), boolean_op.name.clone(), html_element, "BooleanOperation", None);
    }
    
    fn visit_table(&mut self, table: &TableNode, _context: &NodeContext) {
        let html_element = table.to_html();
        self.store_html_element(table.id.clone(), table.name.clone(), html_element, "Table", None);
    }
    
    fn visit_transform_group(&mut self, transform_group: &TransformGroupNode, _context: &NodeContext) {
        let html_element = transform_group.to_html();
        self.store_html_element(transform_group.id.clone(), transform_group.name.clone(), html_element, "TransformGroup", None);
    }
    
    // Leaf nodes (no children)
    fn visit_table_cell(&mut self, table_cell: &TableCellNode, _context: &NodeContext) {
        let html_element = table_cell.to_html();
        self.store_html_element(table_cell.id.clone(), table_cell.name.clone(), html_element, "TableCell", None);
    }
    
    fn visit_text(&mut self, text: &TextNode, _context: &NodeContext) {
        use crate::extensions::tailwind::TailwindStyleExt;
        let styles = text.to_tailwind();
        let html_element = text.to_html();
        self.store_html_element(text.id.clone(), text.name.clone(), html_element, "Text", Some(styles));
    }
    
    fn visit_vector(&mut self, vector: &VectorNode, _context: &NodeContext) {
        let html_element = vector.to_html();
        self.store_html_element(vector.id.clone(), vector.name.clone(), html_element, "Vector", None);
    }
    
    fn visit_rectangle(&mut self, rectangle: &RectangleNode, _context: &NodeContext) {
        let html_element = rectangle.to_html();
        self.store_html_element(rectangle.id.clone(), rectangle.name.clone(), html_element, "Rectangle", None);
    }
    
    fn visit_ellipse(&mut self, ellipse: &EllipseNode, _context: &NodeContext) {
        let html_element = ellipse.to_html();
        self.store_html_element(ellipse.id.clone(), ellipse.name.clone(), html_element, "Ellipse", None);
    }
    
    fn visit_line(&mut self, line: &LineNode, _context: &NodeContext) {
        let html_element = line.to_html();
        self.store_html_element(line.id.clone(), line.name.clone(), html_element, "Line", None);
    }
    
    fn visit_star(&mut self, star: &StarNode, _context: &NodeContext) {
        let html_element = star.to_html();
        self.store_html_element(star.id.clone(), star.name.clone(), html_element, "Star", None);
    }
    
    fn visit_regular_polygon(&mut self, polygon: &RegularPolygonNode, _context: &NodeContext) {
        let html_element = polygon.to_html();
        self.store_html_element(polygon.id.clone(), polygon.name.clone(), html_element, "RegularPolygon", None);
    }
    
    fn visit_shape_with_text(&mut self, shape_with_text: &ShapeWithTextNode, _context: &NodeContext) {
        let html_element = shape_with_text.to_html();
        self.store_html_element(shape_with_text.id.clone(), shape_with_text.name.clone(), html_element, "ShapeWithText", None);
    }
    
    fn visit_text_path(&mut self, text_path: &TextPathNode, _context: &NodeContext) {
        let html_element = text_path.to_html();
        self.store_html_element(text_path.id.clone(), text_path.name.clone(), html_element, "TextPath", None);
    }
    
    fn visit_sticky(&mut self, sticky: &StickyNode, _context: &NodeContext) {
        let html_element = sticky.to_html();
        self.store_html_element(sticky.id.clone(), sticky.name.clone(), html_element, "Sticky", None);
    }
    
    fn visit_connector(&mut self, connector: &ConnectorNode, _context: &NodeContext) {
        let html_element = connector.to_html();
        self.store_html_element(connector.id.clone(), connector.name.clone(), html_element, "Connector", None);
    }
    
    fn visit_embed(&mut self, embed: &EmbedNode, _context: &NodeContext) {
        let html_element = embed.to_html();
        self.store_html_element(embed.id.clone(), embed.name.clone(), html_element, "Embed", None);
    }
    
    fn visit_link_unfurl(&mut self, link_unfurl: &LinkUnfurlNode, _context: &NodeContext) {
        let html_element = link_unfurl.to_html();
        self.store_html_element(link_unfurl.id.clone(), link_unfurl.name.clone(), html_element, "LinkUnfurl", None);
    }
    
    fn visit_slice(&mut self, slice: &SliceNode, _context: &NodeContext) {
        let html_element = slice.to_html();
        self.store_html_element(slice.id.clone(), slice.name.clone(), html_element, "Slice", None);
    }
    
    fn visit_widget(&mut self, widget: &WidgetNode, _context: &NodeContext) {
        let html_element = widget.to_html();
        self.store_html_element(widget.id.clone(), widget.name.clone(), html_element, "Widget", None);
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

// We need to implement ToHtml for all the node types
// Let's add default implementations for the missing ones

/// Trait for converting Figma nodes to HTML
pub trait ToHtml {
    fn to_html(&self) -> String;
}

// Default HTML implementations for nodes we haven't implemented yet
macro_rules! impl_default_html {
    ($node_type:ty, $tag:expr) => {
        impl ToHtml for $node_type {
            fn to_html(&self) -> String {
                format!(
                    "<{} data-name=\"{}\" data-type=\"{}\"></{}>",
                    $tag, self.name, $tag, $tag
                )
            }
        }
    };
}

// Implement default HTML for all missing node types
impl_default_html!(ComponentNode, "div");
impl_default_html!(ComponentSetNode, "div");
impl_default_html!(InstanceNode, "div");
impl_default_html!(SectionNode, "section");
impl_default_html!(BooleanOperationNode, "div");
impl_default_html!(TableNode, "table");
impl_default_html!(TransformGroupNode, "div");
impl_default_html!(TableCellNode, "td");
impl_default_html!(EllipseNode, "div");
impl_default_html!(LineNode, "div");
impl_default_html!(StarNode, "div");
impl_default_html!(RegularPolygonNode, "div");
impl_default_html!(ShapeWithTextNode, "div");
impl_default_html!(TextPathNode, "div");
impl_default_html!(StickyNode, "div");
impl_default_html!(ConnectorNode, "div");
impl_default_html!(EmbedNode, "div");
impl_default_html!(LinkUnfurlNode, "div");
impl_default_html!(SliceNode, "div");
impl_default_html!(WashiTapeNode, "div");
impl_default_html!(WidgetNode, "div");

// Implement ToHtml for the main node types
impl ToHtml for FrameNode {
    fn to_html(&self) -> String {
        format!(
            "<div data-name=\"{}\" data-type=\"frame\"></div>",
            self.name
        )
    }
}

impl ToHtml for GroupNode {
    fn to_html(&self) -> String {
        format!(
            "<div data-name=\"{}\" data-type=\"group\"></div>",
            self.name
        )
    }
}

impl ToHtml for TextNode {
    fn to_html(&self) -> String {
        format!(
            "<p data-name=\"{}\" data-type=\"text\">{}</p>",
            self.name, self.characters
        )
    }
}

impl ToHtml for VectorNode {
    fn to_html(&self) -> String {
        format!(
            "<svg data-name=\"{}\" data-type=\"vector\"></svg>",
            self.name
        )
    }
}

impl ToHtml for RectangleNode {
    fn to_html(&self) -> String {
        format!(
            "<div data-name=\"{}\" data-type=\"rectangle\"></div>",
            self.name
        )
    }
}
