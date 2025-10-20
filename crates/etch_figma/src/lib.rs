pub mod analyzers;
pub mod html;
pub mod svg;
pub mod tsx;
pub mod unified_pipeline;

pub mod codegen_ext;
pub mod tailwind_ext;
pub mod walker;

// Re-export the extension traits
pub use codegen_ext::{
    CodeGenConfig, CodeGenResult, CodeGenSystem, CodeGenerator, FileNamingStrategy,
};
pub use tailwind_ext::{TailwindStyleExt, TailwindStyles};

// Re-export generators
pub use html::generator::HtmlGenerator;
pub use tsx::generator::TsxGenerator;

// Re-export unified pipeline
pub use unified_pipeline::{UnifiedPipeline, UnifiedPipelineBuilder};

use figma_api::models::SubcanvasNode;

/// Extension trait for SubcanvasNode to provide common node operations
pub trait SubcanvasNodeExt {
    /// Get the ID of the node, if it has one
    fn id(&self) -> Option<&str>;

    /// Get the name of the node, if it has one
    fn name(&self) -> Option<&str>;

    /// Get the children of the node, if it has any
    fn children(&self) -> Option<&Vec<SubcanvasNode>>;

    /// Check if the node has children
    fn has_children(&self) -> bool {
        self.children()
            .map_or(false, |children| !children.is_empty())
    }

    /// Get the node type as a string for debugging/logging
    fn node_type(&self) -> &'static str;
}

impl SubcanvasNodeExt for SubcanvasNode {
    fn id(&self) -> Option<&str> {
        match self {
            SubcanvasNode::BooleanOperation(n) => Some(&n.id),
            SubcanvasNode::Component(n) => Some(&n.id),
            SubcanvasNode::ComponentSet(n) => Some(&n.id),
            SubcanvasNode::Connector(n) => Some(&n.id),
            SubcanvasNode::Ellipse(n) => Some(&n.id),
            SubcanvasNode::Embed(n) => Some(&n.id),
            SubcanvasNode::Frame(n) => Some(&n.id),
            SubcanvasNode::Group(n) => Some(&n.id),
            SubcanvasNode::Instance(n) => Some(&n.id),
            SubcanvasNode::Line(n) => Some(&n.id),
            SubcanvasNode::LinkUnfurl(n) => Some(&n.id),
            SubcanvasNode::Rectangle(n) => Some(&n.id),
            SubcanvasNode::RegularPolygon(n) => Some(&n.id),
            SubcanvasNode::Section(n) => Some(&n.id),
            SubcanvasNode::ShapeWithText(n) => Some(&n.id),
            SubcanvasNode::Slice(n) => Some(&n.id),
            SubcanvasNode::Star(n) => Some(&n.id),
            SubcanvasNode::Sticky(n) => Some(&n.id),
            SubcanvasNode::Table(n) => Some(&n.id),
            SubcanvasNode::TableCell(n) => Some(&n.id),
            SubcanvasNode::Text(n) => Some(&n.id),
            SubcanvasNode::TextPath(n) => Some(&n.id),
            SubcanvasNode::TransformGroup(n) => Some(&n.id),
            SubcanvasNode::Vector(n) => Some(&n.id),
            SubcanvasNode::WashiTape(n) => Some(&n.id),
            SubcanvasNode::Widget(n) => Some(&n.id),
        }
    }

    fn name(&self) -> Option<&str> {
        match self {
            // Nodes with names
            SubcanvasNode::BooleanOperation(n) => Some(&n.name),
            SubcanvasNode::Component(n) => Some(&n.name),
            SubcanvasNode::ComponentSet(n) => Some(&n.name),
            SubcanvasNode::Frame(n) => Some(&n.name),
            SubcanvasNode::Group(n) => Some(&n.name),
            SubcanvasNode::Instance(n) => Some(&n.name),
            SubcanvasNode::Section(n) => Some(&n.name),
            SubcanvasNode::Table(n) => Some(&n.name),
            SubcanvasNode::TableCell(n) => Some(&n.name),
            SubcanvasNode::TransformGroup(n) => Some(&n.name),
            // Leaf nodes typically don't have meaningful names
            _ => None,
        }
    }

    fn children(&self) -> Option<&Vec<SubcanvasNode>> {
        match self {
            // Container nodes with children
            SubcanvasNode::BooleanOperation(n) => Some(&n.children),
            SubcanvasNode::Component(n) => Some(&n.children),
            SubcanvasNode::ComponentSet(n) => Some(&n.children),
            SubcanvasNode::Frame(n) => Some(&n.children),
            SubcanvasNode::Group(n) => Some(&n.children),
            SubcanvasNode::Instance(n) => Some(&n.children),
            SubcanvasNode::Section(n) => Some(&n.children),
            SubcanvasNode::Table(n) => Some(&n.children),
            SubcanvasNode::TransformGroup(n) => Some(&n.children),
            // Leaf nodes don't have children
            _ => None,
        }
    }

    fn node_type(&self) -> &'static str {
        match self {
            SubcanvasNode::BooleanOperation(_) => "BooleanOperation",
            SubcanvasNode::Component(_) => "Component",
            SubcanvasNode::ComponentSet(_) => "ComponentSet",
            SubcanvasNode::Connector(_) => "Connector",
            SubcanvasNode::Ellipse(_) => "Ellipse",
            SubcanvasNode::Embed(_) => "Embed",
            SubcanvasNode::Frame(_) => "Frame",
            SubcanvasNode::Group(_) => "Group",
            SubcanvasNode::Instance(_) => "Instance",
            SubcanvasNode::Line(_) => "Line",
            SubcanvasNode::LinkUnfurl(_) => "LinkUnfurl",
            SubcanvasNode::Rectangle(_) => "Rectangle",
            SubcanvasNode::RegularPolygon(_) => "RegularPolygon",
            SubcanvasNode::Section(_) => "Section",
            SubcanvasNode::ShapeWithText(_) => "ShapeWithText",
            SubcanvasNode::Slice(_) => "Slice",
            SubcanvasNode::Star(_) => "Star",
            SubcanvasNode::Sticky(_) => "Sticky",
            SubcanvasNode::Table(_) => "Table",
            SubcanvasNode::TableCell(_) => "TableCell",
            SubcanvasNode::Text(_) => "Text",
            SubcanvasNode::TextPath(_) => "TextPath",
            SubcanvasNode::TransformGroup(_) => "TransformGroup",
            SubcanvasNode::Vector(_) => "Vector",
            SubcanvasNode::WashiTape(_) => "WashiTape",
            SubcanvasNode::Widget(_) => "Widget",
        }
    }
}
