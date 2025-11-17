//! # Etch Figma - Convert Figma designs to React/TSX
//!
//! This library converts Figma design files into production-ready React components
//! with Tailwind CSS styling.
//!
//! ## Architecture
//!
//! The library is organized into several key modules:
//!
//! - **`core`** - Core abstractions (walker, visitor trait, config)
//! - **`analysis`** - Pre-conversion analysis of Figma data
//! - **`conversion`** - Transform Figma nodes to intermediate representations
//! - **`generators`** - Generate output files (TSX, HTML, etc.)
//! - **`pipeline`** - High-level orchestration and builder pattern
//! - **`extensions`** - Optional utilities (Tailwind helpers, etc.)
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use etch_figma::pipeline::UnifiedPipelineBuilder;
//! use etch_figma::core::CodeGenConfig;
//!
//! let config = CodeGenConfig::default();
//! let mut pipeline = UnifiedPipelineBuilder::new()
//!     .with_figma_config(config)
//!     .with_file_key("your_figma_file_key")
//!     .build();
//!
//! let result = pipeline.process_canvas(&canvas).await?;
//! result.write_to_filesystem(Path::new("./generated"))?;
//! ```
//!
//! ## Custom Visitor Example
//!
//! ```rust,ignore
//! use etch_figma::core::{Walker, NodeVisitor, NodeContext};
//! use figma_api::models::FrameNode;
//!
//! struct MyVisitor {
//!     frame_count: usize,
//! }
//!
//! impl NodeVisitor for MyVisitor {
//!     fn visit_frame(&mut self, frame: &FrameNode, context: &NodeContext) {
//!         println!("Found frame: {}", frame.name);
//!         self.frame_count += 1;
//!     }
//! }
//!
//! let visitor = MyVisitor { frame_count: 0 };
//! let walker = Walker::new(visitor);
//! let completed = walker.walk_canvas(&canvas);
//! ```
//!
//! See [ARCHITECTURE.md](../ARCHITECTURE.md) for detailed documentation.

// ============================================================================
// Primary Module Structure
// ============================================================================

pub mod analysis;
pub mod conversion;
pub mod core;
pub mod extensions;
pub mod generators;
pub mod pipeline;

// ============================================================================
// Primary Public API - Use these!
// ============================================================================

/// Convenient prelude for common imports
///
/// Use this for quick imports of the most commonly used types:
///
/// ```rust,ignore
/// use etch_figma::prelude::*;
/// ```
///
/// This provides access to:
/// - Configuration: `CodeGenConfig`, `CodeGenConfigBuilder`
/// - Pipeline: `UnifiedPipeline`, `UnifiedPipelineBuilder`
/// - Generators: `TsxGenerator`, `TsxGeneratorBuilder`, `HtmlGenerator`
/// - Custom visitors: `Walker`, `NodeVisitor`, `NodeContext`
/// - Error handling: `Error`, `Result`
/// - Configuration enums: `VectorExportStrategy`, `SvgContainerMode`, etc.
pub mod prelude {
    // Core types
    pub use crate::core::{
        CodeGenConfig, CodeGenConfigBuilder, CodeGenResult, CodeGenSystem, CodeGenerator, Error,
        FileNamingStrategy, NodeContext, NodeVisitor, Result, SvgContainerMode, TextInSvgMode,
        VectorExportStrategy, Walker,
    };

    // Generators
    pub use crate::generators::{HtmlGenerator, TsxGenerator, TsxGeneratorBuilder, TsxVisitor};

    // Pipeline
    pub use crate::pipeline::{UnifiedPipeline, UnifiedPipelineBuilder};

    // Extensions (optional utilities)
    pub use crate::extensions::{TailwindStyleExt, TailwindStyles};
}

// ============================================================================
// Root-level Re-exports for Convenience
// ============================================================================
//
// These re-exports allow you to use common types without the prelude:
//   use etch_figma::{CodeGenConfig, UnifiedPipelineBuilder};
//

// Configuration and core types
pub use core::{
    CodeGenConfig, CodeGenConfigBuilder, CodeGenResult, CodeGenSystem, CodeGenerator, Error,
    FileNamingStrategy, NodeContext, NodeVisitor, Result, SvgContainerMode, TextInSvgMode,
    VectorExportStrategy, Walker,
};

// Generators and visitors
pub use generators::{HtmlGenerator, TsxGenerator, TsxGeneratorBuilder, TsxVisitor};

// Pipeline
pub use pipeline::{UnifiedPipeline, UnifiedPipelineBuilder};

// Extensions
pub use extensions::{TailwindStyleExt, TailwindStyles};

// ============================================================================
// Extension Traits
// ============================================================================

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
