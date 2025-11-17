use super::export_config::VectorExportConfig;
use super::postprocessor::SvgPostprocessor;
use super::strategy::{PathComplexity, SvgConfig};
use figma_api::models::{FrameNode, VectorNode};
use std::collections::HashMap;

/// Service for exporting SVG content from Figma nodes
#[derive(Clone)]
pub struct FigmaSvgExporter {
    config: SvgConfig,
    /// Cache for exported SVG content
    pub svg_cache: HashMap<String, SvgExportResult>,
    /// External path mappings
    external_paths: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SvgExportResult {
    /// The SVG content
    pub svg_content: String,
    /// Whether this was exported as external file
    pub is_external: bool,
    /// External file path (if applicable)
    pub external_path: Option<String>,
    /// ViewBox dimensions
    pub viewbox: Option<(f64, f64, f64, f64)>,
    /// Optimized path data
    pub path_data: Option<String>,
}

impl FigmaSvgExporter {
    pub fn new(config: SvgConfig) -> Self {
        Self {
            config,
            svg_cache: HashMap::new(),
            external_paths: HashMap::new(),
        }
    }

    /// Exports a vector node as SVG using Figma's native export
    pub async fn export_vector_node(
        &mut self,
        node: &VectorNode,
        file_key: &str,
    ) -> Result<SvgExportResult, SvgExportError> {
        let cache_key = format!("{}_{}", file_key, node.id);

        if let Some(cached) = self.svg_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Use Figma's native SVG export API
        let svg_content = self.fetch_figma_svg(file_key, &node.id).await?;

        // Analyze path complexity
        let path_complexity = self.analyze_svg_complexity(&svg_content);

        let result = if self.should_use_external_paths(&path_complexity) {
            self.export_to_external_file(&svg_content, &node.id).await?
        } else {
            let viewbox = self.extract_viewbox(&svg_content);
            let path_data = self.extract_path_data(&svg_content);
            SvgExportResult {
                svg_content: svg_content.clone(),
                is_external: false,
                external_path: None,
                viewbox,
                path_data,
            }
        };

        self.svg_cache.insert(cache_key, result.clone());
        Ok(result)
    }

    /// Fetch and cache SVG data for a vector node by ID
    pub async fn fetch_and_cache_svg(
        &mut self,
        file_key: &str,
        node_id: &str,
    ) -> Result<(), SvgExportError> {
        let cache_key = format!("{}_{}", file_key, node_id);

        // Check if already cached
        if self.svg_cache.contains_key(&cache_key) {
            return Ok(());
        }

        // Fetch SVG content from Figma API
        let svg_content = self.fetch_figma_svg(file_key, node_id).await?;

        // Create a basic SvgExportResult for caching
        let result = SvgExportResult {
            svg_content,
            is_external: false,
            external_path: None,
            viewbox: None, // Will be extracted later if needed
            path_data: None,
        };

        // Cache the result
        self.svg_cache.insert(cache_key, result);

        Ok(())
    }

    /// Batch fetch and cache SVG data for multiple vector nodes
    pub async fn fetch_and_cache_svg_batch(
        &mut self,
        file_key: &str,
        node_ids: &[&str],
    ) -> Result<(), SvgExportError> {
        if node_ids.is_empty() {
            return Ok(());
        }

        // Filter out already cached nodes
        let uncached_ids: Vec<&str> = node_ids
            .iter()
            .filter(|node_id| {
                let cache_key = format!("{}_{}", file_key, node_id);
                !self.svg_cache.contains_key(&cache_key)
            })
            .copied()
            .collect();

        if uncached_ids.is_empty() {
            log::info!("All {} nodes already cached", node_ids.len());
            return Ok(());
        }

        log::info!(
            "Batch fetching {} uncached SVGs ({} total requested)",
            uncached_ids.len(),
            node_ids.len()
        );

        // Fetch SVG contents in batch
        let svg_contents = self.fetch_figma_svg_batch(file_key, &uncached_ids).await?;

        // Cache the results (only cache successful ones)
        for (node_id, svg_content) in uncached_ids.iter().zip(svg_contents.iter()) {
            if !svg_content.is_empty() {
                let cache_key = format!("{}_{}", file_key, node_id);
                let result = SvgExportResult {
                    svg_content: svg_content.clone(),
                    is_external: false,
                    external_path: None,
                    viewbox: None, // Will be extracted later if needed
                    path_data: None,
                };
                self.svg_cache.insert(cache_key, result);
            }
        }

        log::info!("Successfully cached {} SVGs", uncached_ids.len());
        Ok(())
    }

    /// Exports a vector node with postprocessing capabilities
    pub async fn export_with_postprocess(
        &mut self,
        node: &VectorNode,
        file_key: &str,
        config: &VectorExportConfig,
    ) -> Result<SvgExportResult, SvgExportError> {
        let cache_key = format!("{}_{}_postprocessed", file_key, node.id);

        if let Some(cached) = self.svg_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Get base SVG export
        let mut result = self.export_vector_node(node, file_key).await?;

        // Apply postprocessing if enabled
        if config.should_postprocess() {
            result = self.postprocess_svg(&result, node)?;
        }

        self.svg_cache.insert(cache_key, result.clone());
        Ok(result)
    }

    /// Postprocess SVG content to fix known issues
    fn postprocess_svg(
        &self,
        result: &SvgExportResult,
        _node: &VectorNode,
    ) -> Result<SvgExportResult, SvgExportError> {
        let processed_content = SvgPostprocessor::postprocess(&result.svg_content);

        Ok(SvgExportResult {
            svg_content: processed_content,
            is_external: result.is_external,
            external_path: result.external_path.clone(),
            viewbox: result.viewbox,
            path_data: result.path_data.clone(),
        })
    }

    /// Exports a frame/group that contains vector content
    pub async fn export_vector_container(
        &mut self,
        node: &FrameNode,
        file_key: &str,
    ) -> Result<SvgExportResult, SvgExportError> {
        // For containers, we need to determine if all children are vectors
        // This would require analyzing the node's children
        // For now, we'll use a simplified approach

        let cache_key = format!("{}_{}", file_key, node.id);

        if let Some(cached) = self.svg_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Export the entire frame as SVG if it contains vector content
        let svg_content = self.fetch_figma_svg(file_key, &node.id).await?;

        let viewbox = self.extract_viewbox(&svg_content);

        let result = SvgExportResult {
            svg_content: svg_content,
            is_external: false,
            external_path: None,
            viewbox,
            path_data: None,
        };

        self.svg_cache.insert(cache_key, result.clone());
        Ok(result)
    }

    /// Fetches SVG content from Figma API (single node)
    async fn fetch_figma_svg(
        &self,
        file_key: &str,
        node_id: &str,
    ) -> Result<String, SvgExportError> {
        let results = self.fetch_figma_svg_batch(file_key, &[node_id]).await?;
        results.into_iter().next().ok_or_else(|| {
            SvgExportError::ApiError("No result returned for single node".to_string())
        })
    }

    /// Fetches SVG content from Figma API for multiple nodes in a single batch request
    pub async fn fetch_figma_svg_batch(
        &self,
        file_key: &str,
        node_ids: &[&str],
    ) -> Result<Vec<String>, SvgExportError> {
        if node_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Figma API has limits on batch size, so we need to chunk large requests
        let max_batch_size = self.config.max_batch_size;

        if node_ids.len() <= max_batch_size {
            return self.fetch_figma_svg_batch_chunk(file_key, node_ids).await;
        }

        log::info!(
            "Splitting {} SVGs into chunks of {} for batch fetching",
            node_ids.len(),
            max_batch_size
        );

        // Split into chunks and process in parallel
        let mut all_results = Vec::new();
        let chunks: Vec<&[&str]> = node_ids.chunks(max_batch_size).collect();

        // Process chunks in parallel
        let chunk_futures: Vec<_> = chunks
            .into_iter()
            .map(|chunk| self.fetch_figma_svg_batch_chunk(file_key, chunk))
            .collect();

        let chunk_results = futures::future::join_all(chunk_futures).await;

        // Combine results in original order
        for result in chunk_results {
            match result {
                Ok(mut chunk_svgs) => {
                    all_results.append(&mut chunk_svgs);
                }
                Err(e) => {
                    log::error!("Failed to fetch chunk: {}", e);
                    return Err(e);
                }
            }
        }

        log::info!(
            "Successfully fetched {} SVGs in {} chunks",
            all_results.len(),
            (node_ids.len() + max_batch_size - 1) / max_batch_size
        );
        Ok(all_results)
    }

    /// Fetches SVG content from Figma API for a single chunk of nodes
    async fn fetch_figma_svg_batch_chunk(
        &self,
        file_key: &str,
        node_ids: &[&str],
    ) -> Result<Vec<String>, SvgExportError> {
        if node_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Use Figma's image export API with batching
        // https://api.figma.com/v1/images/{file_key}?ids={node_id1},{node_id2}&format=svg
        // URL encode node IDs since they may contain special characters like colons and semicolons
        let encoded_ids: Vec<String> = node_ids
            .iter()
            .map(|id| urlencoding::encode(id).to_string())
            .collect();
        let ids_param = encoded_ids.join(",");
        let url = format!(
            "https://api.figma.com/v1/images/{}?ids={}&format=svg",
            file_key, ids_param
        );

        log::info!(
            "Fetching {} SVGs in batch chunk from Figma API",
            node_ids.len()
        );
        log::debug!("Node IDs being requested: {:?}", node_ids);
        log::debug!("Encoded IDs param: {}", &ids_param);

        // Get Figma API token from environment
        let token = std::env::var("X_FIGMA_TOKEN")
            .map_err(|_| SvgExportError::ApiError("X_FIGMA_TOKEN not set".to_string()))?;

        // Make the API request to get the SVG URLs
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("X-Figma-Token", token.clone())
            .send()
            .await
            .map_err(|e| SvgExportError::ApiError(format!("Failed to fetch SVG URLs: {}", e)))?;

        if !response.status().is_success() {
            return Err(SvgExportError::ApiError(format!(
                "Figma API error: {}",
                response.status()
            )));
        }

        // Parse the response to get the SVG URLs
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SvgExportError::ApiError(format!("Failed to parse response: {}", e)))?;

        let images = json["images"]
            .as_object()
            .ok_or_else(|| SvgExportError::ApiError("Invalid response format".to_string()))?;

        log::debug!("Figma API returned {} image URLs", images.len());
        log::debug!(
            "Response keys: {:?}",
            images.keys().take(10).collect::<Vec<_>>()
        );

        // Check if there's an error field in the response
        if let Some(err) = json["err"].as_str() {
            log::error!("Figma API returned error: {}", err);
        }

        // Collect SVG URLs and fetch them in parallel
        let mut svg_futures = Vec::new();
        let mut node_id_to_url = Vec::new();

        for node_id in node_ids {
            // Check what Figma actually returned for this node
            match images.get(*node_id) {
                Some(serde_json::Value::String(svg_url)) if !svg_url.is_empty() => {
                    log::debug!("Found SVG URL for node {}: {}", node_id, svg_url);
                    node_id_to_url.push((*node_id, svg_url.to_string()));
                }
                Some(serde_json::Value::String(svg_url)) if svg_url.is_empty() => {
                    log::warn!("Empty SVG URL returned for node {}", node_id);
                }
                Some(serde_json::Value::Null) => {
                    log::warn!(
                        "Null SVG URL returned for node {} - node may not be exportable (instance/component/etc)",
                        node_id
                    );
                }
                Some(other) => {
                    log::warn!("Unexpected value type for node {}: {:?}", node_id, other);
                }
                None => {
                    log::warn!("No entry for node {} in API response", node_id);
                    if node_ids.len() <= 5 {
                        // Only log all keys for small batches to avoid spam
                        log::debug!("Available keys: {:?}", images.keys().collect::<Vec<_>>());
                    }
                }
            }
        }

        // Fetch all SVG contents in parallel
        for (node_id, svg_url) in node_id_to_url {
            let client_clone = client.clone();
            let future = async move {
                let svg_response = client_clone.get(&svg_url).send().await.map_err(|e| {
                    SvgExportError::ApiError(format!("Failed to fetch SVG for {}: {}", node_id, e))
                })?;

                let svg_content = svg_response.text().await.map_err(|e| {
                    SvgExportError::ApiError(format!("Failed to read SVG for {}: {}", node_id, e))
                })?;

                Ok::<(String, String), SvgExportError>((node_id.to_string(), svg_content))
            };
            svg_futures.push(future);
        }

        // Wait for all SVG fetches to complete
        let results = futures::future::join_all(svg_futures).await;

        // Sort results by original node_id order and collect SVG contents
        let mut svg_contents = Vec::new();
        let mut failed_nodes = Vec::new();

        for node_id in node_ids {
            if let Some(Ok((_, content))) = results.iter().find(|r| {
                if let Ok((id, _)) = r {
                    id == *node_id
                } else {
                    false
                }
            }) {
                svg_contents.push(content.clone());
            } else {
                log::warn!("Failed to fetch SVG for node {} in batch chunk", node_id);
                failed_nodes.push(*node_id);
                // Add empty string as placeholder to maintain order
                svg_contents.push(String::new());
            }
        }

        // If we have some failures, log them but don't fail the entire batch
        if !failed_nodes.is_empty() {
            log::warn!(
                "Failed to fetch {} out of {} SVGs in batch chunk: {:?}",
                failed_nodes.len(),
                node_ids.len(),
                failed_nodes
            );
        }

        log::info!(
            "Successfully fetched {} SVGs in batch chunk",
            svg_contents.len()
        );
        Ok(svg_contents)
    }

    /// Analyzes SVG content to determine complexity
    fn analyze_svg_complexity(&self, svg_content: &str) -> PathComplexity {
        // Count path elements and analyze complexity
        let path_count = svg_content.matches("<path").count();
        let total_length = svg_content.len();

        match (path_count, total_length) {
            (count, _) if count <= 1 && total_length < 200 => PathComplexity::Simple,
            (count, _) if count <= 5 && total_length < 1000 => PathComplexity::Complex,
            _ => PathComplexity::VeryComplex,
        }
    }

    /// Determines if external paths should be used
    fn should_use_external_paths(&self, complexity: &PathComplexity) -> bool {
        match complexity {
            PathComplexity::Simple => false,
            PathComplexity::Complex => self.config.use_external_paths,
            PathComplexity::VeryComplex => true,
        }
    }

    /// Exports SVG to external file
    async fn export_to_external_file(
        &mut self,
        svg_content: &str,
        node_id: &str,
    ) -> Result<SvgExportResult, SvgExportError> {
        use std::fs;
        use std::path::Path;

        // Sanitize filename by replacing invalid characters
        let sanitized_id = node_id
            .chars()
            .map(|c| match c {
                ':' | ';' | '/' | '\\' | '<' | '>' | '"' | '|' | '?' | '*' => '_',
                _ => c,
            })
            .collect::<String>();

        let filename = format!("{}.svg", sanitized_id);
        let file_path = format!("{}/{}", self.config.external_path_base, filename);

        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&file_path).parent() {
            fs::create_dir_all(parent).map_err(|e| {
                SvgExportError::FileError(format!("Failed to create directory: {}", e))
            })?;
        }

        // Write SVG content to file
        fs::write(&file_path, svg_content)
            .map_err(|e| SvgExportError::FileError(format!("Failed to write SVG file: {}", e)))?;

        self.external_paths
            .insert(node_id.to_string(), file_path.clone());

        Ok(SvgExportResult {
            svg_content: svg_content.to_string(),
            is_external: true,
            external_path: Some(file_path),
            viewbox: self.extract_viewbox(svg_content),
            path_data: self.extract_path_data(svg_content),
        })
    }

    /// Extracts viewBox from SVG content
    fn extract_viewbox(&self, svg_content: &str) -> Option<(f64, f64, f64, f64)> {
        // Parse viewBox from SVG content
        // This is a simplified implementation
        if let Some(start) = svg_content.find("viewBox=\"") {
            if let Some(end) = svg_content[start + 9..].find("\"") {
                let viewbox_str = &svg_content[start + 9..start + 9 + end];
                let coords: Vec<f64> = viewbox_str
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if coords.len() == 4 {
                    return Some((coords[0], coords[1], coords[2], coords[3]));
                }
            }
        }
        None
    }

    /// Extracts path data from SVG content
    fn extract_path_data(&self, svg_content: &str) -> Option<String> {
        if let Some(start) = svg_content.find("d=\"") {
            if let Some(end) = svg_content[start + 3..].find("\"") {
                return Some(svg_content[start + 3..start + 3 + end].to_string());
            }
        }
        None
    }

    /// Gets external path for a node
    pub fn get_external_path(&self, node_id: &str) -> Option<&String> {
        self.external_paths.get(node_id)
    }

    /// Generates TypeScript path mapping file
    pub fn generate_path_mapping(&self) -> String {
        let mut mapping = String::from("export default {\n");

        for (node_id, path) in &self.external_paths {
            mapping.push_str(&format!("  {}: \"{}\",\n", node_id, path));
        }

        mapping.push_str("};\n");
        mapping
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum SvgExportError {
    #[error("Failed to fetch SVG from Figma API: {0}")]
    ApiError(String),
    #[error("Failed to write external file: {0}")]
    FileError(String),
    #[error("Invalid SVG content: {0}")]
    InvalidSvg(String),
}
