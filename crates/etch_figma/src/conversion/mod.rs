//! Conversion phase: Figma data to intermediate representations
//!
//! This module handles the conversion of Figma node data into intermediate
//! representations (JSX, HTML elements, SVG, etc.) that can then be emitted
//! by generators.

pub mod jsx;
pub mod styles;
pub mod svg;

// Re-export key types
pub use jsx::{RenderContext, ToJsx, extract_position_from_transform};
pub use styles::{
    LayoutMode, PositionType, create_absolute_position_classes, create_flex_classes,
    create_padding_classes, create_position_classes,
};
pub use svg::{
    AsyncSvgProcessor, FigmaSvgExporter, PathRegistry, SvgConfig, SvgGroupingManager, SvgParser,
    VectorExportConfig,
};
