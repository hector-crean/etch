//! SVG conversion and optimization
//!
//! This module handles all SVG-related operations:
//! - Inline SVG generation from simple shapes
//! - API-based SVG export for complex vectors
//! - SVG grouping and optimization
//! - Path extraction to external files
//! - SVG parsing and postprocessing
//! - Strategy patterns for rendering

pub mod async_bridge;
pub mod async_processor;
pub mod export_config;
pub mod exporter;
pub mod grouper;
pub mod parser;
pub mod path_registry;
pub mod postprocessor;
pub mod strategy;
pub mod utils;
pub mod wrapper;

// Re-export commonly used types with clearer names
pub use async_bridge::{
    SvgBridgeError, SvgRequest, SvgResponse, SvgWorkerPool, create_svg_channels,
};
pub use async_processor::{AsyncSvgProcessor, AsyncSvgResult, ProcessingStats};
pub use export_config::VectorExportConfig;
pub use exporter::FigmaSvgExporter;
pub use grouper::{GroupedSvgElement, PositioningUtils, SvgGroupingManager};
pub use parser::SvgParser;
pub use path_registry::PathRegistry;
pub use postprocessor::SvgPostprocessor;
pub use strategy::{NodeAnalyzer, PathComplexity, RenderingStrategy, SvgConfig};
pub use utils::{
    extract_first_paint_color, fill_to_svg_attr, gradient_to_css, paint_to_css, rgba_to_css,
    stroke_to_svg_attrs,
};
pub use wrapper::create_svg_wrapper;
