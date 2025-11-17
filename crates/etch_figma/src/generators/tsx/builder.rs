//! Builder pattern for TsxGenerator
//!
//! Provides a fluent API for configuring and creating TSX generators.

use std::cell::RefCell;
use crate::conversion::svg::PathRegistry;
use crate::core::config::CodeGenConfig;
use super::generator::TsxGenerator;

/// Builder for configuring TSX generator
///
/// # Example
///
/// ```rust,ignore
/// use etch_figma::generators::tsx::TsxGeneratorBuilder;
///
/// let generator = TsxGeneratorBuilder::new()
///     .with_react_imports(true)
///     .with_separate_files(true)
///     .with_exportable_only(false)
///     .build();
/// ```
#[derive(Debug)]
pub struct TsxGeneratorBuilder {
    include_react_imports: bool,
    separate_files: bool,
    exportable_only: bool,
    config: Option<CodeGenConfig>,
}

impl TsxGeneratorBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            include_react_imports: true,
            separate_files: true,
            exportable_only: false,
            config: None,
        }
    }

    /// Whether to include React imports in generated files
    ///
    /// Default: `true`
    pub fn with_react_imports(mut self, include: bool) -> Self {
        self.include_react_imports = include;
        self
    }

    /// Whether to create separate files for each component
    ///
    /// Default: `true`
    pub fn with_separate_files(mut self, separate: bool) -> Self {
        self.separate_files = separate;
        self
    }

    /// Whether to only generate components marked as exportable in Figma
    ///
    /// Default: `false`
    pub fn with_exportable_only(mut self, exportable_only: bool) -> Self {
        self.exportable_only = exportable_only;
        self
    }

    /// Set advanced configuration options
    pub fn with_config(mut self, config: CodeGenConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Build the TsxGenerator with the configured options
    pub fn build(self) -> TsxGenerator {
        TsxGenerator {
            include_react_imports: self.include_react_imports,
            separate_files: self.separate_files,
            exportable_only: self.exportable_only,
            path_registry: RefCell::new(PathRegistry::new()),
            config: self.config,
        }
    }
}

impl Default for TsxGeneratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Shorthand builder functions for common configurations
impl TsxGeneratorBuilder {
    /// Create a builder configured for production use
    ///
    /// - React imports included
    /// - Separate files for each component
    /// - Only exportable components
    pub fn production() -> Self {
        Self::new()
            .with_react_imports(true)
            .with_separate_files(true)
            .with_exportable_only(true)
    }

    /// Create a builder configured for development/prototyping
    ///
    /// - React imports included
    /// - Single file output
    /// - All components (not just exportable)
    pub fn development() -> Self {
        Self::new()
            .with_react_imports(true)
            .with_separate_files(false)
            .with_exportable_only(false)
    }

    /// Create a builder configured for library/package generation
    ///
    /// - No React imports (assuming consumer provides)
    /// - Separate files
    /// - Only exportable components
    pub fn library() -> Self {
        Self::new()
            .with_react_imports(false)
            .with_separate_files(true)
            .with_exportable_only(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let generator = TsxGeneratorBuilder::new().build();
        assert_eq!(generator.include_react_imports, true);
        assert_eq!(generator.separate_files, true);
        assert_eq!(generator.exportable_only, false);
    }

    #[test]
    fn test_builder_production() {
        let generator = TsxGeneratorBuilder::production().build();
        assert_eq!(generator.include_react_imports, true);
        assert_eq!(generator.separate_files, true);
        assert_eq!(generator.exportable_only, true);
    }

    #[test]
    fn test_builder_development() {
        let generator = TsxGeneratorBuilder::development().build();
        assert_eq!(generator.include_react_imports, true);
        assert_eq!(generator.separate_files, false);
        assert_eq!(generator.exportable_only, false);
    }

    #[test]
    fn test_builder_library() {
        let generator = TsxGeneratorBuilder::library().build();
        assert_eq!(generator.include_react_imports, false);
        assert_eq!(generator.separate_files, true);
        assert_eq!(generator.exportable_only, true);
    }
}

