use super::strategy::PathComplexity;
use crate::core::config::{CodeGenConfig, VectorExportStrategy};
use figma_api::models::VectorNode;

/// Configuration for vector export strategy decisions
#[derive(Debug, Clone)]
pub struct VectorExportConfig {
    pub strategy: VectorExportStrategy,
    pub postprocess_enabled: bool,
    pub fallback_to_manual: bool,
}

impl VectorExportConfig {
    pub fn from(config: &CodeGenConfig) -> Self {
        Self {
            strategy: config.vector_export_strategy.clone(),
            postprocess_enabled: config.svg_postprocess_enabled,
            fallback_to_manual: true, // Always allow fallback for robustness
        }
    }

    pub fn default() -> Self {
        Self {
            strategy: VectorExportStrategy::Hybrid,
            postprocess_enabled: true,
            fallback_to_manual: true,
        }
    }

    /// Determines if we should use Figma API for this vector node
    pub fn should_use_api(&self, node: &VectorNode) -> bool {
        match self.strategy {
            VectorExportStrategy::ManualConversion => false,
            VectorExportStrategy::FigmaApi => {
                // Even with FigmaApi strategy, only use API for complex vectors
                // This prevents overwhelming the API with simple shapes
                self.is_complex_vector(node)
            }
            VectorExportStrategy::Hybrid => {
                // Use API for complex vectors, manual for simple ones
                self.is_complex_vector(node)
            }
        }
    }

    /// Determines if we should postprocess SVG content
    pub fn should_postprocess(&self) -> bool {
        self.postprocess_enabled
    }

    /// Determines if we should fallback to manual conversion on API failure
    pub fn should_fallback(&self) -> bool {
        self.fallback_to_manual
    }

    /// Analyzes if a vector node is complex enough to warrant API export
    fn is_complex_vector(&self, node: &VectorNode) -> bool {
        // If we have fill_geometry, check its complexity
        if let Some(fill_geometry) = &node.fill_geometry {
            if !fill_geometry.is_empty() {
                let path_data = &fill_geometry[0].path;
                let complexity = PathComplexity::from_path_data(path_data);

                match complexity {
                    PathComplexity::Simple => false,
                    PathComplexity::Complex | PathComplexity::VeryComplex => true,
                }
            } else {
                // Empty fill_geometry but still a vector - likely complex
                true
            }
        } else {
            // No fill_geometry - this could be a complex vector that needs API export
            // Many vectors in Figma don't have fill_geometry but are still complex
            // For now, let's be conservative and use API for vectors without fill_geometry
            // since we can't easily determine their complexity from the node data alone
            true
        }
    }
}

#[cfg(test)]
mod tests {}
