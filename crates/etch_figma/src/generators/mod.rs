//! Code generators for different output formats
//!
//! This module contains generators that convert the intermediate representations
//! from visitors into actual output files (TSX, HTML, etc.).

pub mod html;
pub mod tsx;

// Re-export generators and builders
// Note: Internal types like VisitorState, JsxHierarchyBuilder, and SvgHandler
// are kept private to the tsx module as implementation details
pub use html::HtmlGenerator;
pub use tsx::{TsxGenerator, TsxGeneratorBuilder, TsxVisitor};
