use std::collections::HashMap;
use figma_api::models::{VectorNode, FrameNode};
use crate::tsx::svg_strategy::{SvgConfig, PathComplexity};

/// Service for exporting SVG content from Figma nodes
pub struct FigmaSvgExporter {
    config: SvgConfig,
    /// Cache for exported SVG content
    svg_cache: HashMap<String, SvgExportResult>,
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
    pub async fn export_vector_node(&mut self, node: &VectorNode, file_key: &str) -> Result<SvgExportResult, SvgExportError> {
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

    /// Exports a frame/group that contains vector content
    pub async fn export_vector_container(&mut self, node: &FrameNode, file_key: &str) -> Result<SvgExportResult, SvgExportError> {
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

    /// Fetches SVG content from Figma API
    async fn fetch_figma_svg(&self, file_key: &str, node_id: &str) -> Result<String, SvgExportError> {
        // Use Figma's image export API
        // https://api.figma.com/v1/images/{file_key}?ids={node_id}&format=svg
        
        let url = format!(
            "https://api.figma.com/v1/images/{}?ids={}&format=svg",
            file_key, node_id
        );
        
        // Get Figma API token from environment
        let token = std::env::var("X_FIGMA_TOKEN")
            .map_err(|_| SvgExportError::ApiError("X_FIGMA_TOKEN not set".to_string()))?;
        
        // Make the API request to get the SVG URL
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("X-Figma-Token", token.clone())
            .send()
            .await
            .map_err(|e| SvgExportError::ApiError(format!("Failed to fetch SVG URL: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(SvgExportError::ApiError(format!(
                "Figma API error: {}",
                response.status()
            )));
        }
        
        // Parse the response to get the SVG URL
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SvgExportError::ApiError(format!("Failed to parse response: {}", e)))?;
        
        let svg_url = json["images"][node_id]
            .as_str()
            .ok_or_else(|| SvgExportError::ApiError("No SVG URL in response".to_string()))?;
        
        // Fetch the actual SVG content from the returned URL
        let svg_response = client
            .get(svg_url)
            .send()
            .await
            .map_err(|e| SvgExportError::ApiError(format!("Failed to fetch SVG: {}", e)))?;
        
        let svg_content = svg_response
            .text()
            .await
            .map_err(|e| SvgExportError::ApiError(format!("Failed to read SVG: {}", e)))?;
        
        Ok(svg_content)
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
    async fn export_to_external_file(&mut self, svg_content: &str, node_id: &str) -> Result<SvgExportResult, SvgExportError> {
        use std::fs;
        use std::path::Path;
        
        let filename = format!("{}.svg", node_id);
        let file_path = format!("{}/{}", self.config.external_path_base, filename);
        
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&file_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| SvgExportError::FileError(format!("Failed to create directory: {}", e)))?;
        }
        
        // Write SVG content to file
        fs::write(&file_path, svg_content)
            .map_err(|e| SvgExportError::FileError(format!("Failed to write SVG file: {}", e)))?;
        
        self.external_paths.insert(node_id.to_string(), file_path.clone());
        
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
                let coords: Vec<f64> = viewbox_str.split_whitespace()
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

#[derive(Debug, thiserror::Error)]
pub enum SvgExportError {
    #[error("Failed to fetch SVG from Figma API: {0}")]
    ApiError(String),
    #[error("Failed to write external file: {0}")]
    FileError(String),
    #[error("Invalid SVG content: {0}")]
    InvalidSvg(String),
}
