use figma_api::models::{GradientPaint, Paint, Rgba};

/// Convert a Figma Paint to CSS color string
pub fn paint_to_css(paint: &Paint) -> Option<String> {
    match paint {
        Paint::SolidPaint(solid_paint) => Some(rgba_to_css(
            &solid_paint.color,
            solid_paint.opacity.unwrap_or(1.0),
        )),
        Paint::GradientPaint(gradient) => {
            // Check gradient type to determine rendering
            Some(gradient_to_css(gradient))
        }
        Paint::ImagePaint(_) => {
            // Image fills need special handling
            None
        }
        Paint::PatternPaint(_) => {
            // Pattern fills need special handling
            None
        }
    }
}

/// Convert RGBA to CSS color string
pub fn rgba_to_css(color: &Rgba, opacity: f64) -> String {
    let r = (color.r * 255.0).round() as u8;
    let g = (color.g * 255.0).round() as u8;
    let b = (color.b * 255.0).round() as u8;
    let a = color.a * opacity;

    if a >= 0.999 {
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    } else {
        format!("rgba({}, {}, {}, {:.3})", r, g, b, a)
    }
}

/// Convert RGBA to hex string (no alpha)
pub fn rgba_to_hex(color: &Rgba) -> String {
    let r = (color.r * 255.0).round() as u8;
    let g = (color.g * 255.0).round() as u8;
    let b = (color.b * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Convert gradient to CSS
pub fn gradient_to_css(gradient: &GradientPaint) -> String {
    // Extract gradient stops
    let stops = gradient
        .gradient_stops
        .iter()
        .map(|stop| {
            let color = rgba_to_css(&stop.color, 1.0);
            let position = (stop.position * 100.0).round();
            format!("{} {}%", color, position)
        })
        .collect::<Vec<_>>()
        .join(", ");

    // Calculate angle from gradient handles
    // gradient_handle_positions: [start, end, perpendicular]
    // where each element is a Vector with x, y coordinates
    let angle = if gradient.gradient_handle_positions.len() >= 2 {
        let start = &gradient.gradient_handle_positions[0];
        let end = &gradient.gradient_handle_positions[1];

        // Calculate angle in degrees
        let angle_rad = (end.y - start.y).atan2(end.x - start.x);
        let angle_deg = angle_rad.to_degrees() + 90.0; // Adjust to CSS gradient convention
        format!("{}deg", angle_deg.round())
    } else {
        "180deg".to_string() // Default to top-to-bottom
    };

    format!("linear-gradient({}, {})", angle, stops)
}

/// Extract the first valid color from a list of paints
pub fn extract_first_paint_color(paints: &[Paint]) -> Option<String> {
    paints.iter().find_map(|paint| paint_to_css(paint))
}

/// Check if paint has complex features that need API fallback
pub fn paint_needs_api_fallback(paint: &Paint) -> bool {
    matches!(paint, Paint::ImagePaint(_) | Paint::PatternPaint(_))
}

/// Extract stroke properties for SVG
pub fn stroke_to_svg_attrs(
    strokes: &Option<Vec<Paint>>,
    stroke_weight: Option<f64>,
) -> Vec<(&'static str, String)> {
    let mut attrs = Vec::new();

    if let Some(strokes) = strokes {
        if let Some(color) = extract_first_paint_color(strokes) {
            attrs.push(("stroke", color));
        }
    }

    if let Some(weight) = stroke_weight {
        if weight > 0.0 {
            attrs.push(("strokeWidth", weight.to_string()));
        }
    }

    attrs
}

/// Extract fill properties for SVG
pub fn fill_to_svg_attr(fills: &[Paint]) -> Option<(&'static str, String)> {
    extract_first_paint_color(fills).map(|color| ("fill", color))
}

/// Convert paint to CSS variable with fallback
/// Format: var(--fill-N, #color)
pub fn paint_to_css_var(paint: &Paint, var_index: usize) -> Option<String> {
    paint_to_css(paint).map(|fallback| format!("var(--fill-{}, {})", var_index, fallback))
}

/// Extract fill properties for SVG with CSS variable support
pub fn fill_to_svg_attr_with_var(
    fills: &[Paint],
    var_index: usize,
) -> Option<(&'static str, String)> {
    fills
        .first()
        .and_then(|paint| paint_to_css_var(paint, var_index).map(|value| ("fill", value)))
}

/// Extract stroke properties for SVG with CSS variable support
pub fn stroke_to_svg_attrs_with_var(
    strokes: &Option<Vec<Paint>>,
    stroke_weight: Option<f64>,
    var_index: usize,
) -> Vec<(&'static str, String)> {
    let mut attrs = Vec::new();

    if let Some(strokes) = strokes {
        if let Some(paint) = strokes.first() {
            if let Some(color) = paint_to_css_var(paint, var_index) {
                attrs.push(("stroke", color));
            }
        }
    }

    if let Some(weight) = stroke_weight {
        if weight > 0.0 {
            attrs.push(("strokeWidth", weight.to_string()));
        }
    }

    attrs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgba_to_css() {
        let color = Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        assert_eq!(rgba_to_css(&color, 1.0), "#ff0000");

        let color = Rgba {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 0.5,
        };
        let result = rgba_to_css(&color, 1.0);
        assert!(result.starts_with("rgba("));
    }

    #[test]
    fn test_rgba_to_hex() {
        let color = Rgba {
            r: 0.0,
            g: 0.5,
            b: 1.0,
            a: 1.0,
        };
        assert_eq!(rgba_to_hex(&color), "#0080ff");
    }
}
