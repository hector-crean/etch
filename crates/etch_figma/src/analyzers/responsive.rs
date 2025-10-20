use crate::codegen_ext::CodeGenConfig;
use figma_api::models::{FrameNode, frame_node::LayoutMode};

/// Analyzes frames for responsive behavior opportunities
pub struct ResponsiveAnalyzer;

impl ResponsiveAnalyzer {
    /// Determine if a frame should have container queries added
    pub fn should_add_container_query(frame: &FrameNode, config: &CodeGenConfig) -> bool {
        if !config.enable_container_queries {
            return false;
        }

        // Check if frame has auto-layout
        if frame.layout_mode.is_none() {
            return false;
        }

        // Check if frame has multiple children that would benefit from responsive behavior
        if !frame.children.is_empty() {
            let children = &frame.children;
            if children.len() < 2 {
                return false;
            }

            // Check if children have different sizes that would cause wrapping issues
            Self::has_varying_child_sizes(frame)
        } else {
            false
        }
    }

    /// Calculate the appropriate breakpoint for a frame
    pub fn calculate_breakpoint(frame: &FrameNode, config: &CodeGenConfig) -> Option<f64> {
        if !Self::should_add_container_query(frame, config) {
            return None;
        }

        // Use config default or calculate based on content
        let base_breakpoint = config.responsive_breakpoint_px;

        // Adjust based on frame properties
        if let Some(size) = &frame.size {
            // If frame is naturally narrow, use a smaller breakpoint
            if size.x < 400.0 {
                Some(base_breakpoint * 0.75)
            } else if size.x > 800.0 {
                // If frame is naturally wide, use a larger breakpoint
                Some(base_breakpoint * 1.25)
            } else {
                Some(base_breakpoint)
            }
        } else {
            Some(base_breakpoint)
        }
    }

    /// Get responsive classes for a frame
    pub fn get_responsive_classes(frame: &FrameNode, config: &CodeGenConfig) -> Vec<String> {
        let mut classes = Vec::new();

        if !Self::should_add_container_query(frame, config) {
            return classes;
        }

        let breakpoint =
            Self::calculate_breakpoint(frame, config).unwrap_or(config.responsive_breakpoint_px);

        // Add container query marker
        classes.push("@container".to_string());

        // Add responsive behavior based on layout mode
        if let Some(layout_mode) = &frame.layout_mode {
            match layout_mode {
                LayoutMode::Horizontal => {
                    // Switch to column when narrow
                    classes.push(format!("@[<{}px]:flex-col", breakpoint));
                    classes.push(format!("@[<{}px]:items-stretch", breakpoint));

                    // Adjust gap for narrow containers
                    if let Some(gap) = frame.item_spacing {
                        if gap > 16.0 {
                            classes.push(format!("@[<{}px]:gap-[{}px]", breakpoint, gap / 2.0));
                        }
                    }
                }
                LayoutMode::Vertical => {
                    // Could switch to row when very wide
                    let wide_breakpoint = breakpoint * 2.0;
                    classes.push(format!("@[>{}px]:flex-row", wide_breakpoint));
                    classes.push(format!("@[>{}px]:items-center", wide_breakpoint));
                }
                LayoutMode::None | LayoutMode::Grid => {
                    // No special responsive behavior for these modes
                }
            }
        }

        // Add responsive padding adjustments
        Self::add_responsive_padding_classes(frame, &mut classes, breakpoint);

        // Add responsive sizing adjustments
        Self::add_responsive_sizing_classes(frame, &mut classes, breakpoint);

        classes
    }

    /// Check if frame has children with varying sizes that would benefit from responsive behavior
    fn has_varying_child_sizes(frame: &FrameNode) -> bool {
        if !frame.children.is_empty() {
            let children = &frame.children;
            if children.len() < 2 {
                return false;
            }

            // Check if children have different widths (for horizontal layout)
            if let Some(LayoutMode::Horizontal) = frame.layout_mode {
                let mut widths = Vec::new();
                for child in children {
                    if let Some(size) = Self::get_child_size(child) {
                        widths.push(size.x);
                    }
                }

                if widths.len() >= 2 {
                    let min_width = widths.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max_width = widths.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

                    // If there's significant variation in widths, responsive behavior would help
                    return (max_width - min_width) > 50.0;
                }
            }

            // Check if children have different heights (for vertical layout)
            if let Some(LayoutMode::Vertical) = frame.layout_mode {
                let mut heights = Vec::new();
                for child in children {
                    if let Some(size) = Self::get_child_size(child) {
                        heights.push(size.y);
                    }
                }

                if heights.len() >= 2 {
                    let min_height = heights.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max_height = heights.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

                    return (max_height - min_height) > 30.0;
                }
            }
        }

        false
    }

    /// Get the size of a child node
    fn get_child_size(
        child: &figma_api::models::SubcanvasNode,
    ) -> Option<figma_api::models::Vector> {
        match child {
            figma_api::models::SubcanvasNode::Frame(n) => n.size.as_ref().map(|v| (**v).clone()),
            figma_api::models::SubcanvasNode::Group(n) => n.size.as_ref().map(|v| (**v).clone()),
            figma_api::models::SubcanvasNode::Vector(n) => n.size.as_ref().map(|v| (**v).clone()),
            figma_api::models::SubcanvasNode::Rectangle(n) => {
                n.size.as_ref().map(|v| (**v).clone())
            }
            figma_api::models::SubcanvasNode::Ellipse(n) => n.size.as_ref().map(|v| (**v).clone()),
            figma_api::models::SubcanvasNode::Text(n) => n.size.as_ref().map(|v| (**v).clone()),
            _ => None,
        }
    }

    /// Add responsive padding classes
    fn add_responsive_padding_classes(
        frame: &FrameNode,
        classes: &mut Vec<String>,
        breakpoint: f64,
    ) {
        let pt = frame.padding_top.unwrap_or(0.0);
        let pr = frame.padding_right.unwrap_or(0.0);
        let pb = frame.padding_bottom.unwrap_or(0.0);
        let pl = frame.padding_left.unwrap_or(0.0);

        // If padding is significant, reduce it on narrow screens
        if pt > 20.0 || pr > 20.0 || pb > 20.0 || pl > 20.0 {
            if pt > 20.0 {
                classes.push(format!("@[<{}px]:pt-[{}px]", breakpoint, pt / 2.0));
            }
            if pr > 20.0 {
                classes.push(format!("@[<{}px]:pr-[{}px]", breakpoint, pr / 2.0));
            }
            if pb > 20.0 {
                classes.push(format!("@[<{}px]:pb-[{}px]", breakpoint, pb / 2.0));
            }
            if pl > 20.0 {
                classes.push(format!("@[<{}px]:pl-[{}px]", breakpoint, pl / 2.0));
            }
        }
    }

    /// Add responsive sizing classes
    fn add_responsive_sizing_classes(
        frame: &FrameNode,
        classes: &mut Vec<String>,
        breakpoint: f64,
    ) {
        // If frame has fixed width, make it responsive on narrow screens
        if let Some(h_sizing) = &frame.layout_sizing_horizontal {
            match h_sizing {
                figma_api::models::frame_node::LayoutSizingHorizontal::Fixed => {
                    if let Some(size) = &frame.size {
                        if size.x > breakpoint {
                            classes.push(format!("@[<{}px]:w-full", breakpoint));
                        }
                    }
                }
                _ => {}
            }
        }

        // If frame has fixed height, allow it to grow on narrow screens
        if let Some(v_sizing) = &frame.layout_sizing_vertical {
            match v_sizing {
                figma_api::models::frame_node::LayoutSizingVertical::Fixed => {
                    classes.push(format!("@[<{}px]:h-auto", breakpoint));
                }
                _ => {}
            }
        }
    }

    /// Analyze if a frame would benefit from responsive text sizing
    pub fn should_add_responsive_text(frame: &FrameNode, config: &CodeGenConfig) -> bool {
        if !config.enable_container_queries {
            return false;
        }

        // Check if frame contains text that might be too large on narrow screens
        if !frame.children.is_empty() {
            let children = &frame.children;
            for child in children {
                if Self::has_large_text(child) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a node contains large text
    fn has_large_text(node: &figma_api::models::SubcanvasNode) -> bool {
        match node {
            figma_api::models::SubcanvasNode::Text(text_node) => {
                if let Some(font_size) = text_node.style.font_size {
                    font_size > 24.0 // Consider text > 24px as "large"
                } else {
                    false
                }
            }
            figma_api::models::SubcanvasNode::Frame(frame_node) => {
                if !frame_node.children.is_empty() {
                    let children = &frame_node.children;
                    for child in children {
                        if Self::has_large_text(child) {
                            return true;
                        }
                    }
                }
                false
            }
            figma_api::models::SubcanvasNode::Group(group_node) => {
                if !group_node.children.is_empty() {
                    let children = &group_node.children;
                    for child in children {
                        if Self::has_large_text(child) {
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use figma_api::models::{Vector, frame_node::LayoutMode};

    fn create_test_frame() -> FrameNode {
        FrameNode {
            id: "test".to_string(),
            name: "Test Frame".to_string(),
            layout_mode: Some(LayoutMode::Horizontal),
            size: Some(Box::new(Vector { x: 600.0, y: 200.0 })),
            children: vec![
                figma_api::models::SubcanvasNode::Frame(Box::new(FrameNode {
                    id: "child1".to_string(),
                    name: "Child 1".to_string(),
                    size: Some(Box::new(Vector { x: 200.0, y: 100.0 })),
                    ..Default::default()
                })),
                figma_api::models::SubcanvasNode::Frame(Box::new(FrameNode {
                    id: "child2".to_string(),
                    name: "Child 2".to_string(),
                    size: Some(Box::new(Vector { x: 300.0, y: 100.0 })),
                    ..Default::default()
                })),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_should_add_container_query() {
        let frame = create_test_frame();
        let config = CodeGenConfig {
            enable_container_queries: true,
            ..Default::default()
        };

        assert!(ResponsiveAnalyzer::should_add_container_query(
            &frame, &config
        ));
    }

    #[test]
    fn test_should_not_add_container_query_disabled() {
        let frame = create_test_frame();
        let config = CodeGenConfig {
            enable_container_queries: false,
            ..Default::default()
        };

        assert!(!ResponsiveAnalyzer::should_add_container_query(
            &frame, &config
        ));
    }

    #[test]
    fn test_calculate_breakpoint() {
        let frame = create_test_frame();
        let config = CodeGenConfig {
            enable_container_queries: true,
            responsive_breakpoint_px: 640.0,
            ..Default::default()
        };

        let breakpoint = ResponsiveAnalyzer::calculate_breakpoint(&frame, &config);
        assert_eq!(breakpoint, Some(640.0));
    }

    #[test]
    fn test_get_responsive_classes() {
        let frame = create_test_frame();
        let config = CodeGenConfig {
            enable_container_queries: true,
            responsive_breakpoint_px: 640.0,
            ..Default::default()
        };

        let classes = ResponsiveAnalyzer::get_responsive_classes(&frame, &config);

        assert!(classes.contains(&"@container".to_string()));
        assert!(classes.iter().any(|c| c.contains("@[<640px]:flex-col")));
    }

    #[test]
    fn test_has_varying_child_sizes() {
        let frame = create_test_frame();
        assert!(ResponsiveAnalyzer::has_varying_child_sizes(&frame));
    }

    #[test]
    fn test_should_add_responsive_text() {
        let frame = FrameNode {
            id: "test".to_string(),
            name: "Test".to_string(),
            children: vec![figma_api::models::SubcanvasNode::Text(Box::new(
                figma_api::models::TextNode {
                    id: "text".to_string(),
                    name: "Large Text".to_string(),
                    characters: "Hello".to_string(),
                    style: Box::new(figma_api::models::TypeStyle {
                        font_size: Some(32.0), // Large text
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ))],
            ..Default::default()
        };

        let config = CodeGenConfig {
            enable_container_queries: true,
            ..Default::default()
        };

        assert!(ResponsiveAnalyzer::should_add_responsive_text(
            &frame, &config
        ));
    }
}
