use figma_api::models::SubcanvasNode;
use figma_api::models::frame_node::{LayoutMode, LayoutPositioning};

/// Analyzes layout patterns and determines the best positioning strategy
pub struct LayoutAnalyzer;

impl LayoutAnalyzer {
    /// Analyze a group of Figma nodes and determine the best layout strategy
    /// Returns a suggested LayoutMode and LayoutPositioning based on the node arrangement
    pub fn analyze_layout(nodes: &[&SubcanvasNode]) -> LayoutAnalysis {
        if nodes.is_empty() {
            return LayoutAnalysis {
                suggested_mode: LayoutMode::None,
                suggested_positioning: LayoutPositioning::Absolute,
                confidence: 1.0,
            };
        }

        if nodes.len() == 1 {
            return LayoutAnalysis {
                suggested_mode: LayoutMode::None,
                suggested_positioning: LayoutPositioning::Absolute,
                confidence: 1.0,
            };
        }

        // For now, default to absolute positioning
        // TODO: Implement proper layout analysis when bounding box data is available
        LayoutAnalysis {
            suggested_mode: LayoutMode::None,
            suggested_positioning: LayoutPositioning::Absolute,
            confidence: 0.5, // Low confidence since we're not doing real analysis yet
        }
    }
}

/// Result of layout analysis containing suggested Figma layout properties
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutAnalysis {
    /// Suggested LayoutMode for the container
    pub suggested_mode: LayoutMode,
    /// Suggested LayoutPositioning for child elements
    pub suggested_positioning: LayoutPositioning,
    /// Confidence level (0.0 to 1.0) in the analysis
    pub confidence: f64,
}
