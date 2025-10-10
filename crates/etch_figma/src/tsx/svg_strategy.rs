// Removed unused imports - we only need the string-based node type checking

/// Determines the optimal rendering strategy for a Figma node
#[derive(Debug, Clone, PartialEq)]
pub enum RenderingStrategy {
    /// Render as HTML element (div, p, etc.)
    Html,
    /// Render as SVG element with inline content
    SvgInline,
    /// Render as SVG element with external path reference
    SvgExternal,
    /// Render as mixed content (HTML container with SVG children)
    Mixed,
}

/// Analyzes a node to determine the best rendering strategy
pub struct NodeAnalyzer;

impl NodeAnalyzer {
    /// Determines if a node should be rendered as SVG
    pub fn should_render_as_svg(node_type: &str, has_vector_content: bool, has_text_content: bool) -> bool {
        match node_type {
            // Always render as SVG
            "Vector" | "Ellipse" | "Line" | "Star" | "RegularPolygon" | "Rectangle" => true,
            
            // Render as SVG if they contain vector content
            "Group" | "Frame" | "Component" | "Instance" => has_vector_content && !has_text_content,
            
            // Never render as SVG
            "Text" | "Table" | "TableCell" | "Section" => false,
            
            // Mixed content - depends on children
            "BooleanOperation" | "ShapeWithText" | "TextPath" => has_vector_content,
            
            // Default to HTML
            _ => false,
        }
    }

    /// Determines if a node should use external SVG paths
    pub fn should_use_external_paths(_node: &str, path_complexity: PathComplexity) -> bool {
        match path_complexity {
            PathComplexity::Simple => false, // Inline for simple paths
            PathComplexity::Complex => true, // External for complex paths
            PathComplexity::VeryComplex => true, // Always external for very complex
        }
    }

    /// Analyzes a node's content to determine rendering strategy
    pub fn analyze_node(node_type: &str, has_children: bool, children_are_vectors: bool, has_text: bool) -> RenderingStrategy {
        let is_vector_node = Self::should_render_as_svg(node_type, children_are_vectors, has_text);
        
        if is_vector_node {
            if has_children && children_are_vectors {
                RenderingStrategy::SvgInline
            } else {
                RenderingStrategy::SvgExternal
            }
        } else if has_children && children_are_vectors {
            RenderingStrategy::Mixed
        } else {
            RenderingStrategy::Html
        }
    }
}

/// Represents the complexity of a vector path
#[derive(Debug, Clone, PartialEq)]
pub enum PathComplexity {
    Simple,      // Basic shapes (rect, circle, line)
    Complex,     // Custom paths with moderate complexity
    VeryComplex, // Complex artistic paths
}

impl PathComplexity {
    /// Analyzes a path string to determine complexity
    pub fn from_path_data(path_data: &str) -> Self {
        let command_count = path_data.matches(char::is_alphabetic).count();
        let length = path_data.len();
        
        match (command_count, length) {
            (count, _) if count <= 3 && length < 100 => PathComplexity::Simple,
            (count, _) if count <= 10 && length < 500 => PathComplexity::Complex,
            _ => PathComplexity::VeryComplex,
        }
    }
}

/// Configuration for SVG generation
#[derive(Debug, Clone)]
pub struct SvgConfig {
    /// Whether to use external path files
    pub use_external_paths: bool,
    /// Base path for external SVG files
    pub external_path_base: String,
    /// Whether to optimize SVG output
    pub optimize: bool,
    /// Whether to include viewBox
    pub include_viewbox: bool,
}

impl Default for SvgConfig {
    fn default() -> Self {
        Self {
            use_external_paths: true,
            external_path_base: "assets/svg".to_string(),
            optimize: true,
            include_viewbox: true,
        }
    }
}
