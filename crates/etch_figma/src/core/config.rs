//! Code generation configuration and result types

use super::error::{Error, Result};
use super::walker::NodeVisitor;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Generic code generation result containing file paths and their contents
///
/// This is the output of any code generator, containing the generated files
/// and optional metadata about the generation process.
#[derive(Debug, Clone)]
pub struct CodeGenResult {
    /// Map of file paths to file contents
    pub files: HashMap<PathBuf, String>,
    /// Optional metadata about the generation process
    pub metadata: HashMap<String, String>,
}

impl CodeGenResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a file to the result
    pub fn add_file(&mut self, path: PathBuf, content: String) {
        self.files.insert(path, content);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Write all files to the filesystem
    pub fn write_to_filesystem(&self, base_path: &Path) -> Result<()> {
        for (file_path, content) in &self.files {
            let full_path = base_path.join(file_path);

            // Create parent directories if they don't exist
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&full_path, content)?;
        }
        Ok(())
    }
}

impl Default for CodeGenResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic code generator trait that works with any visitor
///
/// This trait defines how to generate code from visitor results. Implement this
/// to create new output formats (TSX, HTML, CSS, etc.).
///
/// # Example
///
/// ```rust,ignore
/// use etch_figma::core::{CodeGenerator, CodeGenResult};
///
/// struct MyGenerator;
///
/// impl CodeGenerator<MyVisitor> for MyGenerator {
///     fn generate(&self, visitor: &MyVisitor) -> Result<CodeGenResult> {
///         let mut result = CodeGenResult::new();
///         // Generate code from visitor data
///         result.add_file(PathBuf::from("output.txt"), "content".to_string());
///         Ok(result)
///     }
///
///     fn file_extension(&self) -> &str { "txt" }
///     fn output_directory(&self) -> &str { "output" }
/// }
/// ```
pub trait CodeGenerator<V: NodeVisitor> {
    /// Generate code from a visitor's results
    fn generate(&self, visitor: &V) -> Result<CodeGenResult>;

    /// Get the file extension for this generator
    fn file_extension(&self) -> &str;

    /// Get the default output directory name for this generator
    fn output_directory(&self) -> &str;
}

/// Configuration for code generation
///
/// Controls all aspects of the code generation process, from output formatting
/// to SVG export strategies.
#[derive(Debug, Clone)]
pub struct CodeGenConfig {
    /// Base output directory
    pub output_dir: PathBuf,

    /// Whether to create separate files for each component
    pub separate_files: bool,

    /// Whether to only generate exportable components (those with export settings in Figma)
    pub exportable_only: bool,

    /// Custom file naming strategy
    pub file_naming: FileNamingStrategy,

    /// Whether to use CSS variables for theming
    pub use_css_variables: bool,

    /// Whether to extract complex SVG paths to external file
    pub extract_svg_paths: bool,

    /// Name of the SVG paths file (without extension)
    pub svg_paths_filename: String,

    // Vector export strategy
    /// How to handle vector graphics export
    pub vector_export_strategy: VectorExportStrategy,

    /// Whether to post-process exported SVGs
    pub svg_postprocess_enabled: bool,

    // SVG container strategy
    /// How to wrap SVG elements in containers
    pub svg_container_mode: SvgContainerMode,

    /// Whether to make SVGs responsive (width/height 100%, viewBox set)
    pub svg_responsive_mode: bool,

    // Text handling
    /// Whether to detect and convert superscripts
    pub detect_superscripts: bool,

    /// How to handle text within SVG contexts
    pub text_in_svg_mode: TextInSvgMode,

    // Responsive design
    /// Whether to use CSS container queries
    pub enable_container_queries: bool,

    /// Default responsive breakpoint in pixels
    pub responsive_breakpoint_px: f64,
}

/// Strategy for exporting vector graphics
#[derive(Debug, Clone, PartialEq)]
pub enum VectorExportStrategy {
    /// Use internal path conversion only (fast, but may not handle complex vectors)
    ManualConversion,

    /// Use Figma's export API for all vectors (high fidelity, but requires API calls)
    FigmaApi,

    /// Use API for complex vectors, internal conversion for simple shapes, with fallback (recommended)
    Hybrid,
}

/// Strategy for wrapping SVG elements in containers
#[derive(Debug, Clone)]
pub enum SvgContainerMode {
    /// Wrap all descendants in single SVG container
    WrapAll,

    /// Try to preserve HTML layout where possible, use SVG when necessary
    Mixed,

    /// Make per-node decision based on node properties (transforms, masks, etc.)
    Configurable,
}

/// Strategy for handling text within SVG contexts
#[derive(Debug, Clone)]
pub enum TextInSvgMode {
    /// Position HTML text over SVG (better text rendering, complex positioning)
    HtmlOverlay,

    /// Convert to SVG <text> elements (simpler, but limited text features)
    SvgText,

    /// Automatically decide based on text complexity
    Auto,
}

/// Strategy for naming output files
#[derive(Debug, Clone)]
pub enum FileNamingStrategy {
    /// Use the component name as filename
    ComponentName,

    /// Use the node ID as filename
    NodeId,

    /// Use a custom naming function
    Custom(fn(&str, &str) -> String),
}

impl Default for CodeGenConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("generated"),
            separate_files: true,
            exportable_only: false,
            file_naming: FileNamingStrategy::ComponentName,
            use_css_variables: false,
            extract_svg_paths: true,
            svg_paths_filename: "svg-paths".to_string(),
            vector_export_strategy: VectorExportStrategy::Hybrid,
            svg_postprocess_enabled: true,
            svg_container_mode: SvgContainerMode::WrapAll,
            svg_responsive_mode: true,
            detect_superscripts: true,
            text_in_svg_mode: TextInSvgMode::Auto,
            enable_container_queries: true,
            responsive_breakpoint_px: 640.0,
        }
    }
}

impl CodeGenConfig {
    /// Create a new builder for CodeGenConfig
    pub fn builder() -> CodeGenConfigBuilder {
        CodeGenConfigBuilder::new()
    }
}

/// Builder for CodeGenConfig with a fluent API
///
/// # Example
///
/// ```rust,ignore
/// use etch_figma::core::{CodeGenConfig, VectorExportStrategy};
///
/// let config = CodeGenConfig::builder()
///     .output_dir("./output")
///     .separate_files(true)
///     .exportable_only(true)
///     .vector_strategy(VectorExportStrategy::Hybrid)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct CodeGenConfigBuilder {
    config: CodeGenConfig,
}

impl CodeGenConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            config: CodeGenConfig::default(),
        }
    }

    /// Set the output directory
    pub fn output_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.output_dir = path.into();
        self
    }

    /// Set whether to create separate files for each component
    pub fn separate_files(mut self, separate: bool) -> Self {
        self.config.separate_files = separate;
        self
    }

    /// Set whether to only generate exportable components
    pub fn exportable_only(mut self, exportable: bool) -> Self {
        self.config.exportable_only = exportable;
        self
    }

    /// Set the file naming strategy
    pub fn file_naming(mut self, strategy: FileNamingStrategy) -> Self {
        self.config.file_naming = strategy;
        self
    }

    /// Set whether to use CSS variables for theming
    pub fn use_css_variables(mut self, use_vars: bool) -> Self {
        self.config.use_css_variables = use_vars;
        self
    }

    /// Set whether to extract complex SVG paths to external file
    pub fn extract_svg_paths(mut self, extract: bool) -> Self {
        self.config.extract_svg_paths = extract;
        self
    }

    /// Set the SVG paths filename (without extension)
    pub fn svg_paths_filename(mut self, filename: impl Into<String>) -> Self {
        self.config.svg_paths_filename = filename.into();
        self
    }

    /// Set the vector export strategy
    pub fn vector_strategy(mut self, strategy: VectorExportStrategy) -> Self {
        self.config.vector_export_strategy = strategy;
        self
    }

    /// Set whether to post-process exported SVGs
    pub fn svg_postprocess(mut self, enabled: bool) -> Self {
        self.config.svg_postprocess_enabled = enabled;
        self
    }

    /// Set the SVG container mode
    pub fn svg_container_mode(mut self, mode: SvgContainerMode) -> Self {
        self.config.svg_container_mode = mode;
        self
    }

    /// Set whether to make SVGs responsive
    pub fn svg_responsive(mut self, responsive: bool) -> Self {
        self.config.svg_responsive_mode = responsive;
        self
    }

    /// Set whether to detect and convert superscripts
    pub fn detect_superscripts(mut self, detect: bool) -> Self {
        self.config.detect_superscripts = detect;
        self
    }

    /// Set how to handle text within SVG contexts
    pub fn text_in_svg_mode(mut self, mode: TextInSvgMode) -> Self {
        self.config.text_in_svg_mode = mode;
        self
    }

    /// Set whether to use CSS container queries
    pub fn container_queries(mut self, enabled: bool) -> Self {
        self.config.enable_container_queries = enabled;
        self
    }

    /// Set the responsive breakpoint in pixels
    pub fn responsive_breakpoint(mut self, pixels: f64) -> Self {
        self.config.responsive_breakpoint_px = pixels;
        self
    }

    /// Build the final configuration
    pub fn build(self) -> CodeGenConfig {
        self.config
    }
}

impl Default for CodeGenConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic code generation system that works with any visitor and generator
///
/// This orchestrates the code generation process, tying together a visitor
/// (which collects data) and a generator (which produces output).
///
/// # Example
///
/// ```rust,ignore
/// use etch_figma::core::{Walker, CodeGenSystem, CodeGenConfig};
/// use etch_figma::generators::tsx::{TsxVisitor, TsxGenerator};
///
/// let config = CodeGenConfig::default();
/// let visitor = TsxVisitor::new();
/// let visitor = Walker::new(visitor).walk_canvas(&canvas);
/// let generator = TsxGenerator::new();
/// let system = CodeGenSystem::new(visitor, generator, config);
/// let result = system.generate_and_write()?;
/// ```
pub struct CodeGenSystem<V: NodeVisitor, G: CodeGenerator<V>> {
    visitor: V,
    generator: G,
    config: CodeGenConfig,
}

impl<V: NodeVisitor, G: CodeGenerator<V>> CodeGenSystem<V, G> {
    /// Create a new code generation system
    pub fn new(visitor: V, generator: G, config: CodeGenConfig) -> Self {
        Self {
            visitor,
            generator,
            config,
        }
    }

    /// Generate code and write to filesystem
    pub fn generate_and_write(&self) -> Result<CodeGenResult> {
        let result = self.generator.generate(&self.visitor)?;
        result.write_to_filesystem(&self.config.output_dir)?;
        Ok(result)
    }

    /// Generate code without writing to filesystem
    pub fn generate(&self) -> Result<CodeGenResult> {
        self.generator.generate(&self.visitor)
    }
}
