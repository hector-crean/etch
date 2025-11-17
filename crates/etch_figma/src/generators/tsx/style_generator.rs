//! Style and positioning strategy generation for TSX conversion
//!
//! This module handles:
//! - Positioning strategy determination (relative, absolute, flex, grid)
//! - Tailwind style coordination
//! - Layout analysis

use crate::extensions::tailwind::TailwindStyles;
use std::collections::HashMap;

/// Positioning strategy for elements based on their parent's layout mode
#[derive(Debug, Clone, PartialEq)]
pub enum PositioningStrategy {
    /// Use relative positioning (default)
    Relative,
    /// Use absolute positioning with percentage-based inset values
    Absolute,
    /// Use flexbox layout
    Flex,
    /// Use CSS Grid layout
    Grid,
}

/// Handles style generation and positioning strategy for TSX elements
pub struct StyleGenerator {
    /// Cache of positioning strategies by node ID
    positioning_cache: HashMap<String, PositioningStrategy>,
}

impl StyleGenerator {
    /// Create a new style generator
    pub fn new() -> Self {
        Self {
            positioning_cache: HashMap::new(),
        }
    }

    /// Determine the positioning strategy for a node based on its parent
    pub fn determine_positioning_strategy(
        &mut self,
        node_id: &str,
        parent_child_map: &HashMap<String, Vec<String>>,
    ) -> PositioningStrategy {
        // Check cache first
        if let Some(strategy) = self.positioning_cache.get(node_id) {
            return strategy.clone();
        }

        // Find parent
        let parent_id = parent_child_map
            .iter()
            .find(|(_, children)| children.contains(&node_id.to_string()))
            .map(|(parent, _)| parent);

        let strategy = if parent_id.is_some() {
            // If has parent, use relative positioning by default
            // In the future, we can analyze parent's layout mode to determine
            // whether to use flex, grid, or absolute positioning
            PositioningStrategy::Relative
        } else {
            // Root nodes use relative positioning
            PositioningStrategy::Relative
        };

        // Cache the result
        self.positioning_cache
            .insert(node_id.to_string(), strategy.clone());

        strategy
    }

    /// Clear the positioning cache (useful between conversions)
    pub fn clear_cache(&mut self) {
        self.positioning_cache.clear();
    }

    /// Get the cached positioning strategy for a node
    pub fn get_cached_strategy(&self, node_id: &str) -> Option<&PositioningStrategy> {
        self.positioning_cache.get(node_id)
    }
}

impl Default for StyleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_node_positioning() {
        let mut generator = StyleGenerator::new();
        let parent_map = HashMap::new();

        let strategy = generator.determine_positioning_strategy("root_node", &parent_map);
        assert_eq!(strategy, PositioningStrategy::Relative);
    }

    #[test]
    fn test_child_node_positioning() {
        let mut generator = StyleGenerator::new();
        let mut parent_map = HashMap::new();
        parent_map.insert("parent".to_string(), vec!["child".to_string()]);

        let strategy = generator.determine_positioning_strategy("child", &parent_map);
        assert_eq!(strategy, PositioningStrategy::Relative);
    }

    #[test]
    fn test_caching() {
        let mut generator = StyleGenerator::new();
        let parent_map = HashMap::new();

        // First call computes
        generator.determine_positioning_strategy("node1", &parent_map);

        // Second call should use cache
        let cached = generator.get_cached_strategy("node1");
        assert!(cached.is_some());
        assert_eq!(*cached.unwrap(), PositioningStrategy::Relative);
    }
}
