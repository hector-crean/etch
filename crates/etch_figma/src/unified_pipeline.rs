use crate::codegen_ext::{CodeGenConfig, CodeGenResult};
use crate::svg::strategy::SvgConfig;
use crate::tsx::generator::TsxGenerator;
use crate::tsx::svgr::async_bridge::{SvgWorkerPool, create_svg_channels};
use crate::tsx::svgr::exporter::FigmaSvgExporter;
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

    /// Generate TSX from Figma data
    async fn generate_figma_tsx(
        &self,
        canvas: &CanvasNode,
    ) -> Result<CodeGenResult, Box<dyn std::error::Error>> {
        // Check if we should use async SVG processing
        let use_async_svg = self.figma_config.vector_export_strategy
            == crate::codegen_ext::VectorExportStrategy::FigmaApi
            && self.file_key.is_some();

        if use_async_svg {
            self.generate_figma_tsx_async(canvas).await
        } else {
            self.generate_figma_tsx_sync(canvas).await
        }
    }

    /// Generate TSX using async SVG processing
    async fn generate_figma_tsx_async(
        &self,
        canvas: &CanvasNode,
    ) -> Result<CodeGenResult, Box<dyn std::error::Error>> {
        info!("Using async SVG processing for Figma generation");

        // Create channels for async SVG processing
        let (request_tx, request_rx, response_tx, mut response_rx) = create_svg_channels();

        // Create SVG exporter and worker pool
        let exporter = FigmaSvgExporter::new(SvgConfig::default());
        let worker_pool = SvgWorkerPool::new(request_rx, response_tx, exporter, 10); // Max 10 concurrent requests

        // Spawn worker pool in background
        let worker_handle = tokio::spawn(async move {
            if let Err(e) = worker_pool.run().await {
                log::error!("SVG worker pool failed: {}", e);
            }
        });

        // Create visitor with async channel (clone the sender for later use)
        let request_tx_clone = request_tx.clone();
        let mut visitor = TsxVisitor::with_async_svg_channel(self.figma_config.clone(), request_tx);

        if let Some(file_key) = &self.file_key {
            visitor.set_file_key(file_key.clone());
        }

        // Run synchronous visitor traversal
        info!("Starting synchronous visitor traversal");
        let mut tsx_visitor = Walker::new(visitor).walk_canvas(canvas);

        // Resolve pending vectors with async responses
        info!("Resolving pending vectors with async responses");
        tsx_visitor
            .resolve_pending_vectors(&mut response_rx)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        // Process SVG groups to optimize SVG containers and positioning
        tsx_visitor.process_svg_groups();

        // Close the request channel to signal the worker pool to stop
        drop(request_tx_clone);
        info!("Closing SVG request channel to signal worker pool shutdown");

        // Wait for worker pool to complete with timeout
        use tokio::time::{Duration, timeout};
        let worker_timeout = Duration::from_secs(60); // 60 second timeout for worker pool shutdown

        match timeout(worker_timeout, worker_handle).await {
            Ok(Ok(())) => {
                info!("✓ SVG worker pool shutdown completed successfully");
            }
            Ok(Err(e)) => {
                log::error!("SVG worker pool task panicked: {}", e);
            }
            Err(_) => {
                log::warn!(
                    "⚠ SVG worker pool shutdown timed out after {} seconds - some tasks may still be running",
                    worker_timeout.as_secs()
                );
            }
        }

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

    /// Generate TSX using synchronous processing (fallback)
    async fn generate_figma_tsx_sync(
        &self,
        canvas: &CanvasNode,
    ) -> Result<CodeGenResult, Box<dyn std::error::Error>> {
        info!("Using synchronous SVG processing for Figma generation");

        // Create visitor with configuration and file key
        let mut visitor = TsxVisitor::with_config(self.figma_config.clone());
        if let Some(file_key) = &self.file_key {
            visitor.set_file_key(file_key.clone());
        }

        // Pre-fetch SVG data for vector nodes if using FigmaApi strategy
        if self.figma_config.vector_export_strategy
            == crate::codegen_ext::VectorExportStrategy::FigmaApi
        {
            if let Some(file_key) = &self.file_key {
                self.prefetch_svg_data(canvas, file_key, &mut visitor)
                    .await?;
            }
        }

        // Extract ALL node components using the unified visitor with configuration
        let mut tsx_visitor = Walker::new(visitor).walk_canvas(canvas);

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

    /// Pre-fetch SVG data for all vector nodes in the canvas
    async fn prefetch_svg_data(
        &self,
        _canvas: &CanvasNode,
        _file_key: &str,
        _visitor: &mut TsxVisitor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // For now, we'll skip pre-fetching and implement a simpler approach
        // The visitor will handle the strategy selection and fallback appropriately
        info!(
            "Skipping pre-fetch for now - vectors will use inline conversion with API strategy detection"
        );

        // We could implement actual pre-fetching here by:
        // 1. Traversing the canvas to find all vector nodes
        // 2. Calling visitor.get_svg_exporter().fetch_and_cache_svg() for each vector
        // 3. But this requires complex node traversal that we're avoiding for now

        Ok(())
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
