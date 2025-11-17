//! Core abstractions for Figma tree traversal and code generation
//!
//! This module contains the fundamental building blocks:
//! - Walker: Generic tree traversal engine
//! - NodeVisitor: Trait for visiting Figma nodes
//! - NodeContext: Context information during traversal
//! - Config: Code generation configuration
//! - Error: Unified error type

pub mod config;
pub mod error;
pub mod walker;

// Re-export commonly used types
pub use config::{
    CodeGenConfig, CodeGenConfigBuilder, CodeGenResult, CodeGenSystem, CodeGenerator,
    FileNamingStrategy, SvgContainerMode, TextInSvgMode, VectorExportStrategy,
};
pub use error::{Error, Result};
pub use walker::{NodeContext, NodeVisitor, Walker};
