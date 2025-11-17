use super::async_bridge::{
    SvgBridgeError, SvgRequest, SvgResponse, SvgWorkerPool, create_svg_channels,
};
use super::exporter::FigmaSvgExporter;
use super::strategy::SvgConfig;
use figma_api::models::VectorNode;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};

/// High-level async SVG processor that encapsulates all async complexity
pub struct AsyncSvgProcessor {
    file_key: String,
    exporter: FigmaSvgExporter,
    max_concurrent: usize,
    timeout_duration: Duration,
}

/// Result of async SVG processing
#[derive(Debug)]
pub struct AsyncSvgResult {
    /// Successfully processed SVG content by node ID
    pub svg_content: HashMap<String, String>,
    /// Errors encountered during processing
    pub errors: HashMap<String, SvgBridgeError>,
    /// Processing statistics
    pub stats: ProcessingStats,
}

/// Statistics about the async processing
#[derive(Debug)]
pub struct ProcessingStats {
    pub total_requested: usize,
    pub successful: usize,
    pub failed: usize,
    pub batch_prefetch_used: bool,
    pub processing_time_ms: u64,
}

impl AsyncSvgProcessor {
    /// Create a new async SVG processor
    pub fn new(file_key: String, svg_config: SvgConfig) -> Self {
        Self {
            file_key,
            exporter: FigmaSvgExporter::new(svg_config),
            max_concurrent: 10,
            timeout_duration: Duration::from_secs(60),
        }
    }

    /// Configure concurrency settings
    pub fn with_concurrency(mut self, max_concurrent: usize) -> Self {
        self.max_concurrent = max_concurrent;
        self
    }

    /// Configure timeout settings
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_duration = timeout;
        self
    }

    /// Process a collection of vector nodes intelligently
    /// This automatically decides between API fetching and inline conversion
    /// based on vector complexity and availability
    pub async fn process_vectors(
        &mut self,
        vectors: HashMap<String, VectorNode>,
    ) -> Result<AsyncSvgResult, SvgBridgeError> {
        let start_time = std::time::Instant::now();
        let total_requested = vectors.len();

        info!(
            "Starting intelligent SVG processing for {} vectors",
            total_requested
        );

        if vectors.is_empty() {
            return Ok(AsyncSvgResult {
                svg_content: HashMap::new(),
                errors: HashMap::new(),
                stats: ProcessingStats {
                    total_requested: 0,
                    successful: 0,
                    failed: 0,
                    batch_prefetch_used: false,
                    processing_time_ms: 0,
                },
            });
        }

        // Separate vectors that should use API vs inline conversion
        let (api_vectors, inline_vectors) = self.categorize_vectors(&vectors);

        info!(
            "Categorized vectors: {} for API, {} for inline conversion",
            api_vectors.len(),
            inline_vectors.len()
        );

        let mut svg_content = HashMap::new();
        let mut errors = HashMap::new();
        let mut batch_prefetch_used = false;

        // Process inline vectors immediately (they're simple enough)
        for (node_id, vector) in inline_vectors {
            match self.create_inline_svg_content(&vector) {
                Ok(content) => {
                    svg_content.insert(node_id, content);
                }
                Err(e) => {
                    errors.insert(node_id, e);
                }
            }
        }

        // Process API vectors if any
        if !api_vectors.is_empty() {
            // Try batch processing first for better performance
            let batch_result = self.try_batch_processing(&api_vectors).await;

            match batch_result {
                Ok(api_results) => {
                    batch_prefetch_used = true;
                    svg_content.extend(api_results);
                    info!(
                        "✓ Batch processing completed successfully for {} API vectors",
                        api_vectors.len()
                    );
                }
                Err(batch_error) => {
                    warn!(
                        "Batch processing failed: {}, falling back to individual processing",
                        batch_error
                    );
                    let individual_result = self
                        .fallback_to_individual_processing(api_vectors, start_time)
                        .await?;
                    svg_content.extend(individual_result.svg_content);
                    errors.extend(individual_result.errors);
                }
            }
        }

        let processing_time = start_time.elapsed().as_millis() as u64;
        let successful = svg_content.len();
        let failed = errors.len();

        info!(
            "Intelligent SVG processing completed: {}/{} successful in {}ms (batch: {})",
            successful, total_requested, processing_time, batch_prefetch_used
        );

        Ok(AsyncSvgResult {
            svg_content,
            errors,
            stats: ProcessingStats {
                total_requested,
                successful,
                failed,
                batch_prefetch_used,
                processing_time_ms: processing_time,
            },
        })
    }

    /// Categorize vectors into those that should use API vs inline conversion
    fn categorize_vectors(
        &self,
        vectors: &HashMap<String, VectorNode>,
    ) -> (HashMap<String, VectorNode>, HashMap<String, VectorNode>) {
        let mut api_vectors = HashMap::new();
        let mut inline_vectors = HashMap::new();

        for (node_id, vector) in vectors {
            if self.should_use_api_for_vector(vector) {
                api_vectors.insert(node_id.clone(), vector.clone());
            } else {
                inline_vectors.insert(node_id.clone(), vector.clone());
            }
        }

        (api_vectors, inline_vectors)
    }

    /// Determine if a vector should use the Figma API or inline conversion
    fn should_use_api_for_vector(&self, vector: &VectorNode) -> bool {
        // Use API for complex vectors that would benefit from Figma's rendering
        // Use inline for simple shapes that we can handle well ourselves

        // If no fill geometry, definitely use API
        if vector.fill_geometry.is_none() || vector.fill_geometry.as_ref().unwrap().is_empty() {
            return true;
        }

        let fill_geometry = vector.fill_geometry.as_ref().unwrap();

        // If multiple paths or complex paths, use API
        if fill_geometry.len() > 3 {
            return true;
        }

        // Check path complexity - if any path is very long, use API
        for path in fill_geometry {
            if path.path.len() > 200 {
                // Arbitrary threshold for "complex" path
                return true;
            }
        }

        // Simple vectors can use inline conversion
        false
    }

    /// Create inline SVG content for simple vectors
    fn create_inline_svg_content(&self, vector: &VectorNode) -> Result<String, SvgBridgeError> {
        // This is a simplified inline SVG generator
        // For now, we'll create a basic SVG structure

        let width = vector.size.as_ref().map(|s| s.x).unwrap_or(100.0);
        let height = vector.size.as_ref().map(|s| s.y).unwrap_or(100.0);

        let mut svg = format!(
            r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
            width, height, width, height
        );

        // Add paths from fill geometry
        if let Some(fill_geometry) = &vector.fill_geometry {
            for path in fill_geometry {
                svg.push_str(&format!(
                    r#"<path d="{}" fill="currentColor" />"#,
                    path.path
                ));
            }
        } else {
            // Fallback placeholder
            svg.push_str(r#"<rect width="100%" height="100%" fill="currentColor" />"#);
        }

        svg.push_str("</svg>");

        Ok(svg)
    }

    /// Try to process all vectors using batch API for better performance
    async fn try_batch_processing(
        &mut self,
        vectors: &HashMap<String, VectorNode>,
    ) -> Result<HashMap<String, String>, SvgBridgeError> {
        let node_ids: Vec<&str> = vectors.keys().map(|s| s.as_str()).collect();

        debug!("Attempting batch prefetch for {} vectors", node_ids.len());

        // Use the exporter's batch functionality
        self.exporter
            .fetch_and_cache_svg_batch(&self.file_key, &node_ids)
            .await
            .map_err(|e| SvgBridgeError::SvgExport(e))?;

        // Extract results from cache
        let mut results = HashMap::new();
        for node_id in node_ids {
            let cache_key = format!("{}_{}", self.file_key, node_id);
            if let Some(svg_result) = self.exporter.svg_cache.get(&cache_key) {
                results.insert(node_id.to_string(), svg_result.svg_content.clone());
            } else {
                return Err(SvgBridgeError::Channel(format!(
                    "Vector {} not found in cache after batch prefetch",
                    node_id
                )));
            }
        }

        Ok(results)
    }

    /// Fallback to individual processing using worker pool
    async fn fallback_to_individual_processing(
        &mut self,
        vectors: HashMap<String, VectorNode>,
        start_time: std::time::Instant,
    ) -> Result<AsyncSvgResult, SvgBridgeError> {
        info!("Using individual async processing with worker pool");

        // Create channels and worker pool
        let (request_tx, request_rx, response_tx, mut response_rx) = create_svg_channels();
        let worker_pool = SvgWorkerPool::new(
            request_rx,
            response_tx,
            self.exporter.clone(),
            self.max_concurrent,
        );

        // Spawn worker pool
        let worker_handle = tokio::spawn(async move {
            if let Err(e) = worker_pool.run().await {
                error!("SVG worker pool failed: {}", e);
            }
        });

        // Send all requests
        for (node_id, vector) in &vectors {
            let request = SvgRequest {
                node_id: node_id.clone(),
                file_key: self.file_key.clone(),
                vector_node: vector.clone(),
            };

            if let Err(e) = request_tx.send(request) {
                error!("Failed to send SVG request for {}: {}", node_id, e);
            }
        }

        // Close request channel to signal completion
        drop(request_tx);

        // Collect responses
        let (svg_content, errors) = self
            .collect_responses(&mut response_rx, vectors.len())
            .await;

        // Wait for worker pool to complete
        self.cleanup_worker_pool(worker_handle).await;

        let processing_time = start_time.elapsed().as_millis() as u64;
        let successful = svg_content.len();
        let failed = errors.len();

        info!(
            "Individual processing completed: {}/{} successful in {}ms",
            successful,
            vectors.len(),
            processing_time
        );

        Ok(AsyncSvgResult {
            svg_content,
            errors,
            stats: ProcessingStats {
                total_requested: vectors.len(),
                successful,
                failed,
                batch_prefetch_used: false,
                processing_time_ms: processing_time,
            },
        })
    }

    /// Collect responses from the worker pool with timeout
    async fn collect_responses(
        &self,
        response_rx: &mut mpsc::UnboundedReceiver<SvgResponse>,
        expected_count: usize,
    ) -> (HashMap<String, String>, HashMap<String, SvgBridgeError>) {
        let mut svg_content = HashMap::new();
        let mut errors = HashMap::new();
        let mut received_count = 0;

        while received_count < expected_count {
            let response_future = response_rx.recv();
            match timeout(self.timeout_duration, response_future).await {
                Ok(Some(response)) => {
                    match response.result {
                        Ok(svg_result) => {
                            svg_content.insert(response.node_id.clone(), svg_result.svg_content);
                            debug!("✓ Received SVG for vector {}", response.node_id);
                        }
                        Err(e) => {
                            errors.insert(response.node_id.clone(), e);
                            warn!("✗ Failed to get SVG for vector {}", response.node_id);
                        }
                    }
                    received_count += 1;
                }
                Ok(None) => {
                    warn!("Response channel closed unexpectedly");
                    break;
                }
                Err(_) => {
                    error!("Timeout waiting for SVG responses");
                    break;
                }
            }
        }

        (svg_content, errors)
    }

    /// Clean up the worker pool with timeout
    async fn cleanup_worker_pool(&self, worker_handle: tokio::task::JoinHandle<()>) {
        match timeout(self.timeout_duration, worker_handle).await {
            Ok(Ok(())) => {
                debug!("✓ Worker pool shutdown completed successfully");
            }
            Ok(Err(e)) => {
                error!("Worker pool task panicked: {}", e);
            }
            Err(_) => {
                warn!(
                    "⚠ Worker pool shutdown timed out after {} seconds",
                    self.timeout_duration.as_secs()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_vectors() {
        let mut processor = AsyncSvgProcessor::new("test_file".to_string(), SvgConfig::default());
        let vectors = HashMap::new();

        let result = processor.process_vectors(vectors).await.unwrap();

        assert_eq!(result.stats.total_requested, 0);
        assert_eq!(result.stats.successful, 0);
        assert_eq!(result.stats.failed, 0);
    }

    #[test]
    fn test_processor_configuration() {
        let processor = AsyncSvgProcessor::new("test_file".to_string(), SvgConfig::default())
            .with_concurrency(5)
            .with_timeout(Duration::from_secs(30));

        assert_eq!(processor.max_concurrent, 5);
        assert_eq!(processor.timeout_duration, Duration::from_secs(30));
    }
}
