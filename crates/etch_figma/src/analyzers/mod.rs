/// Common trait for all analyzers in the system
pub trait Analyzer {
    /// Type of input the analyzer processes
    type Input;
    /// Type of output/analysis result
    type Output;

    /// Perform analysis on the input
    fn analyze(input: &Self::Input) -> Self::Output;
}

pub mod layout;
pub mod responsive;
pub mod svg_container;
pub mod text;

// Re-export the analyzer types for convenience
pub use layout::{LayoutAnalysis, LayoutAnalyzer};
pub use responsive::ResponsiveAnalyzer;
pub use svg_container::{ContainerAnalysis, SvgContainerAnalyzer, SvgWrapperConfig};
pub use text::TextAnalyzer;
