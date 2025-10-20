pub mod export_config;
pub mod parser;
pub mod path_registry;
pub mod postprocessor;
pub mod strategy;
pub mod utils;

// Re-export commonly used types
pub use export_config::VectorExportConfig;
pub use parser::SvgParser;
pub use path_registry::PathRegistry;
pub use postprocessor::SvgPostprocessor;
pub use strategy::{NodeAnalyzer, RenderingStrategy, SvgConfig};
pub use utils::{
    extract_first_paint_color, fill_to_svg_attr, gradient_to_css, paint_to_css, rgba_to_css,
    stroke_to_svg_attrs,
};
