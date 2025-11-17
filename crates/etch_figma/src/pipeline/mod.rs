//! Orchestration pipeline for Figma code generation
//!
//! This module provides high-level orchestration for the entire code generation
//! process, including:
//! - Coordinating walker, visitor, and generator
//! - Async SVG processing
//! - Post-processing with TSX visitors
//! - Builder pattern for easy configuration

pub mod unified;

// Re-export main types
pub use unified::{UnifiedPipeline, UnifiedPipelineBuilder};

