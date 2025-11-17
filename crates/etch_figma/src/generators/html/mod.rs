//! HTML code generation
//!
//! This module generates static HTML from Figma designs.

pub mod generator;
pub mod visitor;

// Re-export for convenience
pub use generator::HtmlGenerator;
pub use visitor::HtmlVisitor;

