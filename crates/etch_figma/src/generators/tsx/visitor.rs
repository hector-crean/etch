use figma_api::models::{
    BooleanOperationNode, ComponentNode, ComponentSetNode, ConnectorNode, EllipseNode, EmbedNode,
    FrameNode, GroupNode, InstanceNode, LineNode, LinkUnfurlNode, RectangleNode,
    RegularPolygonNode, SectionNode, ShapeWithTextNode, SliceNode, StarNode, StickyNode,
    TableCellNode, TableNode, TextNode, TextPathNode, TransformGroupNode, VectorNode, WidgetNode,
};
use std::collections::HashMap;
use swc_ecma_ast::{JSXElement, JSXElementChild, JSXElementName};

use crate::analysis::svg_container::SvgContainerAnalyzer;
use crate::conversion::jsx::ToJsx as _;
use crate::conversion::svg::{
    FigmaSvgExporter, GroupedSvgElement, RenderingStrategy, SvgConfig, SvgGroupingManager,
    VectorExportConfig,
};
use crate::core::config::{CodeGenConfig, SvgContainerMode};
use crate::core::walker::{NodeContext, NodeVisitor};
use crate::extensions::tailwind::TailwindStyles;
use swc_atoms::Atom;

// Import helper modules
use super::state::VisitorState;
use super::style_generator::{PositioningStrategy, StyleGenerator};
use super::svg_handler::SvgHandler;

/// Unified visitor that converts all Figma node types to hierarchical JSX elements
///
/// This visitor uses composition to delegate responsibilities to specialized helpers:
/// - `state`: Tracks all conversion state (elements, relationships, etc.)
/// - `svg_handler`: Manages SVG export and optimization
/// - `style_generator`: Handles positioning and style strategies
/// - `config`: Global configuration options
pub struct TsxVisitor {
    /// State management - tracks elements, relationships, and metadata
    state: VisitorState,

    /// SVG handling - manages vector export and optimization
    svg_handler: SvgHandler,

    /// Style generation - handles positioning and layout strategies
    #[allow(dead_code)]
    style_generator: StyleGenerator,

    /// SVG container mode configuration
    #[allow(dead_code)]
    svg_container_mode: SvgContainerMode,

    /// Global configuration options
    config: Option<CodeGenConfig>,
}

impl TsxVisitor {
    /// Create a new visitor with default configuration
    pub fn new() -> Self {
        Self {
            state: VisitorState::new(),
            svg_handler: SvgHandler::new(),
            style_generator: StyleGenerator::new(),
            svg_container_mode: SvgContainerMode::WrapAll,
            config: None,
        }
    }

    /// Create a new visitor with custom SVG configuration
    pub fn with_svg_config(svg_config: SvgConfig) -> Self {
        Self {
            state: VisitorState::new(),
            svg_handler: SvgHandler::with_svg_config(svg_config),
            style_generator: StyleGenerator::new(),
            svg_container_mode: SvgContainerMode::WrapAll,
            config: None,
        }
    }

    /// Create a new visitor with full code generation configuration
    pub fn with_config(config: CodeGenConfig) -> Self {
        let mut svg_handler = SvgHandler::new();
        svg_handler.vector_export_config = Some(VectorExportConfig::from(&config));

        Self {
            state: VisitorState::new(),
            svg_handler,
            style_generator: StyleGenerator::new(),
            svg_container_mode: config.svg_container_mode.clone(),
            config: Some(config),
        }
    }

    /// Set the Figma file key for SVG exports
    pub fn set_file_key(&mut self, file_key: String) {
        self.svg_handler.set_file_key(file_key);
    }

    /// Get a mutable reference to the SVG exporter
    pub fn get_svg_exporter(&mut self) -> &mut FigmaSvgExporter {
        self.svg_handler.get_svg_exporter_mut()
    }

    /// Set the SVG exporter (used after pre-fetching)
    pub fn set_svg_exporter(&mut self, svg_exporter: FigmaSvgExporter) {
        self.svg_handler.set_svg_exporter(svg_exporter);
    }

    /// Create placeholder JSX element for async SVG loading
    fn create_svg_placeholder_jsx(&self, vector: &VectorNode) -> JSXElement {
        use crate::conversion::jsx::extract_position_from_transform;
        use crate::conversion::styles::create_absolute_position_classes;
        use swc_common::{DUMMY_SP, SyntaxContext};
        use swc_ecma_ast::*;

        let mut attrs = Vec::new();

        // Add data attributes to mark as pending
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

        attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "data-pending".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: "true".into(),
                raw: None,
            }))),
        }));

        // Add basic dimensions if available
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
        }

        // Create placeholder content
        let children = vec![JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: format!("<!-- Loading SVG for vector: {} -->", vector.name).into(),
            raw: Atom::new(""),
        })];

        let svg_element = JSXElement {
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
        };

        // Wrap in positioned div if relative_transform is available
        if let Some((x, y)) = extract_position_from_transform(&vector.relative_transform) {
            let positioning_classes = create_absolute_position_classes(x, y);
            let wrapper_attrs = vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "className".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: positioning_classes.join(" ").into(),
                    raw: None,
                }))),
            })];

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
                    attrs: wrapper_attrs,
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
                children: vec![JSXElementChild::JSXElement(Box::new(svg_element))],
            }
        } else {
            svg_element
        }
    }

    /// Create JSX element from SVG content
    pub fn create_jsx_from_svg_content(&self, svg_content: &str) -> JSXElement {
        use crate::conversion::svg::SvgParser;
        use swc_common::{DUMMY_SP, SyntaxContext};
        use swc_ecma_ast::*;

        // Clean the SVG content first
        let cleaned_svg = SvgParser::clean_svg_content(svg_content);

        // Try to parse the SVG content to proper JSX
        match SvgParser::parse_svg_to_jsx(&cleaned_svg) {
            Ok(jsx_element) => {
                log::debug!("Successfully parsed SVG content to JSX");
                jsx_element
            }
            Err(e) => {
                log::warn!(
                    "Failed to parse SVG content to JSX: {}, falling back to div wrapper",
                    e
                );

                // Fallback: create a div wrapper with the SVG content as text
                // This is safer than dangerouslySetInnerHTML but still not ideal
                let ctxt = SyntaxContext::empty();
                JSXElement {
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(
                            Ident::new("div".into(), DUMMY_SP, ctxt).into(),
                        ),
                        type_args: None,
                        attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(
                                Ident::new("className".into(), DUMMY_SP, ctxt).into(),
                            ),
                            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: "svg-fallback".into(),
                                raw: None,
                            }))),
                        })],
                        self_closing: false,
                    },
                    children: vec![JSXElementChild::JSXText(JSXText {
                        span: DUMMY_SP,
                        value: format!("<!-- SVG parsing failed: {} -->", e).into(),
                        raw: Atom::new(""),
                    })],
                    closing: Some(JSXClosingElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(
                            Ident::new("div".into(), DUMMY_SP, ctxt).into(),
                        ),
                    }),
                }
            }
        }
    }

    /// Add SVG element to the grouping manager for later optimization
    #[allow(dead_code)]
    fn add_svg_to_grouping(&mut self, vector: &VectorNode, svg_content: &str, parent_id: &str) {
        // Extract size from the vector node
        let size = if let Some(size) = &vector.size {
            (size.x, size.y)
        } else {
            (100.0, 100.0) // Default size if not available
        };

        // Extract position from relative_transform if available
        let position = if let Some(transform) = &vector.relative_transform {
            // Extract translation from the transform matrix
            // Transform matrix format: [[a, b, tx], [c, d, ty]]
            if transform.len() >= 2 && transform[0].len() >= 3 && transform[1].len() >= 3 {
                let tx = transform[0][2];
                let ty = transform[1][2];
                (tx, ty)
            } else {
                (0.0, 0.0)
            }
        } else {
            (0.0, 0.0)
        };

        let grouped_element = GroupedSvgElement {
            node_id: vector.id.clone(),
            vector_node: vector.clone(),
            svg_content: svg_content.to_string(),
            position,
            size,
        };

        self.svg_handler
            .svg_grouping_manager
            .add_svg_element(parent_id, grouped_element);
    }

    /// Get all generated JSX elements (flat map for debugging)
    pub fn jsx_elements(&self) -> &HashMap<String, JSXElement> {
        &self.state.jsx_elements
    }

    /// Get mutable access to JSX elements (for async processing)
    pub fn jsx_elements_mut(&mut self) -> &mut HashMap<String, JSXElement> {
        &mut self.state.jsx_elements
    }

    /// Get pending vectors that need async processing
    pub fn pending_vectors(&self) -> &HashMap<String, VectorNode> {
        &self.state.pending_vectors
    }

    /// Get mutable access to pending vectors
    pub fn pending_vectors_mut(&mut self) -> &mut HashMap<String, VectorNode> {
        &mut self.state.pending_vectors
    }

    /// Get mutable access to placeholder JSX elements
    pub fn placeholder_jsx_mut(&mut self) -> &mut HashMap<String, JSXElement> {
        &mut self.state.placeholder_jsx
    }

    /// Get the SVG grouping manager for accessing path registries
    pub fn get_svg_grouping_manager(&self) -> &SvgGroupingManager {
        &self.svg_handler.svg_grouping_manager
    }

    /// Get the current parent ID from the parent stack
    #[allow(dead_code)]
    fn get_current_parent_id(&self) -> String {
        self.state
            .parent_stack
            .last()
            .cloned()
            .unwrap_or_else(|| "root".to_string())
    }

    /// Process SVG groups and replace individual SVG elements with positioned containers
    pub fn process_svg_groups(&mut self) {
        if !self.svg_handler.svg_grouping_manager.has_groups() {
            log::debug!("No SVG groups to process");
            return;
        }

        log::info!(
            "Processing {} SVG groups",
            self.svg_handler.svg_grouping_manager.get_groups().len()
        );

        // Create positioned SVG elements for each group
        let positioned_svgs = self
            .svg_handler
            .svg_grouping_manager
            .create_all_positioned_svgs();

        for (parent_id, positioned_svg_elements) in positioned_svgs {
            log::debug!(
                "Created {} positioned SVG elements for parent: {}",
                positioned_svg_elements.len(),
                parent_id
            );

            // Replace the parent's children with the positioned SVG elements
            if let Some(parent_jsx) = self.state.jsx_elements.get_mut(&parent_id) {
                // Convert JSX elements to JSXElementChild and replace children
                let svg_children: Vec<JSXElementChild> = positioned_svg_elements
                    .into_iter()
                    .map(|jsx| JSXElementChild::JSXElement(Box::new(jsx)))
                    .collect();

                parent_jsx.children = svg_children;
            } else {
                // If parent doesn't exist, create a new container
                let container_jsx = self.create_svg_container(&parent_id, positioned_svg_elements);
                self.state.jsx_elements.insert(parent_id, container_jsx);
            }
        }
    }

    /// Create a container for positioned SVG elements (following Figma export patterns)
    fn create_svg_container(
        &self,
        parent_id: &str,
        positioned_svgs: Vec<JSXElement>,
    ) -> JSXElement {
        use swc_common::{DUMMY_SP, SyntaxContext};
        use swc_ecma_ast::*;

        let ctxt = SyntaxContext::empty();

        // Determine positioning strategy based on parent node
        let positioning_strategy = self.determine_positioning_strategy(parent_id);

        // Create a positioned container div following Figma's patterns
        let mut container_attrs = Vec::new();

        // Add positioning class based on strategy
        let positioning_class = match positioning_strategy {
            PositioningStrategy::Relative => "relative",
            PositioningStrategy::Absolute => "relative", // Parent needs to be relative for absolute children
            PositioningStrategy::Flex => "flex",
            PositioningStrategy::Grid => "grid",
        };

        container_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident::new("className".into(), DUMMY_SP, ctxt).into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: positioning_class.into(),
                raw: None,
            }))),
        }));
        container_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident::new("data-name".into(), DUMMY_SP, ctxt).into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: "svg-container".into(),
                raw: None,
            }))),
        }));
        container_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident::new("data-node-id".into(), DUMMY_SP, ctxt).into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: parent_id.into(),
                raw: None,
            }))),
        }));

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident::new("div".into(), DUMMY_SP, ctxt).into()),
                type_args: None,
                attrs: container_attrs,
                self_closing: false,
            },
            children: positioned_svgs
                .into_iter()
                .map(|jsx| JSXElementChild::JSXElement(Box::new(jsx)))
                .collect(),
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident::new("div".into(), DUMMY_SP, ctxt).into()),
            }),
        }
    }

    /// Determine the positioning strategy for a node based on its parent's layout mode
    fn determine_positioning_strategy(&self, node_id: &str) -> PositioningStrategy {
        // For now, we'll use a simple heuristic
        // In a full implementation, this would check the actual parent node's layout mode

        // Check if this node has a parent with auto-layout
        if let Some(parent_id) = self.get_parent_id(node_id) {
            if let Some(parent_node) = self.get_node_by_id(&parent_id) {
                return match parent_node {
                    figma_api::models::Node::Frame(frame) => {
                        if let Some(layout_mode) = &frame.layout_mode {
                            match layout_mode {
                                figma_api::models::frame_node::LayoutMode::Horizontal
                                | figma_api::models::frame_node::LayoutMode::Vertical => {
                                    PositioningStrategy::Flex
                                }
                                figma_api::models::frame_node::LayoutMode::Grid => {
                                    PositioningStrategy::Grid
                                }
                                figma_api::models::frame_node::LayoutMode::None => {
                                    PositioningStrategy::Absolute
                                }
                            }
                        } else {
                            PositioningStrategy::Absolute
                        }
                    }
                    _ => PositioningStrategy::Absolute,
                };
            }
        }

        // Default to absolute positioning for root nodes or unknown cases
        PositioningStrategy::Absolute
    }

    /// Get the parent ID for a node
    fn get_parent_id(&self, _node_id: &str) -> Option<String> {
        // This would need to be implemented based on the actual parent-child relationships
        // For now, return None to indicate no parent
        None
    }

    /// Get a node by its ID (placeholder implementation)
    fn get_node_by_id(&self, _node_id: &str) -> Option<&figma_api::models::Node> {
        // This would need to be implemented to access the actual node data
        // For now, return None
        None
    }

    /// Get the root JSX elements with full hierarchy built
    pub fn root_jsx_elements(&self) -> HashMap<String, JSXElement> {
        let mut root_elements = HashMap::new();

        for root_id in &self.state.root_nodes {
            if let Some(element) = self.build_hierarchical_jsx(root_id) {
                root_elements.insert(root_id.clone(), element);
            }
        }

        root_elements
    }

    /// Get the generated styles for debugging/inspection
    pub fn node_styles(&self) -> &HashMap<String, TailwindStyles> {
        &self.state.node_styles
    }

    /// Get node types for debugging
    pub fn node_types(&self) -> &HashMap<String, String> {
        &self.state.node_types
    }

    /// Get root node IDs
    pub fn root_nodes(&self) -> &Vec<String> {
        &self.state.root_nodes
    }

    /// Get node names mapping
    pub fn node_names(&self) -> &HashMap<String, String> {
        &self.state.node_names
    }

    /// Get root JSX elements with semantic names as keys
    pub fn root_jsx_elements_by_name(&self) -> HashMap<String, (String, JSXElement)> {
        let mut root_elements = HashMap::new();

        for root_id in &self.state.root_nodes {
            if let Some(element) = self.build_hierarchical_jsx(root_id) {
                let semantic_name = self
                    .state
                    .node_names
                    .get(root_id)
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

        for root_id in &self.state.root_nodes {
            // Only include nodes that are marked as exportable
            if *self.state.exportable_nodes.get(root_id).unwrap_or(&false) {
                if let Some(element) = self.build_hierarchical_jsx(root_id) {
                    let semantic_name = self
                        .state
                        .node_names
                        .get(root_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Component_{}", root_id));
                    root_elements.insert(semantic_name, (root_id.clone(), element));
                }
            }
        }

        root_elements
    }

    /// Get exportable nodes mapping
    /// Get exportable nodes map
    pub fn exportable_nodes(&self) -> &HashMap<String, bool> {
        &self.state.exportable_nodes
    }

    /// Helper method to store JSX element with hierarchy tracking
    fn store_jsx_element(
        &mut self,
        node_id: String,
        node_name: String,
        jsx_element: JSXElement,
        node_type: &str,
        styles: Option<TailwindStyles>,
    ) {
        // Store the base JSX element
        self.state.jsx_elements.insert(node_id.clone(), jsx_element);
        self.state
            .node_types
            .insert(node_id.clone(), node_type.to_string());
        self.state.node_names.insert(node_id.clone(), node_name);

        if let Some(styles) = styles {
            self.state.node_styles.insert(node_id.clone(), styles);
        }

        // Track parent-child relationships
        if let Some(parent_id) = self.state.parent_stack.last() {
            // This node has a parent
            self.state
                .parent_child_map
                .entry(parent_id.clone())
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        } else {
            // This is a root node
            self.state.root_nodes.push(node_id.clone());
        }
    }

    /// Build hierarchical JSX element with all children populated
    fn build_hierarchical_jsx(&self, node_id: &str) -> Option<JSXElement> {
        let mut element = self.state.jsx_elements.get(node_id)?.clone();

        // Get children for this node
        if let Some(child_ids) = self.state.parent_child_map.get(node_id) {
            let mut children = Vec::new();

            for child_id in child_ids {
                if let Some(child_element) = self.build_hierarchical_jsx(child_id) {
                    children.push(JSXElementChild::JSXElement(Box::new(child_element)));
                }
            }

            // Check if this element has a nested div structure for coordinate system separation
            // (outer div for positioning, inner div for layout/content)
            // If the element has exactly one child that's a div JSXElement, place children in that inner div
            if element.children.len() == 1 {
                if let Some(JSXElementChild::JSXElement(inner_element_box)) =
                    element.children.get_mut(0)
                {
                    // Check if the inner element is a div (coordinate system separation pattern)
                    if let JSXElementName::Ident(ident) = &inner_element_box.opening.name {
                        if ident.sym.as_ref() == "div" {
                            // This is a nested div structure - place children in the inner div
                            inner_element_box.children = children;
                            return Some(element);
                        }
                    }
                }
            }

            // Standard case: place children directly in this element
            element.children = children;
        }

        Some(element)
    }

    /// Enter a container node (push to parent stack)
    fn push_parent(&mut self, node_id: &str) {
        self.state.parent_stack.push(node_id.to_string());
    }

    /// Exit a container node (pop from parent stack)
    fn pop_parent(&mut self) {
        self.state.parent_stack.pop();
    }

    /// Check if a node should be analyzed for SVG container decisions
    fn should_analyze_for_svg_container(&self, node: &figma_api::models::SubcanvasNode) -> bool {
        use crate::SubcanvasNodeExt;

        // Only analyze container nodes that could potentially wrap children in SVG
        match node {
            figma_api::models::SubcanvasNode::Group(_)
            | figma_api::models::SubcanvasNode::Frame(_)
            | figma_api::models::SubcanvasNode::Component(_)
            | figma_api::models::SubcanvasNode::Instance(_) => {
                if let Some(children) = node.children() {
                    children.len() > 1 // Only analyze if there are multiple children
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check if a Figma node is SVG-like (should be rendered as SVG)
    fn is_svg_like_node(node: &figma_api::models::SubcanvasNode) -> bool {
        matches!(
            node,
            figma_api::models::SubcanvasNode::Vector(_)
                | figma_api::models::SubcanvasNode::Rectangle(_)
                | figma_api::models::SubcanvasNode::Ellipse(_)
                | figma_api::models::SubcanvasNode::Line(_)
                | figma_api::models::SubcanvasNode::Star(_)
                | figma_api::models::SubcanvasNode::RegularPolygon(_)
                | figma_api::models::SubcanvasNode::BooleanOperation(_)
        )
    }

    /// Analyze children and create grouped SVG containers for consecutive SVG-like nodes
    fn create_svg_groups_from_children(
        &mut self,
        children: &[figma_api::models::SubcanvasNode],
        parent_id: &str,
    ) {
        use crate::SubcanvasNodeExt;

        let mut i = 0;
        while i < children.len() {
            let node = &children[i];

            if Self::is_svg_like_node(node) {
                // Found an SVG-like node - collect consecutive SVG siblings
                let mut svg_group = vec![node];

                i += 1;
                while i < children.len() && Self::is_svg_like_node(&children[i]) {
                    svg_group.push(&children[i]);
                    i += 1;
                }

                // Only create a group if we have multiple SVG nodes
                if svg_group.len() > 1 {
                    // Generate unique ID for the SVG group
                    let group_id =
                        format!("svg_group_{}_{}", parent_id, self.state.svg_group_counter);
                    self.state.svg_group_counter += 1;

                    // Mark all nodes in the group as grouped (to skip individual processing)
                    for svg_node in &svg_group {
                        if let Some(node_id) = svg_node.id() {
                            self.state.grouped_svg_nodes.insert(node_id.to_string());
                        }
                    }

                    // Create a synthetic SVG container JSX element
                    let svg_container = self.create_svg_group_container(&svg_group, &group_id);

                    // Store the SVG container as a JSX element
                    self.state
                        .jsx_elements
                        .insert(group_id.clone(), svg_container);
                    self.state
                        .node_types
                        .insert(group_id.clone(), "SVGGroup".to_string());
                    self.state
                        .node_names
                        .insert(group_id.clone(), format!("SVG Group"));

                    // Add the group as a child of the parent
                    self.state
                        .parent_child_map
                        .entry(parent_id.to_string())
                        .or_insert_with(Vec::new)
                        .push(group_id.clone());

                    // Track the individual SVG nodes as children of the group
                    let mut group_children = Vec::new();
                    for svg_node in svg_group {
                        if let Some(node_id) = svg_node.id() {
                            group_children.push(node_id.to_string());
                        }
                    }
                    self.state.parent_child_map.insert(group_id, group_children);
                }
            } else {
                // Non-SVG node, just skip it (will be processed normally)
                i += 1;
            }
        }
    }

    /// Get the parent position for an SVG group by finding the parent container's bounding box
    fn get_svg_group_parent_position(&self, _node_id: &str) -> (f64, f64) {
        // TODO: Extract actual parent bounding box from parent node
        // For now, return 0,0 - in a real implementation, we'd look up the parent
        // SVG group container and get its position
        (0.0, 0.0)
    }

    /// Create a grouped SVG container JSX element
    fn create_svg_group_container(
        &self,
        _svg_nodes: &[&figma_api::models::SubcanvasNode],
        group_id: &str,
    ) -> JSXElement {
        use swc_common::{DUMMY_SP, SyntaxContext};
        use swc_ecma_ast::*;

        // Calculate bounding box from all SVG nodes
        let min_x = 0.0f64;
        let min_y = 0.0f64;
        let max_x = 100.0f64;
        let max_y = 100.0f64;

        // Try to extract bounds from the nodes
        // For now, use default values - proper bounds calculation would require accessing node properties

        let width = max_x - min_x;
        let height = max_y - min_y;

        // Create SVG element attributes
        let attrs = vec![
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "width".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: format!("{}", width).into(),
                    raw: None,
                }))),
            }),
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "height".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: format!("{}", height).into(),
                    raw: None,
                }))),
            }),
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "viewBox".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: format!("{} {} {} {}", min_x, min_y, width, height).into(),
                    raw: None,
                }))),
            }),
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "xmlns".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: "http://www.w3.org/2000/svg".into(),
                    raw: None,
                }))),
            }),
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "data-svg-group".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: group_id.into(),
                    raw: None,
                }))),
            }),
        ];

        // Create the SVG element (children will be added during build_hierarchical_jsx)
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
            children: vec![],
        }
    }
}

impl NodeVisitor for TsxVisitor {
    // Container nodes (have children)
    fn visit_frame(&mut self, frame: &FrameNode, _context: &NodeContext) {
        use crate::extensions::tailwind::TailwindStyleExt;
        let styles = frame.to_tailwind();
        let jsx_element = frame.to_jsx();

        // Check if this frame has export settings (is marked for export in Figma)
        let is_exportable = frame
            .export_settings
            .as_ref()
            .map(|settings| !settings.is_empty())
            .unwrap_or(false);

        self.state
            .exportable_nodes
            .insert(frame.id.clone(), is_exportable);
        self.store_jsx_element(
            frame.id.clone(),
            frame.name.clone(),
            jsx_element,
            "Frame",
            Some(styles),
        );
    }

    fn visit_group(&mut self, group: &GroupNode, _context: &NodeContext) {
        use crate::extensions::tailwind::TailwindStyleExt;
        let styles = group.to_tailwind();
        let jsx_element = group.to_jsx();
        self.store_jsx_element(
            group.id.clone(),
            group.name.clone(),
            jsx_element,
            "Group",
            Some(styles),
        );
    }

    fn visit_component(&mut self, component: &ComponentNode, _context: &NodeContext) {
        // Components are similar to frames but with component-specific behavior
        let jsx_element = component.to_jsx();
        self.store_jsx_element(
            component.id.clone(),
            component.name.clone(),
            jsx_element,
            "Component",
            None,
        );
    }

    fn visit_component_set(&mut self, component_set: &ComponentSetNode, _context: &NodeContext) {
        // Component sets are containers for component variants
        let jsx_element = component_set.to_jsx();
        self.store_jsx_element(
            component_set.id.clone(),
            component_set.name.clone(),
            jsx_element,
            "ComponentSet",
            None,
        );
    }

    fn visit_instance(&mut self, instance: &InstanceNode, _context: &NodeContext) {
        // Instances reference components
        let jsx_element = instance.to_jsx();
        self.store_jsx_element(
            instance.id.clone(),
            instance.name.clone(),
            jsx_element,
            "Instance",
            None,
        );
    }

    fn visit_section(&mut self, section: &SectionNode, _context: &NodeContext) {
        let jsx_element = section.to_jsx();
        self.store_jsx_element(
            section.id.clone(),
            section.name.clone(),
            jsx_element,
            "Section",
            None,
        );
    }

    fn visit_boolean_operation(
        &mut self,
        boolean_op: &BooleanOperationNode,
        _context: &NodeContext,
    ) {
        let jsx_element = boolean_op.to_jsx();
        self.store_jsx_element(
            boolean_op.id.clone(),
            boolean_op.name.clone(),
            jsx_element,
            "BooleanOperation",
            None,
        );
    }

    fn visit_table(&mut self, table: &TableNode, _context: &NodeContext) {
        let jsx_element = table.to_jsx();
        self.store_jsx_element(
            table.id.clone(),
            table.name.clone(),
            jsx_element,
            "Table",
            None,
        );
    }

    fn visit_transform_group(
        &mut self,
        transform_group: &TransformGroupNode,
        _context: &NodeContext,
    ) {
        let jsx_element = transform_group.to_jsx();
        self.store_jsx_element(
            transform_group.id.clone(),
            transform_group.name.clone(),
            jsx_element,
            "TransformGroup",
            None,
        );
    }

    // Leaf nodes (no children)
    fn visit_table_cell(&mut self, table_cell: &TableCellNode, _context: &NodeContext) {
        let jsx_element = table_cell.to_jsx();
        self.store_jsx_element(
            table_cell.id.clone(),
            table_cell.name.clone(),
            jsx_element,
            "TableCell",
            None,
        );
    }

    fn visit_text(&mut self, text: &TextNode, _context: &NodeContext) {
        use crate::conversion::jsx::converters::text::TextNodeExt;
        use crate::extensions::tailwind::TailwindStyleExt;

        let styles = text.to_tailwind();

        // Use config-aware JSX generation if config is available
        let jsx_element = if let Some(config) = &self.config {
            TextNodeExt::to_jsx_with_config(text, config)
        } else {
            TextNodeExt::to_jsx(text)
        };

        self.store_jsx_element(
            text.id.clone(),
            text.name.clone(),
            jsx_element,
            "Text",
            Some(styles),
        );
    }

    fn visit_vector(&mut self, vector: &VectorNode, _context: &NodeContext) {
        log::debug!("Processing vector node: {} ({})", vector.name, vector.id);

        // Determine rendering context based on whether we're in an SVG group
        let render_context = if self.state.grouped_svg_nodes.contains(&vector.id) {
            // In SVG group - use SVG context with parent positioning
            let (parent_x, parent_y) = self.get_svg_group_parent_position(&vector.id);
            crate::conversion::jsx::RenderContext::Svg { parent_x, parent_y }
        } else {
            crate::conversion::jsx::RenderContext::Html
        };

        log::debug!(
            "Processing vector node: {} ({}) with context: {:?}",
            vector.name,
            vector.id,
            render_context
        );

        // Determine rendering strategy based on configuration
        let default_config = VectorExportConfig::default();
        let config = self
            .svg_handler
            .vector_export_config
            .as_ref()
            .unwrap_or(&default_config);

        let should_use_api = config.should_use_api(vector);
        log::info!(
            "Vector {} should_use_api: {} (strategy: {:?})",
            vector.id,
            should_use_api,
            config.strategy
        );

        // Debug fill_geometry
        if let Some(fill_geometry) = &vector.fill_geometry {
            log::debug!(
                "Vector {} has fill_geometry with {} items",
                vector.id,
                fill_geometry.len()
            );
            if !fill_geometry.is_empty() {
                log::debug!(
                    "Vector {} path data length: {}",
                    vector.id,
                    fill_geometry[0].path.len()
                );
            }
        } else {
            log::debug!("Vector {} has no fill_geometry", vector.id);
        }

        let strategy = if should_use_api {
            RenderingStrategy::SvgExternal // Use API
        } else {
            RenderingStrategy::SvgInline // Manual conversion
        };

        self.state
            .rendering_strategies
            .insert(vector.id.clone(), strategy.clone());

        // Use the appropriate rendering strategy
        let jsx_element = match strategy {
            RenderingStrategy::SvgExternal => {
                log::debug!("Using SvgExternal strategy for vector {}", vector.id);
                // Always store vectors that should use API as pending
                // The AsyncSvgProcessor will handle them intelligently later
                if self.svg_handler.file_key.is_some() {
                    log::debug!(
                        "Storing vector {} as pending for async processing",
                        vector.id
                    );
                    // Store vector as pending and create placeholder
                    self.state
                        .pending_vectors
                        .insert(vector.id.clone(), vector.clone());
                    let placeholder = self.create_svg_placeholder_jsx(vector);
                    self.state
                        .placeholder_jsx
                        .insert(vector.id.clone(), placeholder.clone());
                    placeholder
                } else {
                    log::debug!(
                        "No file key available for vector {}, falling back to inline",
                        vector.id
                    );
                    // No file key available, fall back to inline conversion
                    create_inline_vector_jsx_element(vector)
                }
            }
            RenderingStrategy::SvgInline => {
                log::debug!("Using SvgInline strategy for vector {}", vector.id);
                // Use manual conversion
                create_inline_vector_jsx_element(vector)
            }
            RenderingStrategy::Html | RenderingStrategy::Mixed => {
                log::debug!(
                    "Using Html/Mixed strategy for vector {}, falling back to inline",
                    vector.id
                );
                // Fallback to inline conversion for now
                create_inline_vector_jsx_element(vector)
            }
        };

        self.store_jsx_element(
            vector.id.clone(),
            vector.name.clone(),
            jsx_element,
            "Vector",
            None,
        );
    }

    fn visit_rectangle(&mut self, rectangle: &RectangleNode, _context: &NodeContext) {
        // Determine rendering context based on whether we're in an SVG group
        let render_context = if self.state.grouped_svg_nodes.contains(&rectangle.id) {
            let (parent_x, parent_y) = self.get_svg_group_parent_position(&rectangle.id);
            crate::conversion::jsx::RenderContext::Svg { parent_x, parent_y }
        } else {
            crate::conversion::jsx::RenderContext::Html
        };

        // Basic shapes like rectangles are simple enough to always generate inline
        // No need to use the Figma API for these
        let strategy = RenderingStrategy::SvgInline;

        self.state
            .rendering_strategies
            .insert(rectangle.id.clone(), strategy);

        let jsx_element = rectangle.to_jsx_with_context(render_context);
        self.store_jsx_element(
            rectangle.id.clone(),
            rectangle.name.clone(),
            jsx_element,
            "Rectangle",
            None,
        );
    }

    fn visit_ellipse(&mut self, ellipse: &EllipseNode, _context: &NodeContext) {
        // Determine rendering context based on whether we're in an SVG group
        let render_context = if self.state.grouped_svg_nodes.contains(&ellipse.id) {
            let (parent_x, parent_y) = self.get_svg_group_parent_position(&ellipse.id);
            crate::conversion::jsx::RenderContext::Svg { parent_x, parent_y }
        } else {
            crate::conversion::jsx::RenderContext::Html
        };

        // Basic shapes like ellipses are simple enough to always generate inline
        // No need to use the Figma API for these
        let strategy = RenderingStrategy::SvgInline;

        self.state
            .rendering_strategies
            .insert(ellipse.id.clone(), strategy);

        let jsx_element = ellipse.to_jsx_with_context(render_context);
        self.store_jsx_element(
            ellipse.id.clone(),
            ellipse.name.clone(),
            jsx_element,
            "Ellipse",
            None,
        );
    }

    fn visit_line(&mut self, line: &LineNode, _context: &NodeContext) {
        // Determine rendering context based on whether we're in an SVG group
        let render_context = if self.state.grouped_svg_nodes.contains(&line.id) {
            let (parent_x, parent_y) = self.get_svg_group_parent_position(&line.id);
            crate::conversion::jsx::RenderContext::Svg { parent_x, parent_y }
        } else {
            crate::conversion::jsx::RenderContext::Html
        };

        // Basic shapes like lines are simple enough to always generate inline
        // No need to use the Figma API for these
        let strategy = RenderingStrategy::SvgInline;

        self.state
            .rendering_strategies
            .insert(line.id.clone(), strategy);

        let jsx_element = line.to_jsx_with_context(render_context);
        self.store_jsx_element(
            line.id.clone(),
            line.name.clone(),
            jsx_element,
            "Line",
            None,
        );
    }

    fn visit_star(&mut self, star: &StarNode, _context: &NodeContext) {
        // Determine rendering context based on whether we're in an SVG group
        let render_context = if self.state.grouped_svg_nodes.contains(&star.id) {
            let (parent_x, parent_y) = self.get_svg_group_parent_position(&star.id);
            crate::conversion::jsx::RenderContext::Svg { parent_x, parent_y }
        } else {
            crate::conversion::jsx::RenderContext::Html
        };

        // Basic shapes like stars are simple enough to always generate inline
        // No need to use the Figma API for these
        let strategy = RenderingStrategy::SvgInline;

        self.state
            .rendering_strategies
            .insert(star.id.clone(), strategy);

        let jsx_element = star.to_jsx_with_context(render_context);
        self.store_jsx_element(
            star.id.clone(),
            star.name.clone(),
            jsx_element,
            "Star",
            None,
        );
    }

    fn visit_regular_polygon(&mut self, polygon: &RegularPolygonNode, _context: &NodeContext) {
        // Determine rendering context based on whether we're in an SVG group
        let render_context = if self.state.grouped_svg_nodes.contains(&polygon.id) {
            let (parent_x, parent_y) = self.get_svg_group_parent_position(&polygon.id);
            crate::conversion::jsx::RenderContext::Svg { parent_x, parent_y }
        } else {
            crate::conversion::jsx::RenderContext::Html
        };

        // Basic shapes like regular polygons are simple enough to always generate inline
        // No need to use the Figma API for these
        let strategy = RenderingStrategy::SvgInline;

        self.state
            .rendering_strategies
            .insert(polygon.id.clone(), strategy);

        let jsx_element = polygon.to_jsx_with_context(render_context);
        self.store_jsx_element(
            polygon.id.clone(),
            polygon.name.clone(),
            jsx_element,
            "RegularPolygon",
            None,
        );
    }

    fn visit_shape_with_text(
        &mut self,
        shape_with_text: &ShapeWithTextNode,
        _context: &NodeContext,
    ) {
        let jsx_element = shape_with_text.to_jsx();
        self.store_jsx_element(
            shape_with_text.id.clone(),
            shape_with_text.name.clone(),
            jsx_element,
            "ShapeWithText",
            None,
        );
    }

    fn visit_text_path(&mut self, text_path: &TextPathNode, _context: &NodeContext) {
        let jsx_element = text_path.to_jsx();
        self.store_jsx_element(
            text_path.id.clone(),
            text_path.name.clone(),
            jsx_element,
            "TextPath",
            None,
        );
    }

    fn visit_sticky(&mut self, sticky: &StickyNode, _context: &NodeContext) {
        let jsx_element = sticky.to_jsx();
        self.store_jsx_element(
            sticky.id.clone(),
            sticky.name.clone(),
            jsx_element,
            "Sticky",
            None,
        );
    }

    fn visit_connector(&mut self, connector: &ConnectorNode, _context: &NodeContext) {
        let jsx_element = connector.to_jsx();
        self.store_jsx_element(
            connector.id.clone(),
            connector.name.clone(),
            jsx_element,
            "Connector",
            None,
        );
    }

    fn visit_embed(&mut self, embed: &EmbedNode, _context: &NodeContext) {
        let jsx_element = embed.to_jsx();
        self.store_jsx_element(
            embed.id.clone(),
            embed.name.clone(),
            jsx_element,
            "Embed",
            None,
        );
    }

    fn visit_link_unfurl(&mut self, link_unfurl: &LinkUnfurlNode, _context: &NodeContext) {
        let jsx_element = link_unfurl.to_jsx();
        self.store_jsx_element(
            link_unfurl.id.clone(),
            link_unfurl.name.clone(),
            jsx_element,
            "LinkUnfurl",
            None,
        );
    }

    fn visit_slice(&mut self, slice: &SliceNode, _context: &NodeContext) {
        let jsx_element = slice.to_jsx();
        self.store_jsx_element(
            slice.id.clone(),
            slice.name.clone(),
            jsx_element,
            "Slice",
            None,
        );
    }

    fn visit_widget(&mut self, widget: &WidgetNode, _context: &NodeContext) {
        let jsx_element = widget.to_jsx();
        self.store_jsx_element(
            widget.id.clone(),
            widget.name.clone(),
            jsx_element,
            "Widget",
            None,
        );
    }

    /// Called before traversing children - push node to parent stack
    fn enter_container(&mut self, node: &figma_api::models::SubcanvasNode, _context: &NodeContext) {
        use crate::SubcanvasNodeExt;
        if let Some(node_id) = node.id() {
            self.push_parent(node_id);

            // Analyze children for SVG grouping opportunities
            if let Some(children) = node.children() {
                self.create_svg_groups_from_children(children, node_id);
            }

            // Analyze node for SVG container decisions
            if self.should_analyze_for_svg_container(node) {
                let analysis = SvgContainerAnalyzer::analyze_children(node);
                self.state
                    .container_analyses
                    .insert(node_id.to_string(), analysis);
            }
        }
    }

    /// Called after traversing children - pop node from parent stack
    fn exit_container(&mut self, _node: &figma_api::models::SubcanvasNode, _context: &NodeContext) {
        self.pop_parent();
    }
}

// Note: ToJsx trait implementations are temporarily in crate::tsx::converters module
// They will be migrated to crate::conversion::jsx in a future refactor

/// Create an inline SVG element for vector nodes with path data directly embedded
fn create_inline_vector_jsx_element(vector: &VectorNode) -> JSXElement {
    use crate::conversion::jsx::extract_position_from_transform;
    use crate::conversion::styles::create_absolute_position_classes;
    use swc_common::{DUMMY_SP, SyntaxContext};
    use swc_ecma_ast::{
        Ident, IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement,
        JSXElementChild, JSXElementName, JSXOpeningElement, Lit, Str,
    };

    let mut attrs = Vec::new();

    // Add className for SVG styling (no absolute positioning on SVG itself)
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
            let mut path_attrs = vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
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
            })];

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

    // Create the SVG element
    let svg_element = JSXElement {
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
    };

    // Wrap in positioned div if relative_transform is available
    if let Some((x, y)) = extract_position_from_transform(&vector.relative_transform) {
        let positioning_classes = create_absolute_position_classes(x, y);
        let wrapper_attrs = vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "className".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: positioning_classes.join(" ").into(),
                raw: None,
            }))),
        })];

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
                attrs: wrapper_attrs,
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
            children: vec![JSXElementChild::JSXElement(Box::new(svg_element))],
        }
    } else {
        svg_element
    }
}

/// Extract a simple color from a Paint enum (simplified version)
fn extract_simple_color(_paint: &figma_api::models::Paint) -> Option<String> {
    // For now, we'll just return a placeholder since we need to understand the Paint structure better
    // This can be enhanced once we know the actual Paint enum structure
    Some("#000000".to_string())
}
