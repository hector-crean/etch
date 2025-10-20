use crate::walker::NodeVisitor;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Generic code generation result containing file paths and their contents
#[derive(Debug, Clone)]
pub struct CodeGenResult {
    /// Map of file paths to file contents
    pub files: HashMap<PathBuf, String>,
    /// Optional metadata about the generation process
    pub metadata: HashMap<String, String>,
}

impl CodeGenResult {
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
    pub fn write_to_filesystem(&self, base_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

/// Generic code generator trait that works with any visitor
/// This trait defines how to generate code from visitor results
pub trait CodeGenerator<V: NodeVisitor> {
    /// Generate code from a visitor's results
    fn generate(&self, visitor: &V) -> Result<CodeGenResult, Box<dyn std::error::Error>>;

    /// Get the file extension for this generator
    fn file_extension(&self) -> &str;

    /// Get the default output directory name for this generator
    fn output_directory(&self) -> &str;
}

/// Configuration for code generation
#[derive(Debug, Clone)]
pub struct CodeGenConfig {
    /// Base output directory
    pub output_dir: PathBuf,
    /// Whether to create separate files for each component
    pub separate_files: bool,
    /// Whether to only generate exportable components
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
    pub vector_export_strategy: VectorExportStrategy,
    pub svg_postprocess_enabled: bool,
    
    // SVG container strategy
    pub svg_container_mode: SvgContainerMode,
    pub svg_responsive_mode: bool, // width/height 100%, viewBox set
    
    // Text handling
    pub detect_superscripts: bool,
    pub text_in_svg_mode: TextInSvgMode,
    
    // Responsive
    pub enable_container_queries: bool,
    pub responsive_breakpoint_px: f64, // default 640px
}

#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub enum VectorExportStrategy {
    ManualConversion,     // Use internal path conversion
    FigmaApi,             // Use Figma's export API
    Hybrid,               // API + postprocess, fallback to manual
}

#[derive(Debug, Clone)]
pub enum SvgContainerMode {
    WrapAll,              // Wrap all descendants in single SVG
    Mixed,                // Try to preserve HTML layout where possible
    Configurable,         // Per-node decision based on properties
}

#[derive(Debug, Clone)]
pub enum TextInSvgMode {
    HtmlOverlay,          // Position HTML text over SVG
    SvgText,              // Convert to SVG <text> elements
    Auto,                 // Decide based on complexity
}

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

/// Generic code generation system that works with any visitor and generator
pub struct CodeGenSystem<V: NodeVisitor, G: CodeGenerator<V>> {
    visitor: V,
    generator: G,
    config: CodeGenConfig,
}

impl<V: NodeVisitor, G: CodeGenerator<V>> CodeGenSystem<V, G> {
    pub fn new(visitor: V, generator: G, config: CodeGenConfig) -> Self {
        Self {
            visitor,
            generator,
            config,
        }
    }

    /// Generate code and write to filesystem
    pub fn generate_and_write(&self) -> Result<CodeGenResult, Box<dyn std::error::Error>> {
        let result = self.generator.generate(&self.visitor)?;
        result.write_to_filesystem(&self.config.output_dir)?;
        Ok(result)
    }

    /// Generate code without writing to filesystem
    pub fn generate(&self) -> Result<CodeGenResult, Box<dyn std::error::Error>> {
        self.generator.generate(&self.visitor)
    }
}
