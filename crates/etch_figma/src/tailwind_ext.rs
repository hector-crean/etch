use figma_api::models::{FrameNode, TextNode, VectorNode, RectangleNode, EllipseNode, LineNode, GroupNode};
use figma_api::models::frame_node::{
    LayoutMode, LayoutAlign, LayoutSizingHorizontal, LayoutSizingVertical, 
    LayoutPositioning, PrimaryAxisAlignItems, CounterAxisAlignItems
};
use std::collections::HashMap;

/// Result of mapping Figma properties to CSS classes and styles
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct TailwindStyles {
    /// Tailwind CSS classes
    pub classes: Vec<String>,
    /// Inline CSS styles for properties that can't be expressed in Tailwind
    pub inline_styles: HashMap<String, String>,
    /// CSS custom properties (variables)
    pub css_variables: HashMap<String, String>,
}

impl TailwindStyles {
    /// Get all classes as a single space-separated string
    pub fn class_string(&self) -> String {
        self.classes.join(" ")
    }
    
    /// Check if there are any styles to apply
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty() && self.inline_styles.is_empty() && self.css_variables.is_empty()
    }
    
    /// Merge with another TailwindStyles
    pub fn merge(&mut self, other: TailwindStyles) {
        self.classes.extend(other.classes);
        self.inline_styles.extend(other.inline_styles);
        self.css_variables.extend(other.css_variables);
    }
}

/// Extension trait for FrameNode to generate Tailwind CSS classes
pub trait TailwindStyleExt {
    /// Get all Tailwind styles for this frame
    fn to_tailwind(&self) -> TailwindStyles;
    
    /// Get just the CSS classes as a vector
    fn to_tailwind_classes(&self) -> Vec<String> {
        self.to_tailwind().classes
    }
    
    /// Get just the CSS classes as a space-separated string
    fn to_tailwind_class_string(&self) -> String {
        self.to_tailwind().class_string()
    }
    
    // Specific property mappings
    fn layout_classes(&self) -> Vec<String>;
    fn sizing_classes(&self) -> Vec<String>;
    fn spacing_classes(&self) -> Vec<String>;
    fn visual_classes(&self) -> Vec<String>;
    fn position_classes(&self) -> Vec<String>;
}

impl TailwindStyleExt for FrameNode {
    fn to_tailwind(&self) -> TailwindStyles {
        let mut styles = TailwindStyles::default();
        
        // Combine all style categories
        styles.classes.extend(self.layout_classes());
        styles.classes.extend(self.sizing_classes());
        styles.classes.extend(self.spacing_classes());
        styles.classes.extend(self.visual_classes());
        styles.classes.extend(self.position_classes());
        
        // Add inline styles for properties that need them
        add_inline_styles(self, &mut styles);
        
        styles
    }
    
    fn layout_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();
        
        // Layout mode (auto-layout direction)
        if let Some(layout_mode) = &self.layout_mode {
            classes.push("flex".to_string());
            
            match layout_mode {
                LayoutMode::Horizontal => {
                    classes.push("flex-row".to_string());
                }
                LayoutMode::Vertical => {
                    classes.push("flex-col".to_string());
                }
                _ => {
                    // For non-auto-layout frames, use relative positioning
                    classes.pop(); // Remove flex
                    classes.push("relative".to_string());
                }
            }
        } else {
            classes.push("relative".to_string());
        }
        
        // Primary axis alignment (justify-content)
        if let Some(primary_align) = &self.primary_axis_align_items {
            let class = match primary_align {
                PrimaryAxisAlignItems::Min => "justify-start",
                PrimaryAxisAlignItems::Center => "justify-center", 
                PrimaryAxisAlignItems::Max => "justify-end",
                PrimaryAxisAlignItems::SpaceBetween => "justify-between",
            };
            classes.push(class.to_string());
        }
        
        // Counter axis alignment (align-items)
        if let Some(counter_align) = &self.counter_axis_align_items {
            let class = match counter_align {
                CounterAxisAlignItems::Min => "items-start",
                CounterAxisAlignItems::Center => "items-center",
                CounterAxisAlignItems::Max => "items-end", 
                CounterAxisAlignItems::Baseline => "items-baseline",
            };
            classes.push(class.to_string());
        }
        
        classes
    }
    
    fn sizing_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();
        
        // Horizontal sizing
        if let Some(h_sizing) = &self.layout_sizing_horizontal {
            match h_sizing {
                LayoutSizingHorizontal::Fixed => {
                    if let Some(size) = &self.size {
                        classes.push(format!("w-[{}px]", size.x));
                    }
                }
                LayoutSizingHorizontal::Hug => {
                    classes.push("w-fit".to_string());
                }
                LayoutSizingHorizontal::Fill => {
                    classes.push("w-full".to_string());
                }
            }
        }
        
        // Vertical sizing
        if let Some(v_sizing) = &self.layout_sizing_vertical {
            match v_sizing {
                LayoutSizingVertical::Fixed => {
                    if let Some(size) = &self.size {
                        classes.push(format!("h-[{}px]", size.y));
                    }
                }
                LayoutSizingVertical::Hug => {
                    classes.push("h-fit".to_string());
                }
                LayoutSizingVertical::Fill => {
                    classes.push("h-full".to_string());
                }
            }
        }
        
        // Flex grow
        if let Some(grow) = self.layout_grow {
            if grow > 0.0 {
                if grow == 1.0 {
                    classes.push("flex-1".to_string());
                } else {
                    classes.push(format!("flex-[{}]", grow));
                }
            }
        }
        
        // Min/max constraints
        if let Some(min_w) = self.min_width {
            classes.push(format!("min-w-[{}px]", min_w));
        }
        if let Some(max_w) = self.max_width {
            classes.push(format!("max-w-[{}px]", max_w));
        }
        if let Some(min_h) = self.min_height {
            classes.push(format!("min-h-[{}px]", min_h));
        }
        if let Some(max_h) = self.max_height {
            classes.push(format!("max-h-[{}px]", max_h));
        }
        
        classes
    }
    
    fn spacing_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();
        
        // Padding
        let pt = self.padding_top.unwrap_or(0.0);
        let pr = self.padding_right.unwrap_or(0.0);
        let pb = self.padding_bottom.unwrap_or(0.0);
        let pl = self.padding_left.unwrap_or(0.0);
        
        // Check for uniform padding
        if pt == pr && pr == pb && pb == pl && pt > 0.0 {
            classes.push(format!("p-[{}px]", pt));
        } else {
            // Individual padding values
            if pt > 0.0 { classes.push(format!("pt-[{}px]", pt)); }
            if pr > 0.0 { classes.push(format!("pr-[{}px]", pr)); }
            if pb > 0.0 { classes.push(format!("pb-[{}px]", pb)); }
            if pl > 0.0 { classes.push(format!("pl-[{}px]", pl)); }
        }
        
        // Gap between children (item spacing)
        if let Some(gap) = self.item_spacing {
            if gap > 0.0 {
                classes.push(format!("gap-[{}px]", gap));
            }
        }
        
        classes
    }
    
    fn visual_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();
        
        // Background fills - simplified
        if !self.fills.is_empty() {
            classes.push("bg-white".to_string()); // Default for now
        }
        
        // Strokes (borders) - simplified
        if let Some(strokes) = &self.strokes {
            if !strokes.is_empty() {
                if let Some(stroke_weight) = self.stroke_weight {
                    if stroke_weight > 0.0 {
                        classes.push(format!("border-[{}px]", stroke_weight));
                        classes.push("border-solid".to_string());
                        classes.push("border-gray-300".to_string());
                    }
                }
            }
        }
        
        // Corner radius
        if let Some(corner_radius) = self.corner_radius {
            if corner_radius > 0.0 {
                classes.extend(rounded_classes(corner_radius));
            }
        }
        
        // Opacity
        if let Some(opacity) = self.opacity {
            if opacity < 1.0 {
                if let Some(opacity_class) = opacity_class(opacity) {
                    classes.push(opacity_class);
                }
            }
        }
        
        classes
    }
    
    fn position_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();
        
        // Layout positioning
        if let Some(positioning) = &self.layout_positioning {
            match positioning {
                LayoutPositioning::Absolute => {
                    classes.push("absolute".to_string());
                }
                LayoutPositioning::Auto => {
                    // Default behavior - no class needed
                }
            }
        }
        
        // Layout align (for children of auto-layout frames)
        if let Some(align) = &self.layout_align {
            let class = match align {
                LayoutAlign::Inherit => return classes, // No class needed
                LayoutAlign::Stretch => "self-stretch",
                LayoutAlign::Min => "self-start",
                LayoutAlign::Center => "self-center", 
                LayoutAlign::Max => "self-end",
            };
            classes.push(class.to_string());
        }
        
        // Transform (rotation)
        if let Some(rotation) = self.rotation {
            if rotation != 0.0 {
                let degrees = rotation * 180.0 / std::f64::consts::PI;
                classes.push(format!("rotate-[{}deg]", degrees));
            }
        }
        
        classes
    }
}

// Helper functions for FrameNode styling
/// Add inline styles that can't be expressed as Tailwind classes
fn add_inline_styles(frame: &FrameNode, styles: &mut TailwindStyles) {
    // Custom opacity values
    if let Some(opacity) = frame.opacity {
        if opacity < 1.0 && opacity_class(opacity).is_none() {
            styles.inline_styles.insert("opacity".to_string(), opacity.to_string());
        }
    }
    
    // Counter axis spacing (for wrapped layouts)
    if let Some(counter_spacing) = frame.counter_axis_spacing {
        if counter_spacing > 0.0 {
            styles.css_variables.insert(
                "--counter-axis-spacing".to_string(),
                format!("{}px", counter_spacing)
            );
        }
    }
}

/// Get rounded classes for a radius value
fn rounded_classes(radius: f64) -> Vec<String> {
    if radius <= 0.0 {
        return vec![];
    }
    
    let class = match radius as u8 {
        0 => return vec![],
        2 => "rounded-sm",
        4 => "rounded",
        6 => "rounded-md", 
        8 => "rounded-lg",
        12 => "rounded-xl",
        16 => "rounded-2xl",
        24 => "rounded-3xl",
        _ if radius >= 9999.0 => "rounded-full",
        _ => return vec![format!("rounded-[{}px]", radius)],
    };
    vec![class.to_string()]
}

/// Get opacity class for common values, None for custom values
fn opacity_class(opacity: f64) -> Option<String> {
    let class = match (opacity * 100.0) as u8 {
        0 => "opacity-0",
        5 => "opacity-5",
        10 => "opacity-10",
        20 => "opacity-20",
        25 => "opacity-25",
        30 => "opacity-30",
        40 => "opacity-40",
        50 => "opacity-50",
        60 => "opacity-60",
        70 => "opacity-70",
        75 => "opacity-75",
        80 => "opacity-80",
        90 => "opacity-90",
        95 => "opacity-95",
        _ => return None, // Use inline style for custom values
    };
    Some(class.to_string())
}

// Implement TailwindStyleExt for other node types
impl TailwindStyleExt for TextNode {
    fn to_tailwind(&self) -> TailwindStyles {
        let mut styles = TailwindStyles::default();
        
        // Add text-specific classes
        styles.classes.extend(self.layout_classes());
        styles.classes.extend(self.sizing_classes());
        styles.classes.extend(self.spacing_classes());
        styles.classes.extend(self.visual_classes());
        styles.classes.extend(self.position_classes());
        
        // Add text-specific inline styles
        add_text_inline_styles(self, &mut styles);
        
        styles
    }
    
    fn layout_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();
        
        // Text nodes are typically inline or block
        classes.push("block".to_string());
        
        // Add text alignment from style
        if let Some(align) = &self.style.text_align_horizontal {
            let class = match align {
                figma_api::models::type_style::TextAlignHorizontal::Left => "text-left",
                figma_api::models::type_style::TextAlignHorizontal::Center => "text-center",
                figma_api::models::type_style::TextAlignHorizontal::Right => "text-right",
                figma_api::models::type_style::TextAlignHorizontal::Justified => "text-justify",
            };
            classes.push(class.to_string());
        }
        
        classes
    }
    
    fn sizing_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();
        
        // Text sizing is handled by font-size, but we can add width/height if specified
        if let Some(size) = &self.size {
            classes.push(format!("w-[{}px]", size.x));
            classes.push(format!("h-[{}px]", size.y));
        }
        
        classes
    }
    
    fn spacing_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();
        
        // Letter spacing
        if let Some(letter_spacing) = self.style.letter_spacing {
            if letter_spacing != 0.0 {
                classes.push(format!("tracking-[{}em]", letter_spacing / 16.0)); // Convert px to em
            }
        }
        
        classes
    }
    
    fn visual_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();
        
        // Font weight
        if let Some(font_weight) = self.style.font_weight {
            let weight_class = match font_weight as u16 {
                100 => "font-thin",
                200 => "font-extralight",
                300 => "font-light",
                400 => "font-normal",
                500 => "font-medium",
                600 => "font-semibold",
                700 => "font-bold",
                800 => "font-extrabold",
                900 => "font-black",
                _ => "font-normal",
            };
            classes.push(weight_class.to_string());
        }
        
        // Font size
        if let Some(font_size) = self.style.font_size {
            classes.push(format!("text-[{}px]", font_size));
        }
        
        // Text decoration
        if let Some(decoration) = &self.style.text_decoration {
            let decoration_class = match decoration {
                figma_api::models::type_style::TextDecoration::Underline => "underline",
                figma_api::models::type_style::TextDecoration::Strikethrough => "line-through",
                figma_api::models::type_style::TextDecoration::None => "", // No decoration
            };
            if !decoration_class.is_empty() {
                classes.push(decoration_class.to_string());
            }
        }
        
        // Italic
        if self.style.italic.unwrap_or(false) {
            classes.push("italic".to_string());
        }
        
        classes
    }
    
    fn position_classes(&self) -> Vec<String> {
        // Text nodes don't typically have complex positioning
        // This can be enhanced based on the actual TextNode structure
        Vec::new()
    }
}

impl TailwindStyleExt for GroupNode {
    fn to_tailwind(&self) -> TailwindStyles {
        let mut styles = TailwindStyles::default();
        
        // Groups are typically containers
        styles.classes.push("relative".to_string());
        
        styles
    }
    
    fn layout_classes(&self) -> Vec<String> {
        vec!["relative".to_string()]
    }
    
    fn sizing_classes(&self) -> Vec<String> {
        Vec::new()
    }
    
    fn spacing_classes(&self) -> Vec<String> {
        Vec::new()
    }
    
    fn visual_classes(&self) -> Vec<String> {
        Vec::new()
    }
    
    fn position_classes(&self) -> Vec<String> {
        Vec::new()
    }
}

// Simple implementations for shape nodes
impl TailwindStyleExt for RectangleNode {
    fn to_tailwind(&self) -> TailwindStyles {
        let mut styles = TailwindStyles::default();
        styles.classes.push("inline-block".to_string());
        styles
    }
    
    fn layout_classes(&self) -> Vec<String> { vec!["inline-block".to_string()] }
    fn sizing_classes(&self) -> Vec<String> { Vec::new() }
    fn spacing_classes(&self) -> Vec<String> { Vec::new() }
    fn visual_classes(&self) -> Vec<String> { Vec::new() }
    fn position_classes(&self) -> Vec<String> { Vec::new() }
}

impl TailwindStyleExt for EllipseNode {
    fn to_tailwind(&self) -> TailwindStyles {
        let mut styles = TailwindStyles::default();
        styles.classes.push("inline-block".to_string());
        styles
    }
    
    fn layout_classes(&self) -> Vec<String> { vec!["inline-block".to_string()] }
    fn sizing_classes(&self) -> Vec<String> { Vec::new() }
    fn spacing_classes(&self) -> Vec<String> { Vec::new() }
    fn visual_classes(&self) -> Vec<String> { Vec::new() }
    fn position_classes(&self) -> Vec<String> { Vec::new() }
}

impl TailwindStyleExt for LineNode {
    fn to_tailwind(&self) -> TailwindStyles {
        let mut styles = TailwindStyles::default();
        styles.classes.push("inline-block".to_string());
        styles
    }
    
    fn layout_classes(&self) -> Vec<String> { vec!["inline-block".to_string()] }
    fn sizing_classes(&self) -> Vec<String> { Vec::new() }
    fn spacing_classes(&self) -> Vec<String> { Vec::new() }
    fn visual_classes(&self) -> Vec<String> { Vec::new() }
    fn position_classes(&self) -> Vec<String> { Vec::new() }
}

impl TailwindStyleExt for VectorNode {
    fn to_tailwind(&self) -> TailwindStyles {
        let mut styles = TailwindStyles::default();
        styles.classes.push("inline-block".to_string());
        styles
    }
    
    fn layout_classes(&self) -> Vec<String> { vec!["inline-block".to_string()] }
    fn sizing_classes(&self) -> Vec<String> { Vec::new() }
    fn spacing_classes(&self) -> Vec<String> { Vec::new() }
    fn visual_classes(&self) -> Vec<String> { Vec::new() }
    fn position_classes(&self) -> Vec<String> { Vec::new() }
}

/// Add text-specific inline styles
fn add_text_inline_styles(text: &TextNode, styles: &mut TailwindStyles) {
    // Line height
    if let Some(line_height_px) = text.style.line_height_px {
        styles.inline_styles.insert("lineHeight".to_string(), format!("{}px", line_height_px));
    }
    
    // Font family
    if let Some(font_family) = &text.style.font_family {
        styles.inline_styles.insert("fontFamily".to_string(), format!("'{}'", font_family));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use figma_api::models::frame_node::LayoutMode;
    
    #[test]
    fn test_extension_trait_basic() {
        // Create a frame with some basic properties
        let frame = FrameNode {
            id: "test".to_string(),
            name: "Test Frame".to_string(),
            layout_mode: Some(LayoutMode::Horizontal),
            ..Default::default()
        };
        
        let classes = frame.to_tailwind_classes();
        assert!(classes.contains(&"flex".to_string()));
        assert!(classes.contains(&"flex-row".to_string()));
        
        let class_string = frame.to_tailwind_class_string();
        assert!(class_string.contains("flex"));
        assert!(class_string.contains("flex-row"));
    }
    
    #[test]
    fn test_individual_methods() {
        let frame = FrameNode {
            id: "test".to_string(),
            name: "Test Frame".to_string(),
            layout_mode: Some(LayoutMode::Vertical),
            padding_top: Some(16.0),
            padding_bottom: Some(16.0),
            corner_radius: Some(8.0),
            ..Default::default()
        };
        
        let layout_classes = frame.layout_classes();
        assert!(layout_classes.contains(&"flex".to_string()));
        assert!(layout_classes.contains(&"flex-col".to_string()));
        
        let spacing_classes = frame.spacing_classes();
        assert!(spacing_classes.contains(&"pt-[16px]".to_string()));
        assert!(spacing_classes.contains(&"pb-[16px]".to_string()));
        
        let visual_classes = frame.visual_classes();
        assert!(visual_classes.contains(&"rounded-lg".to_string()));
    }
    
    #[test]
    fn test_tailwind_styles_merge() {
        let mut styles1 = TailwindStyles {
            classes: vec!["flex".to_string(), "flex-row".to_string()],
            inline_styles: HashMap::new(),
            css_variables: HashMap::new(),
        };
        
        let styles2 = TailwindStyles {
            classes: vec!["p-4".to_string()],
            inline_styles: [("opacity".to_string(), "0.8".to_string())].into(),
            css_variables: HashMap::new(),
        };
        
        styles1.merge(styles2);
        
        assert_eq!(styles1.classes.len(), 3);
        assert!(styles1.classes.contains(&"flex".to_string()));
        assert!(styles1.classes.contains(&"p-4".to_string()));
        assert_eq!(styles1.inline_styles.len(), 1);
    }
}
