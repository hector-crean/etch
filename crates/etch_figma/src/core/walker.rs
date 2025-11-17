//! Generic walker for traversing Figma node trees using the visitor pattern

use crate::SubcanvasNodeExt;
use figma_api::models::{CanvasNode, SubcanvasNode};

use figma_api::models::{
    BooleanOperationNode, ComponentNode, ComponentSetNode, ConnectorNode, EllipseNode, EmbedNode,
    FrameNode, GroupNode, InstanceNode, LineNode, LinkUnfurlNode, RectangleNode,
    RegularPolygonNode, SectionNode, ShapeWithTextNode, SliceNode, StarNode, StickyNode,
    TableCellNode, TableNode, TextNode, TextPathNode, TransformGroupNode, VectorNode,
    WashiTapeNode, WidgetNode,
};

/// Context information provided during node traversal
#[derive(Debug, Clone)]
pub struct NodeContext {
    /// Hierarchical path from root to current node
    pub path: Vec<String>,
    /// Current traversal depth (0 = root)
    pub depth: usize,
    /// ID of the current node being visited
    pub node_id: String,
}

impl NodeContext {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            depth: 0,
            node_id: String::new(),
        }
    }

    /// Get the full path as a string (e.g., "Canvas > Frame > Group")
    pub fn path_string(&self) -> String {
        self.path.join(" > ")
    }
}

impl Default for NodeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for visiting Figma nodes with rich context information
///
/// All methods have default no-op implementations, so you only need to override
/// the ones you care about.
///
/// # Example
///
/// ```rust,ignore
/// use etch_figma::core::{NodeVisitor, NodeContext, Walker};
///
/// struct MyVisitor {
///     frame_count: usize,
/// }
///
/// impl NodeVisitor for MyVisitor {
///     fn visit_frame(&mut self, frame: &FrameNode, context: &NodeContext) {
///         println!("Found frame at depth {}: {}", context.depth, frame.name);
///         self.frame_count += 1;
///     }
/// }
///
/// let visitor = MyVisitor { frame_count: 0 };
/// let walker = Walker::new(visitor);
/// let completed_visitor = walker.walk_canvas(&canvas);
/// println!("Total frames: {}", completed_visitor.frame_count);
/// ```
pub trait NodeVisitor {
    // Container nodes (have children)
    fn visit_frame(&mut self, _frame: &FrameNode, _context: &NodeContext) {}
    fn visit_group(&mut self, _group: &GroupNode, _context: &NodeContext) {}
    fn visit_component(&mut self, _component: &ComponentNode, _context: &NodeContext) {}
    fn visit_component_set(&mut self, _component_set: &ComponentSetNode, _context: &NodeContext) {}
    fn visit_instance(&mut self, _instance: &InstanceNode, _context: &NodeContext) {}
    fn visit_section(&mut self, _section: &SectionNode, _context: &NodeContext) {}
    fn visit_boolean_operation(
        &mut self,
        _boolean_op: &BooleanOperationNode,
        _context: &NodeContext,
    ) {
    }
    fn visit_table(&mut self, _table: &TableNode, _context: &NodeContext) {}
    fn visit_transform_group(
        &mut self,
        _transform_group: &TransformGroupNode,
        _context: &NodeContext,
    ) {
    }

    // Leaf nodes (no children)
    fn visit_table_cell(&mut self, _table_cell: &TableCellNode, _context: &NodeContext) {}
    fn visit_text(&mut self, _text: &TextNode, _context: &NodeContext) {}
    fn visit_vector(&mut self, _vector: &VectorNode, _context: &NodeContext) {}
    fn visit_rectangle(&mut self, _rectangle: &RectangleNode, _context: &NodeContext) {}
    fn visit_ellipse(&mut self, _ellipse: &EllipseNode, _context: &NodeContext) {}
    fn visit_line(&mut self, _line: &LineNode, _context: &NodeContext) {}
    fn visit_star(&mut self, _star: &StarNode, _context: &NodeContext) {}
    fn visit_regular_polygon(&mut self, _polygon: &RegularPolygonNode, _context: &NodeContext) {}
    fn visit_shape_with_text(
        &mut self,
        _shape_with_text: &ShapeWithTextNode,
        _context: &NodeContext,
    ) {
    }
    fn visit_text_path(&mut self, _text_path: &TextPathNode, _context: &NodeContext) {}
    fn visit_sticky(&mut self, _sticky: &StickyNode, _context: &NodeContext) {}
    fn visit_connector(&mut self, _connector: &ConnectorNode, _context: &NodeContext) {}
    fn visit_washi_tape(&mut self, _washi_tape: &WashiTapeNode, _context: &NodeContext) {}
    fn visit_embed(&mut self, _embed: &EmbedNode, _context: &NodeContext) {}
    fn visit_link_unfurl(&mut self, _link_unfurl: &LinkUnfurlNode, _context: &NodeContext) {}
    fn visit_slice(&mut self, _slice: &SliceNode, _context: &NodeContext) {}
    fn visit_widget(&mut self, _widget: &WidgetNode, _context: &NodeContext) {}

    /// Override to control whether children should be traversed
    /// Default implementation traverses all children
    fn should_traverse_children(&self, _node: &SubcanvasNode) -> bool {
        true
    }

    /// Called before traversing children of a container node
    /// Default implementation does nothing
    fn enter_container(&mut self, _node: &SubcanvasNode, _context: &NodeContext) {}

    /// Called after traversing children of a container node
    /// Default implementation does nothing
    fn exit_container(&mut self, _node: &SubcanvasNode, _context: &NodeContext) {}
}

/// Generic walker that traverses Figma nodes using a visitor pattern
///
/// The walker performs a depth-first, pre-order traversal of the Figma node tree,
/// calling the appropriate visitor methods for each node.
///
/// # Example
///
/// ```rust,ignore
/// use etch_figma::core::Walker;
///
/// let visitor = MyVisitor::new();
/// let walker = Walker::new(visitor);
/// let completed_visitor = walker.walk_canvas(&canvas);
/// // Use completed_visitor to generate output
/// ```
pub struct Walker<V> {
    visitor: V,
}

impl<V: NodeVisitor> Walker<V> {
    /// Create a new walker with the given visitor
    pub fn new(visitor: V) -> Self {
        Self { visitor }
    }

    /// Walk the node tree starting from the given node, returning the visitor
    pub fn walk(mut self, node: &SubcanvasNode) -> V {
        let mut context = NodeContext::new();
        self.walk_recursive(node, &mut context);
        self.visitor
    }

    /// Walk a canvas node, which is the typical entry point
    ///
    /// This is the recommended way to start traversal from a Figma canvas.
    pub fn walk_canvas(mut self, canvas: &CanvasNode) -> V {
        let mut context = NodeContext::new();
        context.path.push(canvas.name.clone());

        for child in &canvas.children {
            self.walk_recursive(child, &mut context);
        }

        self.visitor
    }

    fn walk_recursive(&mut self, node: &SubcanvasNode, context: &mut NodeContext) {
        // Update context with current node info
        if let Some(node_id) = node.id() {
            context.node_id = node_id.to_string();
        }

        // Visit the current node
        match node {
            // Container nodes
            SubcanvasNode::Frame(n) => self.visitor.visit_frame(n, context),
            SubcanvasNode::Group(n) => self.visitor.visit_group(n, context),
            SubcanvasNode::Component(n) => self.visitor.visit_component(n, context),
            SubcanvasNode::ComponentSet(n) => self.visitor.visit_component_set(n, context),
            SubcanvasNode::Instance(n) => self.visitor.visit_instance(n, context),
            SubcanvasNode::Section(n) => self.visitor.visit_section(n, context),
            SubcanvasNode::BooleanOperation(n) => self.visitor.visit_boolean_operation(n, context),
            SubcanvasNode::Table(n) => self.visitor.visit_table(n, context),
            SubcanvasNode::TransformGroup(n) => self.visitor.visit_transform_group(n, context),

            // Leaf nodes
            SubcanvasNode::TableCell(n) => self.visitor.visit_table_cell(n, context),
            SubcanvasNode::Text(n) => self.visitor.visit_text(n, context),
            SubcanvasNode::Vector(n) => self.visitor.visit_vector(n, context),
            SubcanvasNode::Rectangle(n) => self.visitor.visit_rectangle(n, context),
            SubcanvasNode::Ellipse(n) => self.visitor.visit_ellipse(n, context),
            SubcanvasNode::Line(n) => self.visitor.visit_line(n, context),
            SubcanvasNode::Star(n) => self.visitor.visit_star(n, context),
            SubcanvasNode::RegularPolygon(n) => self.visitor.visit_regular_polygon(n, context),
            SubcanvasNode::ShapeWithText(n) => self.visitor.visit_shape_with_text(n, context),
            SubcanvasNode::TextPath(n) => self.visitor.visit_text_path(n, context),
            SubcanvasNode::Sticky(n) => self.visitor.visit_sticky(n, context),
            SubcanvasNode::Connector(n) => self.visitor.visit_connector(n, context),
            SubcanvasNode::WashiTape(n) => self.visitor.visit_washi_tape(n, context),
            SubcanvasNode::Embed(n) => self.visitor.visit_embed(n, context),
            SubcanvasNode::LinkUnfurl(n) => self.visitor.visit_link_unfurl(n, context),
            SubcanvasNode::Slice(n) => self.visitor.visit_slice(n, context),
            SubcanvasNode::Widget(n) => self.visitor.visit_widget(n, context),
        }

        // Traverse children if allowed
        if self.visitor.should_traverse_children(node) {
            if let Some(children) = node.children() {
                // Call enter_container before traversing children
                self.visitor.enter_container(node, context);

                // Update path for children
                if let Some(name) = node.name() {
                    context.path.push(name.to_string());
                }
                context.depth += 1;

                for child in children {
                    self.walk_recursive(child, context);
                }

                // Restore context
                context.depth -= 1;
                if node.name().is_some() {
                    context.path.pop();
                }

                // Call exit_container after traversing children
                self.visitor.exit_container(node, context);
            }
        }
    }
}

