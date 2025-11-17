//! Style generation and Tailwind CSS utilities
//!
//! This module provides functionality for generating Tailwind CSS classes
//! from Figma style properties (layout, colors, borders, effects, etc.).

/// Generate Tailwind absolute positioning classes from coordinates
pub fn create_absolute_position_classes(x: f64, y: f64) -> Vec<String> {
    vec![
        "absolute".to_string(),
        format!("top-[{}px]", y),
        format!("left-[{}px]", x),
    ]
}

/// Generate Tailwind positioning classes based on position type
pub fn create_position_classes(
    position_type: PositionType,
    x: f64,
    y: f64,
) -> Vec<String> {
    match position_type {
        PositionType::Absolute => create_absolute_position_classes(x, y),
        PositionType::Relative => vec!["relative".to_string()],
        PositionType::Fixed => vec![
            "fixed".to_string(),
            format!("top-[{}px]", y),
            format!("left-[{}px]", x),
        ],
        PositionType::Sticky => vec![
            "sticky".to_string(),
            format!("top-[{}px]", y),
            format!("left-[{}px]", x),
        ],
    }
}

/// CSS position type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionType {
    Absolute,
    Relative,
    Fixed,
    Sticky,
}

/// Layout mode for Figma frames
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// No specific layout (absolute positioning)
    None,
    /// Horizontal auto-layout (flexbox row)
    Horizontal,
    /// Vertical auto-layout (flexbox column)
    Vertical,
}

impl LayoutMode {
    /// Convert Figma layoutMode string to LayoutMode enum
    pub fn from_figma(mode: &str) -> Self {
        match mode {
            "HORIZONTAL" => LayoutMode::Horizontal,
            "VERTICAL" => LayoutMode::Vertical,
            _ => LayoutMode::None,
        }
    }

    /// Convert to Tailwind flex direction class
    pub fn to_flex_direction(&self) -> Option<&'static str> {
        match self {
            LayoutMode::Horizontal => Some("flex-row"),
            LayoutMode::Vertical => Some("flex-col"),
            LayoutMode::None => None,
        }
    }
}

/// Generate Tailwind flexbox classes from Figma auto-layout properties
pub fn create_flex_classes(
    layout_mode: LayoutMode,
    primary_axis_align: Option<&str>,
    counter_axis_align: Option<&str>,
    gap: Option<f64>,
) -> Vec<String> {
    let mut classes = vec!["flex".to_string()];

    // Direction
    if let Some(direction) = layout_mode.to_flex_direction() {
        classes.push(direction.to_string());
    }

    // Main axis alignment
    if let Some(align) = primary_axis_align {
        let class = match align {
            "MIN" => "justify-start",
            "CENTER" => "justify-center",
            "MAX" => "justify-end",
            "SPACE_BETWEEN" => "justify-between",
            _ => "justify-start",
        };
        classes.push(class.to_string());
    }

    // Cross axis alignment
    if let Some(align) = counter_axis_align {
        let class = match align {
            "MIN" => "items-start",
            "CENTER" => "items-center",
            "MAX" => "items-end",
            _ => "items-start",
        };
        classes.push(class.to_string());
    }

    // Gap
    if let Some(gap_value) = gap {
        classes.push(format!("gap-[{}px]", gap_value));
    }

    classes
}

/// Generate Tailwind padding classes from Figma padding values
pub fn create_padding_classes(
    padding_left: Option<f64>,
    padding_right: Option<f64>,
    padding_top: Option<f64>,
    padding_bottom: Option<f64>,
) -> Vec<String> {
    let mut classes = Vec::new();

    // Check if all sides are equal
    if padding_left == padding_right
        && padding_right == padding_top
        && padding_top == padding_bottom
    {
        if let Some(padding) = padding_left {
            if padding > 0.0 {
                classes.push(format!("p-[{}px]", padding));
            }
        }
        return classes;
    }

    // Individual sides
    if let Some(p) = padding_left {
        if p > 0.0 {
            classes.push(format!("pl-[{}px]", p));
        }
    }
    if let Some(p) = padding_right {
        if p > 0.0 {
            classes.push(format!("pr-[{}px]", p));
        }
    }
    if let Some(p) = padding_top {
        if p > 0.0 {
            classes.push(format!("pt-[{}px]", p));
        }
    }
    if let Some(p) = padding_bottom {
        if p > 0.0 {
            classes.push(format!("pb-[{}px]", p));
        }
    }

    classes
}

