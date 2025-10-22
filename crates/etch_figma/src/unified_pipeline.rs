use crate::codegen_ext::{CodeGenConfig, CodeGenResult};
use crate::svg::strategy::SvgConfig;
use crate::tsx::generator::TsxGenerator;
use crate::tsx::svgr::async_processor::AsyncSvgProcessor;
use crate::tsx::visitor::TsxVisitor;
use crate::walker::Walker;
use figma_api::models::CanvasNode;
use log::info;
use std::io::Write;
use tempfile::NamedTempFile;

/// Unified pipeline that combines Figma generation with TSX post-processing
pub struct UnifiedPipeline {
    figma_config: CodeGenConfig,
    tsx_visitors: Vec<Box<dyn swc_ecma_visit::VisitMut>>,
    file_key: Option<String>,
}

impl UnifiedPipeline {
    /// Create a new unified pipeline
    pub fn new(figma_config: CodeGenConfig) -> Self {
        Self {
            figma_config,
            tsx_visitors: Vec::new(),
            file_key: None,
        }
    }

    /// Add a TSX visitor for post-processing
    pub fn add_tsx_visitor<V: swc_ecma_visit::VisitMut + 'static>(
        &mut self,
        visitor: V,
    ) -> &mut Self {
        self.tsx_visitors.push(Box::new(visitor));
        self
    }

    /// Process a Figma canvas through the unified pipeline
    pub async fn process_canvas(
        &mut self,
        canvas: &CanvasNode,
    ) -> Result<CodeGenResult, Box<dyn std::error::Error>> {
        // Step 1: Generate TSX from Figma using etch_figma
        let figma_result = self.generate_figma_tsx(canvas).await?;

        // Step 2: Post-process the generated TSX using etch_tsx visitors
        let processed_result = self.postprocess_tsx(figma_result)?;

        Ok(processed_result)
    }

    /// Generate TSX from Figma data using the simplified async processor
    async fn generate_figma_tsx(
        &self,
        canvas: &CanvasNode,
    ) -> Result<CodeGenResult, Box<dyn std::error::Error>> {
        info!("Generating TSX from Figma canvas");

        // Create visitor with configuration
        let mut visitor = TsxVisitor::with_config(self.figma_config.clone());
        if let Some(file_key) = &self.file_key {
            visitor.set_file_key(file_key.clone());
        }

        // First pass: traverse the canvas to collect all vector nodes
        let mut tsx_visitor = Walker::new(visitor).walk_canvas(canvas);

        // Process any vectors that need async SVG fetching
        if !tsx_visitor.pending_vectors().is_empty() && self.file_key.is_some() {
            info!("Processing {} vectors with async SVG processor", tsx_visitor.pending_vectors().len());
            self.process_vectors_async(&mut tsx_visitor).await?;
        }

        // Process SVG groups to optimize SVG containers and positioning
        tsx_visitor.process_svg_groups();

        // Create TSX generator with configuration
        let tsx_generator = TsxGenerator::new()
            .with_react_imports(true)
            .with_separate_files(true)
            .with_exportable_only(true)
            .with_config(self.figma_config.clone());

        // Generate code
        let codegen_system = crate::codegen_ext::CodeGenSystem::new(
            tsx_visitor,
            tsx_generator,
            self.figma_config.clone(),
        );
        let result = codegen_system.generate()?;

        Ok(result)
    }

    /// Process vectors using the intelligent async processor
    async fn process_vectors_async(
        &self,
        tsx_visitor: &mut crate::tsx::visitor::TsxVisitor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file_key = self.file_key.as_ref()
            .ok_or("File key required for vector processing")?;

        // Create async processor with intelligent categorization
        let mut processor = AsyncSvgProcessor::new(
            file_key.clone(),
            SvgConfig::default(),
        )
        .with_concurrency(10)
        .with_timeout(std::time::Duration::from_secs(60));

        // Extract pending vectors from the visitor
        let pending_vectors = std::mem::take(tsx_visitor.pending_vectors_mut());
        
        if pending_vectors.is_empty() {
            return Ok(());
        }

        // Process all vectors intelligently (API + inline)
        let result = processor.process_vectors(pending_vectors).await?;

        info!(
            "Vector processing completed: {}/{} successful ({}ms, batch: {})",
            result.stats.successful,
            result.stats.total_requested,
            result.stats.processing_time_ms,
            result.stats.batch_prefetch_used
        );

        // Update JSX elements with the processed SVG content
        for (node_id, svg_content) in result.svg_content {
            let jsx_element = tsx_visitor.create_jsx_from_svg_content(&svg_content);
            tsx_visitor.jsx_elements_mut().insert(node_id.clone(), jsx_element);
            tsx_visitor.placeholder_jsx_mut().remove(&node_id);
        }

        // Handle any errors - these are already logged by the processor
        if !result.errors.is_empty() {
            info!("Some vectors failed processing but placeholders remain in place");
        }

        Ok(())
    }


    /// Post-process generated TSX files using etch_tsx visitors
    fn postprocess_tsx(
        &mut self,
        mut figma_result: CodeGenResult,
    ) -> Result<CodeGenResult, Box<dyn std::error::Error>> {
        if self.tsx_visitors.is_empty() {
            return Ok(figma_result);
        }

        // Process each generated file
        let mut processed_files = std::collections::HashMap::new();

        for (file_path, content) in figma_result.files.iter() {
            let processed_content = self.process_tsx_content(content)?;
            processed_files.insert(file_path.clone(), processed_content);
        }

        // Update the result with processed content
        figma_result.files = processed_files;

        Ok(figma_result)
    }

    /// Process a single TSX file content through all visitors
    fn process_tsx_content(&mut self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Create a temporary file with the TSX content
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?;

        // Parse the TSX file using etch_tsx
        let (cm, mut module) = etch_tsx::file::parse_tsx_file(temp_file.path())?;

        // Apply each visitor in order
        for visitor in &mut self.tsx_visitors {
            visitor.visit_mut_module(&mut module);
        }

        // Generate the output code
        let mut output = Vec::new();
        {
            let writer =
                swc_ecma_codegen::text_writer::JsWriter::new(cm.clone(), "\n", &mut output, None);
            let mut emitter = swc_ecma_codegen::Emitter {
                cfg: swc_ecma_codegen::Config::default(),
                cm: cm.clone(),
                comments: None,
                wr: writer,
            };
            emitter.emit_module(&module)?;
        }

        // Convert output to string
        let processed_tsx = String::from_utf8(output)?;
        Ok(processed_tsx)
    }
}

/// Builder for creating a unified pipeline with common configurations
pub struct UnifiedPipelineBuilder {
    figma_config: CodeGenConfig,
    tsx_visitors: Vec<Box<dyn swc_ecma_visit::VisitMut>>,
    file_key: Option<String>,
}

impl UnifiedPipelineBuilder {
    /// Create a new builder with default Figma configuration
    pub fn new() -> Self {
        Self {
            figma_config: CodeGenConfig::default(),
            tsx_visitors: Vec::new(),
            file_key: None,
        }
    }

    /// Set the Figma configuration
    pub fn with_figma_config(mut self, config: CodeGenConfig) -> Self {
        self.figma_config = config;
        self
    }

    /// Add Figma SVG visitor for superscript fixes and path optimization
    pub fn with_figma_svg_processing(mut self) -> Self {
        use etch_tsx::visitor::figma_svg_visitor::{FigmaSvgConfig, FigmaSvgVisitor};
        let config = FigmaSvgConfig {
            fix_superscripts: true,
            optimize_paths: true,
            extract_text_elements: false,
        };
        self.tsx_visitors
            .push(Box::new(FigmaSvgVisitor::with_config(config)));
        self
    }

    /// Add Framer Motion visitor for animations
    pub fn with_framer_motion(mut self) -> Self {
        use etch_tsx::visitor::framer_motion_visitor::FramerMotionVisitor;
        use std::collections::HashMap;
        let animations: HashMap<String, _> = HashMap::new();
        self.tsx_visitors.push(Box::new(FramerMotionVisitor::<
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
        >::new(animations)));
        self
    }

    /// Add UUID injection visitor
    pub fn with_uuid_injection(mut self) -> Self {
        use etch_tsx::visitor::inject_uuid_visitor::{InjectUuidPolicy, InjectUuidVisitor};
        self.tsx_visitors.push(Box::new(InjectUuidVisitor::new(
            InjectUuidPolicy::Overwrite,
        )));
        self
    }

    /// Add custom visitor
    pub fn with_visitor<V: swc_ecma_visit::VisitMut + 'static>(mut self, visitor: V) -> Self {
        self.tsx_visitors.push(Box::new(visitor));
        self
    }

    /// Set the file key for Figma API SVG export
    pub fn with_file_key(mut self, file_key: String) -> Self {
        self.file_key = Some(file_key);
        self
    }

    /// Build the unified pipeline
    pub fn build(self) -> UnifiedPipeline {
        let mut pipeline = UnifiedPipeline::new(self.figma_config);
        pipeline.tsx_visitors = self.tsx_visitors;
        pipeline.file_key = self.file_key;
        pipeline
    }
}

impl Default for UnifiedPipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen_ext::{SvgContainerMode, TextInSvgMode, VectorExportStrategy};

    #[test]
    fn test_unified_pipeline_builder() {
        let config = CodeGenConfig {
            vector_export_strategy: VectorExportStrategy::Hybrid,
            svg_container_mode: SvgContainerMode::WrapAll,
            text_in_svg_mode: TextInSvgMode::Auto,
            ..Default::default()
        };

        let pipeline = UnifiedPipelineBuilder::new()
            .with_figma_config(config)
            .with_figma_svg_processing()
            .with_uuid_injection()
            .build();

        // Verify the pipeline was created with the correct configuration
        assert!(matches!(
            pipeline.figma_config.vector_export_strategy,
            VectorExportStrategy::Hybrid
        ));
        assert!(!pipeline.tsx_visitors.is_empty());
    }

    #[test]
    fn test_pipeline_without_visitors() {
        let config = CodeGenConfig::default();
        let pipeline = UnifiedPipelineBuilder::new()
            .with_figma_config(config)
            .build();

        assert!(pipeline.tsx_visitors.is_empty());
    }
}
