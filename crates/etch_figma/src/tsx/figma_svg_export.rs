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
    async fn fetch_figma_svg(&self, _file_key: &str, _node_id: &str) -> Result<String, SvgExportError> {
        // This would integrate with the Figma API to fetch SVG export
        // For now, we'll simulate this with a placeholder
        
        // In a real implementation, this would make an HTTP request to:
        // https://api.figma.com/v1/images/{file_key}?ids={node_id}&format=svg
        
        // Placeholder implementation
        Ok(format!("<svg viewBox=\"0 0 100 100\"><path d=\"M10,10 L90,90\"/></svg>"))
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
        let filename = format!("{}.svg", node_id);
        let file_path = format!("{}/{}", self.config.external_path_base, filename);
        
        // In a real implementation, this would write to the filesystem
        // For now, we'll just track the path
        
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
