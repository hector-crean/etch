use crate::conversion::svg::exporter::{FigmaSvgExporter, SvgExportError, SvgExportResult};
use figma_api::models::VectorNode;
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};

/// Request message for async SVG export
#[derive(Debug, Clone)]
pub struct SvgRequest {
    pub node_id: String,
    pub file_key: String,
    pub vector_node: VectorNode,
}

/// Batch request message for async SVG export
#[derive(Debug, Clone)]
pub struct SvgBatchRequest {
    pub node_ids: Vec<String>,
    pub file_key: String,
    pub vector_nodes: Vec<VectorNode>,
}

/// Response message for async SVG export
#[derive(Debug)]
pub struct SvgResponse {
    pub node_id: String,
    pub result: Result<SvgExportResult, SvgBridgeError>,
}

/// Batch response message for async SVG export
#[derive(Debug)]
pub struct SvgBatchResponse {
    pub node_ids: Vec<String>,
    pub results: Vec<Result<SvgExportResult, SvgBridgeError>>,
}

/// Error types for the async bridge
#[derive(Debug, thiserror::Error)]
pub enum SvgBridgeError {
    #[error("SVG export error: {0}")]
    SvgExport(#[from] SvgExportError),
    #[error("Channel error: {0}")]
    Channel(String),
    #[error("Timeout waiting for SVG export")]
    Timeout,
    #[error("Worker pool error: {0}")]
    WorkerPool(String),
}

/// Async worker pool for processing SVG export requests
pub struct SvgWorkerPool {
    request_rx: mpsc::UnboundedReceiver<SvgRequest>,
    response_tx: mpsc::UnboundedSender<SvgResponse>,
    exporter: FigmaSvgExporter,
    max_concurrent: usize,
}

impl SvgWorkerPool {
    /// Create a new worker pool
    pub fn new(
        request_rx: mpsc::UnboundedReceiver<SvgRequest>,
        response_tx: mpsc::UnboundedSender<SvgResponse>,
        exporter: FigmaSvgExporter,
        max_concurrent: usize,
    ) -> Self {
        Self {
            request_rx,
            response_tx,
            exporter,
            max_concurrent,
        }
    }

    /// Run the worker pool, processing requests with concurrency control
    pub async fn run(mut self) -> Result<(), SvgBridgeError> {
        use futures::stream::{FuturesUnordered, StreamExt};

        let mut active_tasks = FuturesUnordered::new();
        let mut total_requests_processed = 0u64;
        let mut pending_request: Option<SvgRequest> = None;

        log::info!(
            "SVG worker pool started with max {} concurrent tasks",
            self.max_concurrent
        );

        loop {
            tokio::select! {
                // Handle new requests (only if we don't have a pending one)
                request = self.request_rx.recv(), if pending_request.is_none() => {
                    match request {
                        Some(req) => {
                            total_requests_processed += 1;
                            log::debug!("Received SVG request {} for vector {}", total_requests_processed, req.node_id);

                            // Try to spawn immediately or store as pending
                            if active_tasks.len() < self.max_concurrent {
                                let response_tx = self.response_tx.clone();
                                let exporter = self.exporter.clone();

                                let task = tokio::spawn(async move {
                                    let node_id = req.node_id.clone();
                                    let result = Self::process_svg_request(req, exporter).await;
                                    let response = SvgResponse {
                                        node_id,
                                        result,
                                    };

                                    if let Err(e) = response_tx.send(response) {
                                        log::error!("Failed to send SVG response: {}", e);
                                    }
                                });

                                active_tasks.push(task);
                            } else {
                                // Store for later when a slot opens
                                pending_request = Some(req);
                            }
                        }
                        None => {
                            // Channel closed, finish remaining tasks
                            log::info!("SVG request channel closed, processed {} total requests, waiting for {} active tasks", total_requests_processed, active_tasks.len());
                            break;
                        }
                    }
                }

                // Handle completed tasks
                Some(result) = active_tasks.next(), if !active_tasks.is_empty() => {
                    if let Err(e) = result {
                        log::error!("SVG task failed: {}", e);
                    }

                    // If we have a pending request and now have capacity, spawn it
                    if let Some(req) = pending_request.take() {
                        let response_tx = self.response_tx.clone();
                        let exporter = self.exporter.clone();

                        let task = tokio::spawn(async move {
                            let node_id = req.node_id.clone();
                            let result = Self::process_svg_request(req, exporter).await;
                            let response = SvgResponse {
                                node_id,
                                result,
                            };

                            if let Err(e) = response_tx.send(response) {
                                log::error!("Failed to send SVG response: {}", e);
                            }
                        });

                        active_tasks.push(task);
                    }
                }
            }
        }

        // Wait for all remaining tasks to complete
        log::info!(
            "Waiting for {} remaining tasks to complete",
            active_tasks.len()
        );
        while let Some(result) = active_tasks.next().await {
            if let Err(e) = result {
                log::error!("SVG task failed during shutdown: {}", e);
            }
        }

        log::info!("SVG worker pool completed successfully");
        Ok(())
    }

    /// Process a single SVG request with timeout
    async fn process_svg_request(
        request: SvgRequest,
        mut exporter: FigmaSvgExporter,
    ) -> Result<SvgExportResult, SvgBridgeError> {
        let timeout_duration = Duration::from_secs(30);

        let result = timeout(
            timeout_duration,
            exporter.export_vector_node(&request.vector_node, &request.file_key),
        )
        .await;

        match result {
            Ok(Ok(svg_result)) => Ok(svg_result),
            Ok(Err(e)) => Err(SvgBridgeError::SvgExport(e)),
            Err(_) => Err(SvgBridgeError::Timeout),
        }
    }

    /// Process a batch SVG request with timeout
    #[allow(dead_code)]
    async fn process_svg_batch_request(
        request: SvgBatchRequest,
        exporter: FigmaSvgExporter,
    ) -> Result<Vec<Result<SvgExportResult, SvgBridgeError>>, SvgBridgeError> {
        let timeout_duration = Duration::from_secs(60); // Longer timeout for batch requests

        // Extract node IDs as string slices
        let node_ids: Vec<&str> = request.node_ids.iter().map(|s| s.as_str()).collect();

        let result = timeout(
            timeout_duration,
            exporter.fetch_figma_svg_batch(&request.file_key, &node_ids),
        )
        .await;

        match result {
            Ok(Ok(svg_contents)) => {
                // Convert SVG contents to SvgExportResult objects
                let results: Vec<Result<SvgExportResult, SvgBridgeError>> = svg_contents
                    .into_iter()
                    .map(|svg_content| {
                        Ok(SvgExportResult {
                            svg_content,
                            is_external: false,
                            external_path: None,
                            viewbox: None,
                            path_data: None,
                        })
                    })
                    .collect();
                Ok(results)
            }
            Ok(Err(e)) => {
                // Return errors for all nodes in the batch
                let error_results: Vec<Result<SvgExportResult, SvgBridgeError>> = request
                    .node_ids
                    .iter()
                    .map(|_| Err(SvgBridgeError::SvgExport(e.clone())))
                    .collect();
                Ok(error_results)
            }
            Err(_) => {
                // Return timeout errors for all nodes in the batch
                let timeout_results: Vec<Result<SvgExportResult, SvgBridgeError>> = request
                    .node_ids
                    .iter()
                    .map(|_| Err(SvgBridgeError::Timeout))
                    .collect();
                Ok(timeout_results)
            }
        }
    }
}

/// Helper function to create channels for SVG async processing
pub fn create_svg_channels() -> (
    mpsc::UnboundedSender<SvgRequest>,
    mpsc::UnboundedReceiver<SvgRequest>,
    mpsc::UnboundedSender<SvgResponse>,
    mpsc::UnboundedReceiver<SvgResponse>,
) {
    let (request_tx, request_rx) = mpsc::unbounded_channel();
    let (response_tx, response_rx) = mpsc::unbounded_channel();

    (request_tx, request_rx, response_tx, response_rx)
}

#[cfg(test)]
mod tests {
    use super::super::strategy::SvgConfig;
    use super::*;

    #[tokio::test]
    async fn test_svg_worker_pool_creation() {
        let (_request_tx, request_rx, response_tx, _response_rx) = create_svg_channels();
        let exporter = FigmaSvgExporter::new(SvgConfig::default());

        let pool = SvgWorkerPool::new(request_rx, response_tx, exporter, 5);

        // Pool should be created successfully
        assert_eq!(pool.max_concurrent, 5);
    }

    #[tokio::test]
    async fn test_channel_creation() {
        let (request_tx, _request_rx, response_tx, _response_rx) = create_svg_channels();

        // Channels should be created successfully
        assert!(!request_tx.is_closed());
        assert!(!response_tx.is_closed());
    }
}
