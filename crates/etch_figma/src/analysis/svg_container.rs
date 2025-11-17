use crate::SubcanvasNodeExt;
use crate::core::config::SvgContainerMode;
use figma_api::models::SubcanvasNode;

/// Analyzes container nodes to determine SVG wrapping strategy
pub struct SvgContainerAnalyzer;

impl SvgContainerAnalyzer {
    /// Analyze children of a node to determine container characteristics
    pub fn analyze_children(node: &SubcanvasNode) -> ContainerAnalysis {
        let mut analysis = ContainerAnalysis::default();

        if let Some(children) = node.children() {
            for child in children {
                Self::analyze_child(child, &mut analysis);
            }
        }

        // Determine if all children are vectors
        analysis.all_vectors = analysis.has_any_vector
            && analysis.vector_count > 0
            && analysis.vector_count == analysis.total_children;

        analysis
    }

    /// Analyze a single child node
    fn analyze_child(child: &SubcanvasNode, analysis: &mut ContainerAnalysis) {
        analysis.total_children += 1;

        match child {
            // Vector nodes
            SubcanvasNode::Vector(_)
            | SubcanvasNode::Ellipse(_)
            | SubcanvasNode::Line(_)
            | SubcanvasNode::Star(_)
            | SubcanvasNode::RegularPolygon(_)
            | SubcanvasNode::Rectangle(_) => {
                analysis.has_any_vector = true;
                analysis.vector_count += 1;
            }

            // Text nodes
            SubcanvasNode::Text(_)
            | SubcanvasNode::TextPath(_)
            | SubcanvasNode::ShapeWithText(_) => {
                analysis.has_text = true;
                analysis.text_count += 1;
            }

            // Container nodes - analyze recursively
            SubcanvasNode::Group(_)
            | SubcanvasNode::Frame(_)
            | SubcanvasNode::Component(_)
            | SubcanvasNode::Instance(_) => {
                if let Some(grandchildren) = child.children() {
                    if grandchildren.len() > 3 {
                        analysis.has_complex_layout = true;
                    }

                    // Recursively analyze children
                    for grandchild in grandchildren {
                        Self::analyze_child(grandchild, analysis);
                    }
                }
            }

            // Other nodes
            _ => {
                analysis.other_count += 1;
            }
        }
    }

    /// Determine if a node should be wrapped in SVG based on analysis and mode
    pub fn should_wrap_in_svg(analysis: &ContainerAnalysis, mode: SvgContainerMode) -> bool {
        match mode {
            SvgContainerMode::WrapAll => {
                // Wrap if there are any vector elements
                analysis.has_any_vector
            }
            SvgContainerMode::Mixed => {
                // Only wrap if all children are vectors (no mixed content)
                analysis.all_vectors && !analysis.has_text
            }
            SvgContainerMode::Configurable => {
                // Complex decision logic
                Self::configurable_decision(analysis)
            }
        }
    }

    /// Configurable decision logic for SVG wrapping
    fn configurable_decision(analysis: &ContainerAnalysis) -> bool {
        // If all children are vectors, definitely wrap
        if analysis.all_vectors {
            return true;
        }

        // If no vectors, don't wrap
        if !analysis.has_any_vector {
            return false;
        }

        // Mixed content - decide based on ratios
        let vector_ratio = analysis.vector_count as f64 / analysis.total_children as f64;
        let text_ratio = analysis.text_count as f64 / analysis.total_children as f64;

        // If vectors dominate (>70%), wrap in SVG
        if vector_ratio > 0.7 {
            return true;
        }

        // If text dominates (>70%), don't wrap
        if text_ratio > 0.7 {
            return false;
        }

        // If complex layout, prefer HTML for better control
        if analysis.has_complex_layout {
            return false;
        }

        // Default: wrap if more vectors than text
        analysis.vector_count > analysis.text_count
    }

    /// Get recommended SVG wrapper configuration
    pub fn get_svg_wrapper_config(
        _analysis: &ContainerAnalysis,
        responsive: bool,
    ) -> SvgWrapperConfig {
        SvgWrapperConfig {
            responsive,
            viewbox: if responsive {
                // Calculate viewBox from content bounds
                Some((0.0, 0.0, 100.0, 100.0)) // Placeholder - would calculate actual bounds
            } else {
                None
            },
            preserve_aspect_ratio: if responsive {
                "xMidYMid meet".to_string()
            } else {
                "none".to_string()
            },
        }
    }
}

/// Analysis result for container nodes
#[derive(Debug, Clone, Default)]
pub struct ContainerAnalysis {
    pub has_any_vector: bool,
    pub all_vectors: bool,
    pub has_text: bool,
    pub has_complex_layout: bool,
    pub vector_count: usize,
    pub text_count: usize,
    pub other_count: usize,
    pub total_children: usize,
}

/// Configuration for SVG wrapper generation
#[derive(Debug, Clone)]
pub struct SvgWrapperConfig {
    pub responsive: bool,
    pub viewbox: Option<(f64, f64, f64, f64)>,
    pub preserve_aspect_ratio: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use figma_api::models::{GroupNode, TextNode, VectorNode};

    #[test]
    fn test_analyze_all_vectors() {
        // Create a mock node with vector children
        let vector1 = SubcanvasNode::Vector(Box::new(VectorNode {
            id: "v1".to_string(),
            name: "Vector 1".to_string(),
            ..Default::default()
        }));

        let vector2 = SubcanvasNode::Vector(Box::new(VectorNode {
            id: "v2".to_string(),
            name: "Vector 2".to_string(),
            ..Default::default()
        }));

        let group = SubcanvasNode::Group(Box::new(GroupNode {
            id: "group".to_string(),
            name: "Group".to_string(),
            children: vec![vector1, vector2],
            ..Default::default()
        }));

        let analysis = SvgContainerAnalyzer::analyze_children(&group);

        assert!(analysis.has_any_vector);
        assert!(analysis.all_vectors);
        assert!(!analysis.has_text);
        assert_eq!(analysis.vector_count, 2);
        assert_eq!(analysis.total_children, 2);
    }

    #[test]
    fn test_analyze_mixed_content() {
        let vector = SubcanvasNode::Vector(Box::new(VectorNode {
            id: "v1".to_string(),
            name: "Vector".to_string(),
            ..Default::default()
        }));

        let text = SubcanvasNode::Text(Box::new(TextNode {
            id: "t1".to_string(),
            name: "Text".to_string(),
            characters: "Hello".to_string(),
            ..Default::default()
        }));

        let group = SubcanvasNode::Group(Box::new(GroupNode {
            id: "group".to_string(),
            name: "Group".to_string(),
            children: vec![vector, text],
            ..Default::default()
        }));

        let analysis = SvgContainerAnalyzer::analyze_children(&group);

        assert!(analysis.has_any_vector);
        assert!(!analysis.all_vectors);
        assert!(analysis.has_text);
        assert_eq!(analysis.vector_count, 1);
        assert_eq!(analysis.text_count, 1);
        assert_eq!(analysis.total_children, 2);
    }

    #[test]
    fn test_should_wrap_wrap_all() {
        let analysis = ContainerAnalysis {
            has_any_vector: true,
            all_vectors: false,
            has_text: true,
            ..Default::default()
        };

        assert!(SvgContainerAnalyzer::should_wrap_in_svg(
            &analysis,
            SvgContainerMode::WrapAll
        ));
    }

    #[test]
    fn test_should_wrap_mixed() {
        let analysis = ContainerAnalysis {
            has_any_vector: true,
            all_vectors: true,
            has_text: false,
            ..Default::default()
        };

        assert!(SvgContainerAnalyzer::should_wrap_in_svg(
            &analysis,
            SvgContainerMode::Mixed
        ));

        let mixed_analysis = ContainerAnalysis {
            has_any_vector: true,
            all_vectors: false,
            has_text: true,
            ..Default::default()
        };

        assert!(!SvgContainerAnalyzer::should_wrap_in_svg(
            &mixed_analysis,
            SvgContainerMode::Mixed
        ));
    }

    #[test]
    fn test_configurable_decision() {
        // All vectors
        let all_vectors = ContainerAnalysis {
            has_any_vector: true,
            all_vectors: true,
            vector_count: 3,
            total_children: 3,
            ..Default::default()
        };
        assert!(SvgContainerAnalyzer::should_wrap_in_svg(
            &all_vectors,
            SvgContainerMode::Configurable
        ));

        // No vectors
        let no_vectors = ContainerAnalysis {
            has_any_vector: false,
            total_children: 2,
            ..Default::default()
        };
        assert!(!SvgContainerAnalyzer::should_wrap_in_svg(
            &no_vectors,
            SvgContainerMode::Configurable
        ));

        // Vector dominated
        let vector_dominated = ContainerAnalysis {
            has_any_vector: true,
            vector_count: 7,
            text_count: 3,
            total_children: 10,
            ..Default::default()
        };
        assert!(SvgContainerAnalyzer::should_wrap_in_svg(
            &vector_dominated,
            SvgContainerMode::Configurable
        ));
    }
}
