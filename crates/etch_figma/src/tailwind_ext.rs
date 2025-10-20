use crate::analyzers::responsive::ResponsiveAnalyzer;
use crate::codegen_ext::CodeGenConfig;
use figma_api::models::frame_node::{
    CounterAxisAlignItems, GridChildHorizontalAlign, GridChildVerticalAlign, LayoutAlign,
    LayoutMode, LayoutPositioning, LayoutSizingHorizontal, LayoutSizingVertical,
    PrimaryAxisAlignItems,
};
use figma_api::models::{
    EllipseNode, FrameNode, GroupNode, LineNode, RectangleNode, TextNode, VectorNode,
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

    /// Get Tailwind styles with configuration support
    fn to_tailwind_with_config(&self, _config: &CodeGenConfig) -> TailwindStyles {
        self.to_tailwind()
    }

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
    fn layout_classes_with_config(&self, _config: &CodeGenConfig) -> Vec<String> {
        self.layout_classes()
    }
    fn sizing_classes(&self) -> Vec<String>;
    fn spacing_classes(&self) -> Vec<String>;
    fn visual_classes(&self) -> Vec<String>;
    fn position_classes(&self) -> Vec<String>;
    fn add_grid_classes(&self, _classes: &mut Vec<String>) {
        // Default implementation - most node types don't have grid properties
    }
    fn get_grid_child_classes(&self) -> Vec<String> {
        // Default implementation - most node types don't have grid child properties
        Vec::new()
    }

    /// Get positioning classes (for outer div) - handles coordinate system relative to parent
    /// Includes: absolute/fixed/sticky positioning, inset values (top/left/right/bottom), z-index, transforms
    fn positioning_classes(&self) -> Vec<String> {
        // Default implementation - most nodes don't have positioning
        Vec::new()
    }

    /// Get layout and content classes (for inner div) - handles layout for children and content styling
    /// Includes: flex/grid/relative, sizing, padding, gap, alignment, borders, backgrounds, etc.
    fn layout_and_content_classes(&self) -> Vec<String> {
        // Default implementation combines layout, sizing, spacing, and visual
        let mut classes = Vec::new();
        classes.extend(self.layout_classes());
        classes.extend(self.sizing_classes());
        classes.extend(self.spacing_classes());
        classes.extend(self.visual_classes());
        classes
    }
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

    /// Get Tailwind styles with configuration support
    fn to_tailwind_with_config(&self, config: &CodeGenConfig) -> TailwindStyles {
        let mut styles = self.to_tailwind();

        // Add responsive classes if enabled
        if config.enable_container_queries {
            let responsive_classes = ResponsiveAnalyzer::get_responsive_classes(self, config);
            styles.classes.extend(responsive_classes);
        }

        styles
    }

    fn layout_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();

        // Layout mode (auto-layout direction)
        if let Some(layout_mode) = &self.layout_mode {
            match layout_mode {
                LayoutMode::Horizontal => {
                    classes.push("flex".to_string());
                    classes.push("flex-row".to_string());
                }
                LayoutMode::Vertical => {
                    classes.push("flex".to_string());
                    classes.push("flex-col".to_string());
                }
                LayoutMode::Grid => {
                    classes.push("grid".to_string());
                    // Add grid-specific classes
                    self.add_grid_classes(&mut classes);
                }
                LayoutMode::None => {
                    // For non-auto-layout frames, use relative positioning
                    classes.push("relative".to_string());
                }
            }
        } else {
            classes.push("relative".to_string());
        }

        // Primary axis alignment (justify-content for flex, justify-items for grid)
        if let Some(primary_align) = &self.primary_axis_align_items {
            let class = match primary_align {
                PrimaryAxisAlignItems::Min => {
                    if matches!(self.layout_mode, Some(LayoutMode::Grid)) {
                        "justify-items-start"
                    } else {
                        "justify-start"
                    }
                }
                PrimaryAxisAlignItems::Center => {
                    if matches!(self.layout_mode, Some(LayoutMode::Grid)) {
                        "justify-items-center"
                    } else {
                        "justify-center"
                    }
                }
                PrimaryAxisAlignItems::Max => {
                    if matches!(self.layout_mode, Some(LayoutMode::Grid)) {
                        "justify-items-end"
                    } else {
                        "justify-end"
                    }
                }
                PrimaryAxisAlignItems::SpaceBetween => {
                    if matches!(self.layout_mode, Some(LayoutMode::Grid)) {
                        "justify-items-stretch"
                    } else {
                        "justify-between"
                    }
                }
            };
            classes.push(class.to_string());
        }

        // Counter axis alignment (align-items for flex, align-items for grid)
        if let Some(counter_align) = &self.counter_axis_align_items {
            let class = match counter_align {
                CounterAxisAlignItems::Min => {
                    if matches!(self.layout_mode, Some(LayoutMode::Grid)) {
                        "items-start"
                    } else {
                        "items-start"
                    }
                }
                CounterAxisAlignItems::Center => {
                    if matches!(self.layout_mode, Some(LayoutMode::Grid)) {
                        "items-center"
                    } else {
                        "items-center"
                    }
                }
                CounterAxisAlignItems::Max => {
                    if matches!(self.layout_mode, Some(LayoutMode::Grid)) {
                        "items-end"
                    } else {
                        "items-end"
                    }
                }
                CounterAxisAlignItems::Baseline => "items-baseline",
            };
            classes.push(class.to_string());
        }

        classes
    }

    /// Add grid-specific classes based on Figma grid properties
    fn add_grid_classes(&self, classes: &mut Vec<String>) {
        // Grid column count
        if let Some(col_count) = self.grid_column_count {
            if col_count > 0.0 {
                classes.push(format!("grid-cols-{}", col_count as usize));
            }
        }

        // Grid row count
        if let Some(row_count) = self.grid_row_count {
            if row_count > 0.0 {
                classes.push(format!("grid-rows-{}", row_count as usize));
            }
        }

        // Grid column sizing (use custom values if provided)
        if let Some(col_sizing) = &self.grid_columns_sizing {
            if !col_sizing.is_empty() {
                classes.push(format!("grid-cols-[{}]", col_sizing));
            }
        }

        // Grid row sizing (use custom values if provided)
        if let Some(row_sizing) = &self.grid_rows_sizing {
            if !row_sizing.is_empty() {
                classes.push(format!("grid-rows-[{}]", row_sizing));
            }
        }

        // Grid gaps
        if let Some(col_gap) = self.grid_column_gap {
            if col_gap > 0.0 {
                classes.push(format!("gap-x-[{}px]", col_gap));
            }
        }

        if let Some(row_gap) = self.grid_row_gap {
            if row_gap > 0.0 {
                classes.push(format!("gap-y-[{}px]", row_gap));
            }
        }

        // Default grid properties if none specified
        if self.grid_column_count.is_none() && self.grid_columns_sizing.is_none() {
            classes.push("grid-cols-[max-content]".to_string());
        }
        if self.grid_row_count.is_none() && self.grid_rows_sizing.is_none() {
            classes.push("grid-rows-[max-content]".to_string());
        }
    }

    /// Get grid child alignment classes for individual grid items
    fn get_grid_child_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();

        // Grid child horizontal alignment
        if let Some(horizontal_align) = &self.grid_child_horizontal_align {
            let class = match horizontal_align {
                GridChildHorizontalAlign::Min => "justify-self-start",
                GridChildHorizontalAlign::Center => "justify-self-center",
                GridChildHorizontalAlign::Max => "justify-self-end",
                GridChildHorizontalAlign::Auto => "justify-self-start",
            };
            classes.push(class.to_string());
        }

        // Grid child vertical alignment
        if let Some(vertical_align) = &self.grid_child_vertical_align {
            let class = match vertical_align {
                GridChildVerticalAlign::Min => "self-start",
                GridChildVerticalAlign::Center => "self-center",
                GridChildVerticalAlign::Max => "self-end",
                GridChildVerticalAlign::Auto => "self-start",
            };
            classes.push(class.to_string());
        }

        classes
    }

    /// Get layout classes with container query support
    fn layout_classes_with_config(&self, config: &CodeGenConfig) -> Vec<String> {
        let mut classes = self.layout_classes();

        // Add container query support
        if config.enable_container_queries
            && ResponsiveAnalyzer::should_add_container_query(self, config)
        {
            let breakpoint = ResponsiveAnalyzer::calculate_breakpoint(self, config)
                .unwrap_or(config.responsive_breakpoint_px);
            add_container_query_classes(self, &mut classes, breakpoint);
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
            if pt > 0.0 {
                classes.push(format!("pt-[{}px]", pt));
            }
            if pr > 0.0 {
                classes.push(format!("pr-[{}px]", pr));
            }
            if pb > 0.0 {
                classes.push(format!("pb-[{}px]", pb));
            }
            if pl > 0.0 {
                classes.push(format!("pl-[{}px]", pl));
            }
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

        // Effects (shadows, blurs)
        classes.extend(effects_to_classes(&self.effects));

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

    /// Get positioning classes (outer div) - handles coordinate system relative to parent
    fn positioning_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();

        // Layout positioning (absolute/fixed)
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

        // Transform (rotation) - affects positioning coordinate system
        if let Some(rotation) = self.rotation {
            if rotation != 0.0 {
                let degrees = rotation * 180.0 / std::f64::consts::PI;
                classes.push(format!("rotate-[{}deg]", degrees));
            }
        }

        // TODO: Add inset values (top/left/right/bottom) when we have position data
        // TODO: Add z-index when available

        classes
    }

    /// Get layout and content classes (inner div) - handles layout for children and content styling
    fn layout_and_content_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();

        // Layout mode and alignment
        classes.extend(self.layout_classes());

        // Sizing (width, height, flex-grow, etc.)
        classes.extend(self.sizing_classes());

        // Spacing (padding, gap, margin)
        classes.extend(self.spacing_classes());

        // Visual properties (background, border, opacity, effects)
        classes.extend(self.visual_classes());

        // Layout align - how this element behaves within parent's layout
        if let Some(align) = &self.layout_align {
            let class = match align {
                LayoutAlign::Inherit => None,
                LayoutAlign::Stretch => Some("self-stretch"),
                LayoutAlign::Min => Some("self-start"),
                LayoutAlign::Center => Some("self-center"),
                LayoutAlign::Max => Some("self-end"),
            };
            if let Some(class) = class {
                classes.push(class.to_string());
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
            styles
                .inline_styles
                .insert("opacity".to_string(), opacity.to_string());
        }
    }

    // Counter axis spacing (for wrapped layouts)
    if let Some(counter_spacing) = frame.counter_axis_spacing {
        if counter_spacing > 0.0 {
            styles.css_variables.insert(
                "--counter-axis-spacing".to_string(),
                format!("{}px", counter_spacing),
            );
        }
    }
}

/// Convert Figma effects to Tailwind classes
fn effects_to_classes(effects: &[figma_api::models::Effect]) -> Vec<String> {
    use figma_api::models::Effect;

    let mut classes = Vec::new();

    for effect in effects {
        match effect {
            Effect::DropShadow(drop_shadow) => {
                // Skip if effect is not visible
                if !drop_shadow.visible {
                    continue;
                }

                // Map drop shadow to Tailwind shadow classes based on blur radius
                let radius = drop_shadow.radius;

                let shadow_class = if radius <= 2.0 {
                    "shadow-sm"
                } else if radius <= 4.0 {
                    "shadow"
                } else if radius <= 8.0 {
                    "shadow-md"
                } else if radius <= 16.0 {
                    "shadow-lg"
                } else if radius <= 24.0 {
                    "shadow-xl"
                } else {
                    "shadow-2xl"
                };

                classes.push(shadow_class.to_string());
            }
            Effect::InnerShadow(inner_shadow) => {
                if inner_shadow.visible {
                    classes.push("shadow-inner".to_string());
                }
            }
            Effect::LayerBlur(layer_blur) => {
                // Skip if effect is not visible
                if !layer_blur.visible {
                    continue;
                }

                // Blur effects with Tailwind's blur utility
                let radius = layer_blur.radius;
                if radius > 0.0 {
                    let blur_class = if radius <= 4.0 {
                        "blur-sm"
                    } else if radius <= 8.0 {
                        "blur"
                    } else if radius <= 12.0 {
                        "blur-md"
                    } else if radius <= 16.0 {
                        "blur-lg"
                    } else if radius <= 24.0 {
                        "blur-xl"
                    } else {
                        "blur-2xl"
                    };
                    classes.push(blur_class.to_string());
                }
            }
            Effect::BackgroundBlur(bg_blur) => {
                // Skip if effect is not visible
                if !bg_blur.visible {
                    continue;
                }

                // Background blur uses backdrop-filter
                let radius = bg_blur.radius;
                if radius > 0.0 {
                    let blur_class = if radius <= 4.0 {
                        "backdrop-blur-sm"
                    } else if radius <= 8.0 {
                        "backdrop-blur"
                    } else if radius <= 12.0 {
                        "backdrop-blur-md"
                    } else if radius <= 16.0 {
                        "backdrop-blur-lg"
                    } else if radius <= 24.0 {
                        "backdrop-blur-xl"
                    } else {
                        "backdrop-blur-2xl"
                    };
                    classes.push(blur_class.to_string());
                }
            }
            Effect::Texture(_) => {
                // Texture effects are complex and may need custom handling
                // Skip for now
            }
            Effect::Noise(_) => {
                // Noise effects are complex and may need custom handling
                // Skip for now
            }
        }
    }

    classes
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

        // Letter spacing
        if let Some(letter_spacing) = self.style.letter_spacing {
            // Convert to em units for better scaling
            let em_value = letter_spacing / self.style.font_size.unwrap_or(16.0);
            classes.push(format!("tracking-[{}em]", em_value));
        }

        // Text color from fills
        if let Some(fills) = &self.style.fills {
            if let Some(color) = crate::svg::utils::paint::extract_first_paint_color(fills) {
                // Use arbitrary value for exact color match
                classes.push(format!("text-[{}]", color));
            }
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

    /// Get positioning classes (outer div) - handles coordinate system relative to parent
    fn positioning_classes(&self) -> Vec<String> {
        let classes = Vec::new();

        // Groups may have positioning based on their context
        // TODO: Add positioning logic when group position data is available

        classes
    }

    /// Get layout and content classes (inner div) - handles layout for children and content styling
    fn layout_and_content_classes(&self) -> Vec<String> {
        let mut classes = Vec::new();

        // Groups establish a relative coordinate system for their children
        classes.push("relative".to_string());

        // Add any visual styles if available
        classes.extend(self.visual_classes());

        classes
    }
}

// Simple implementations for shape nodes
impl TailwindStyleExt for RectangleNode {
    fn to_tailwind(&self) -> TailwindStyles {
        let mut styles = TailwindStyles::default();
        styles.classes.push("inline-block".to_string());
        styles
    }

    fn layout_classes(&self) -> Vec<String> {
        vec!["inline-block".to_string()]
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

impl TailwindStyleExt for EllipseNode {
    fn to_tailwind(&self) -> TailwindStyles {
        let mut styles = TailwindStyles::default();
        styles.classes.push("inline-block".to_string());
        styles
    }

    fn layout_classes(&self) -> Vec<String> {
        vec!["inline-block".to_string()]
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

impl TailwindStyleExt for LineNode {
    fn to_tailwind(&self) -> TailwindStyles {
        let mut styles = TailwindStyles::default();
        styles.classes.push("inline-block".to_string());
        styles
    }

    fn layout_classes(&self) -> Vec<String> {
        vec!["inline-block".to_string()]
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

impl TailwindStyleExt for VectorNode {
    fn to_tailwind(&self) -> TailwindStyles {
        let mut styles = TailwindStyles::default();
        styles.classes.push("inline-block".to_string());
        styles
    }

    fn layout_classes(&self) -> Vec<String> {
        vec!["inline-block".to_string()]
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

/// Add text-specific inline styles
fn add_text_inline_styles(text: &TextNode, styles: &mut TailwindStyles) {
    // Line height - use Tailwind arbitrary value
    if let Some(line_height_px) = text.style.line_height_px {
        if let Some(font_size) = text.style.font_size {
            // Convert to relative line height
            let line_height_ratio = line_height_px / font_size;
            styles
                .classes
                .push(format!("leading-[{}]", line_height_ratio));
        } else {
            // Fallback to px value
            styles
                .classes
                .push(format!("leading-[{}px]", line_height_px));
        }
    }

    // Font family - use CSS variable for flexibility
    if let Some(font_family) = &text.style.font_family {
        let clean_font_family = font_family.trim().trim_matches(|c| c == '\'' || c == '"');
        // Use Tailwind arbitrary font family
        styles.classes.push(format!(
            "font-['{}']",
            clean_font_family.replace('\'', "\\'")
        ));
    }
}

/// Add container query classes for responsive behavior
fn add_container_query_classes(frame: &FrameNode, classes: &mut Vec<String>, breakpoint: f64) {
    // Mark as container
    classes.push("@container".to_string());

    // Add responsive classes based on layout
    if let Some(layout_mode) = &frame.layout_mode {
        match layout_mode {
            LayoutMode::Horizontal => {
                // Switch to column when narrow
                classes.push(format!("@[<{}px]:flex-col", breakpoint));
                classes.push(format!("@[<{}px]:items-stretch", breakpoint));
            }
            LayoutMode::Vertical => {
                // Could switch to row when wide
                classes.push(format!("@[>{}px]:flex-row", breakpoint * 2.0));
            }
            _ => {}
        }
    }

    // Adjust gap for narrow containers
    if let Some(gap) = frame.item_spacing {
        if gap > 16.0 {
            classes.push(format!("@[<{}px]:gap-[{}px]", breakpoint, gap / 2.0));
        }
    }

    // Add responsive padding adjustments
    let pt = frame.padding_top.unwrap_or(0.0);
    let pr = frame.padding_right.unwrap_or(0.0);
    let pb = frame.padding_bottom.unwrap_or(0.0);
    let pl = frame.padding_left.unwrap_or(0.0);

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
