pub mod async_bridge;
pub mod async_processor;
pub mod exporter;
pub mod grouper;
pub mod wrapper;

// Re-export commonly used types
pub use async_bridge::{
    SvgBridgeError, SvgRequest, SvgResponse, SvgWorkerPool, create_svg_channels,
};
pub use async_processor::{AsyncSvgProcessor, AsyncSvgResult, ProcessingStats};
pub use exporter::FigmaSvgExporter;
pub use grouper::{GroupedSvgElement, PositioningUtils, SvgGroupingManager};
pub use wrapper::create_svg_wrapper;
