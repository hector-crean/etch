use crate::conversion::jsx::{RenderContext, ToJsx, extract_position_from_transform};
use crate::conversion::styles::create_absolute_position_classes;
use crate::conversion::svg::utils::{fill_to_svg_attr, stroke_to_svg_attrs};
use figma_api::models::{
    EllipseNode, LineNode, Rectangle, RectangleNode, RegularPolygonNode, StarNode,
};
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::{
    Ident, IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement,
    JSXElement, JSXElementChild, JSXElementName, JSXOpeningElement, Lit, Str,
};

/// Wrap an SVG element in a <g> group with transform for positioning
fn wrap_svg_element_in_group(
    svg_element: JSXElement,
    absolute_bounding_box: Option<&Rectangle>,
    parent_x: f64,
    parent_y: f64,
) -> JSXElement {
    let mut attrs = Vec::new();

    // Calculate relative position using bounding box
    if let Some(bbox) = absolute_bounding_box {
        let relative_x = bbox.x - parent_x;
        let relative_y = bbox.y - parent_y;

        // Only add transform if there's actual translation
        if relative_x.abs() > 0.01 || relative_y.abs() > 0.01 {
            let transform = format!("translate({}, {})", relative_x, relative_y);
            attrs.push(create_jsx_attr("transform", &transform));
        }
    }

    // Create <g> element
    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "g".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs,
            self_closing: false,
            type_args: None,
        },
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "g".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
        children: vec![JSXElementChild::JSXElement(Box::new(svg_element))],
    }
}

/// Wrap an SVG element in an absolutely positioned div for HTML context
fn wrap_svg_in_positioned_div(
    svg_element: JSXElement,
    x: f64,
    y: f64,
    _size: &Option<Box<figma_api::models::Vector>>,
) -> JSXElement {
    let mut attrs = Vec::new();

    // Add absolute positioning classes
    let positioning_classes = create_absolute_position_classes(x, y);
    attrs.push(create_jsx_attr("className", &positioning_classes.join(" ")));

    // Add inline block display to ensure proper sizing
    // Width and height will be determined by the SVG element itself

    // Create wrapper div with SVG as child
    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "div".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs,
            self_closing: false,
            type_args: None,
        },
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "div".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
        children: vec![JSXElementChild::JSXElement(Box::new(svg_element))],
    }
}

/// Create JSX element for Rectangle as SVG <rect>
pub fn rectangle_to_svg_jsx(rect: &RectangleNode) -> JSXElement {
    let mut attrs = Vec::new();

    // Add position and size
    if let Some(size) = &rect.size {
        attrs.push(create_jsx_attr("width", &size.x.to_string()));
        attrs.push(create_jsx_attr("height", &size.y.to_string()));
    }

    // Position is typically 0,0 in local coordinate system
    attrs.push(create_jsx_attr("x", "0"));
    attrs.push(create_jsx_attr("y", "0"));

    // Add corner radius if present
    if let Some(radius) = rect.corner_radius {
        if radius > 0.0 {
            attrs.push(create_jsx_attr("rx", &radius.to_string()));
            attrs.push(create_jsx_attr("ry", &radius.to_string()));
        }
    } else if let Some(radii) = &rect.rectangle_corner_radii {
        // Individual corner radii - use the first one for rx/ry (simplified)
        if radii.len() > 0 && radii[0] > 0.0 {
            attrs.push(create_jsx_attr("rx", &radii[0].to_string()));
            attrs.push(create_jsx_attr("ry", &radii[0].to_string()));
        }
    }

    // Add fill
    if let Some(fill_attr) = fill_to_svg_attr(&rect.fills) {
        attrs.push(create_jsx_attr(fill_attr.0, &fill_attr.1));
    }

    // Add stroke
    for (key, value) in stroke_to_svg_attrs(&rect.strokes, rect.stroke_weight) {
        attrs.push(create_jsx_attr(key, &value));
    }

    // Add opacity if present
    if let Some(opacity) = rect.opacity {
        if opacity < 1.0 {
            attrs.push(create_jsx_attr("opacity", &opacity.to_string()));
        }
    }

    // Add data attributes
    attrs.push(create_jsx_attr("data-name", &rect.name));
    attrs.push(create_jsx_attr("data-node-id", &rect.id));

    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "rect".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs,
            self_closing: true,
            type_args: None,
        },
        closing: None,
        children: vec![],
    }
}

/// Create JSX element for Ellipse as SVG <ellipse>
pub fn ellipse_to_svg_jsx(ellipse: &EllipseNode) -> JSXElement {
    let mut attrs = Vec::new();

    // Add center and radii
    if let Some(size) = &ellipse.size {
        let cx = size.x / 2.0;
        let cy = size.y / 2.0;
        let rx = size.x / 2.0;
        let ry = size.y / 2.0;

        attrs.push(create_jsx_attr("cx", &cx.to_string()));
        attrs.push(create_jsx_attr("cy", &cy.to_string()));
        attrs.push(create_jsx_attr("rx", &rx.to_string()));
        attrs.push(create_jsx_attr("ry", &ry.to_string()));
    }

    // Add fill
    if let Some(fill_attr) = fill_to_svg_attr(&ellipse.fills) {
        attrs.push(create_jsx_attr(fill_attr.0, &fill_attr.1));
    }

    // Add stroke
    for (key, value) in stroke_to_svg_attrs(&ellipse.strokes, ellipse.stroke_weight) {
        attrs.push(create_jsx_attr(key, &value));
    }

    // Add opacity
    if let Some(opacity) = ellipse.opacity {
        if opacity < 1.0 {
            attrs.push(create_jsx_attr("opacity", &opacity.to_string()));
        }
    }

    // Add data attributes
    attrs.push(create_jsx_attr("data-name", &ellipse.name));
    attrs.push(create_jsx_attr("data-node-id", &ellipse.id));

    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "ellipse".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs,
            self_closing: true,
            type_args: None,
        },
        closing: None,
        children: vec![],
    }
}

/// Create JSX element for Line as SVG <line>
pub fn line_to_svg_jsx(line: &LineNode) -> JSXElement {
    let mut attrs = Vec::new();

    // Lines in Figma have size representing the bounding box
    // Typically from (0, 0) to (width, height)
    if let Some(size) = &line.size {
        attrs.push(create_jsx_attr("x1", "0"));
        attrs.push(create_jsx_attr("y1", "0"));
        attrs.push(create_jsx_attr("x2", &size.x.to_string()));
        attrs.push(create_jsx_attr("y2", &size.y.to_string()));
    }

    // Lines use stroke, not fill
    for (key, value) in stroke_to_svg_attrs(&line.strokes, line.stroke_weight) {
        attrs.push(create_jsx_attr(key, &value));
    }

    // Add stroke cap if present
    if let Some(stroke_cap) = &line.stroke_cap {
        use figma_api::models::line_node::StrokeCap;
        let cap_value = match stroke_cap {
            StrokeCap::Round => "round",
            StrokeCap::Square => "square",
            _ => "butt",
        };
        attrs.push(create_jsx_attr("strokeLinecap", cap_value));
    }

    // Add opacity
    if let Some(opacity) = line.opacity {
        if opacity < 1.0 {
            attrs.push(create_jsx_attr("opacity", &opacity.to_string()));
        }
    }

    // Add data attributes
    attrs.push(create_jsx_attr("data-name", &line.name));
    attrs.push(create_jsx_attr("data-node-id", &line.id));

    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "line".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs,
            self_closing: true,
            type_args: None,
        },
        closing: None,
        children: vec![],
    }
}

/// Create JSX element for Star as SVG <path>  wrapped in SVG container
pub fn star_to_svg_jsx(star: &StarNode) -> JSXElement {
    let mut path_attrs = Vec::new();

    // Stars are best rendered using fill_geometry if available
    if let Some(fill_geometry) = &star.fill_geometry {
        if !fill_geometry.is_empty() {
            let path_data = &fill_geometry[0].path;
            path_attrs.push(create_jsx_attr("d", path_data));
        }
    }

    // Add fill
    if let Some(fill_attr) = fill_to_svg_attr(&star.fills) {
        path_attrs.push(create_jsx_attr(fill_attr.0, &fill_attr.1));
    }

    // Add stroke
    for (key, value) in stroke_to_svg_attrs(&star.strokes, star.stroke_weight) {
        path_attrs.push(create_jsx_attr(key, &value));
    }

    // Add opacity
    if let Some(opacity) = star.opacity {
        if opacity < 1.0 {
            path_attrs.push(create_jsx_attr("opacity", &opacity.to_string()));
        }
    }

    // Create the path element
    let path_element = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "path".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: path_attrs,
            self_closing: true,
            type_args: None,
        },
        closing: None,
        children: vec![],
    };

    // Wrap in SVG element with data attributes
    let mut svg_attrs = Vec::new();

    // Add size attributes if available
    if let Some(size) = &star.size {
        svg_attrs.push(create_jsx_attr("width", &size.x.to_string()));
        svg_attrs.push(create_jsx_attr("height", &size.y.to_string()));
        svg_attrs.push(create_jsx_attr(
            "viewBox",
            &format!("0 0 {} {}", size.x, size.y),
        ));
    }

    svg_attrs.push(create_jsx_attr("fill", "none"));
    svg_attrs.push(create_jsx_attr("xmlns", "http://www.w3.org/2000/svg"));
    svg_attrs.push(create_jsx_attr("data-name", &star.name));
    svg_attrs.push(create_jsx_attr("data-node-id", &star.id));

    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "svg".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: svg_attrs,
            self_closing: false,
            type_args: None,
        },
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "svg".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
        children: vec![JSXElementChild::JSXElement(Box::new(path_element))],
    }
}

/// Create JSX element for RegularPolygon as SVG <path> wrapped in SVG container
pub fn polygon_to_svg_jsx(polygon: &RegularPolygonNode) -> JSXElement {
    let mut path_attrs = Vec::new();

    // Use fill_geometry to get the polygon path, then convert to points
    // For simplicity, we'll use path instead
    if let Some(fill_geometry) = &polygon.fill_geometry {
        if !fill_geometry.is_empty() {
            let path_data = &fill_geometry[0].path;
            // For now, use path instead of polygon points
            // Converting path to points is complex
            path_attrs.push(create_jsx_attr("d", path_data));
        }
    }

    // Add fill
    if let Some(fill_attr) = fill_to_svg_attr(&polygon.fills) {
        path_attrs.push(create_jsx_attr(fill_attr.0, &fill_attr.1));
    }

    // Add stroke
    for (key, value) in stroke_to_svg_attrs(&polygon.strokes, polygon.stroke_weight) {
        path_attrs.push(create_jsx_attr(key, &value));
    }

    // Add opacity
    if let Some(opacity) = polygon.opacity {
        if opacity < 1.0 {
            path_attrs.push(create_jsx_attr("opacity", &opacity.to_string()));
        }
    }

    // Create the path element
    let path_element = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "path".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: path_attrs,
            self_closing: true,
            type_args: None,
        },
        closing: None,
        children: vec![],
    };

    // Wrap in SVG element with data attributes
    let mut svg_attrs = Vec::new();

    // Add size attributes if available
    if let Some(size) = &polygon.size {
        svg_attrs.push(create_jsx_attr("width", &size.x.to_string()));
        svg_attrs.push(create_jsx_attr("height", &size.y.to_string()));
        svg_attrs.push(create_jsx_attr(
            "viewBox",
            &format!("0 0 {} {}", size.x, size.y),
        ));
    }

    svg_attrs.push(create_jsx_attr("fill", "none"));
    svg_attrs.push(create_jsx_attr("xmlns", "http://www.w3.org/2000/svg"));
    svg_attrs.push(create_jsx_attr("data-name", &polygon.name));
    svg_attrs.push(create_jsx_attr("data-node-id", &polygon.id));

    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "svg".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: svg_attrs,
            self_closing: false,
            type_args: None,
        },
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "svg".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
        children: vec![JSXElementChild::JSXElement(Box::new(path_element))],
    }
}

/// Helper to create a JSX attribute
fn create_jsx_attr(name: &str, value: &str) -> JSXAttrOrSpread {
    JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(IdentName {
            span: DUMMY_SP,
            sym: name.into(),
        }),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: value.into(),
            raw: None,
        }))),
    })
}

// ToJsx implementations for shape nodes
impl ToJsx for RectangleNode {
    fn to_jsx_with_context(&self, context: RenderContext) -> JSXElement {
        match context {
            RenderContext::Html => {
                let svg_element = rectangle_to_svg_jsx(self);
                if let Some((x, y)) = extract_position_from_transform(&self.relative_transform) {
                    wrap_svg_in_positioned_div(svg_element, x, y, &self.size)
                } else {
                    svg_element
                }
            }
            RenderContext::Svg { parent_x, parent_y } => {
                // For SVG context, just wrap in <g> without bounding box
                wrap_svg_element_in_group(rectangle_to_svg_jsx(self), None, parent_x, parent_y)
            }
        }
    }
}

impl ToJsx for EllipseNode {
    fn to_jsx_with_context(&self, context: RenderContext) -> JSXElement {
        match context {
            RenderContext::Html => {
                let svg_element = ellipse_to_svg_jsx(self);
                if let Some((x, y)) = extract_position_from_transform(&self.relative_transform) {
                    wrap_svg_in_positioned_div(svg_element, x, y, &self.size)
                } else {
                    svg_element
                }
            }
            RenderContext::Svg { parent_x, parent_y } => {
                // For SVG context, just wrap in <g> without bounding box
                wrap_svg_element_in_group(ellipse_to_svg_jsx(self), None, parent_x, parent_y)
            }
        }
    }
}

impl ToJsx for LineNode {
    fn to_jsx_with_context(&self, context: RenderContext) -> JSXElement {
        match context {
            RenderContext::Html => {
                let svg_element = line_to_svg_jsx(self);
                if let Some((x, y)) = extract_position_from_transform(&self.relative_transform) {
                    wrap_svg_in_positioned_div(svg_element, x, y, &self.size)
                } else {
                    svg_element
                }
            }
            RenderContext::Svg { parent_x, parent_y } => {
                // For SVG context, just wrap in <g> without bounding box
                wrap_svg_element_in_group(line_to_svg_jsx(self), None, parent_x, parent_y)
            }
        }
    }
}

impl ToJsx for StarNode {
    fn to_jsx_with_context(&self, context: RenderContext) -> JSXElement {
        match context {
            RenderContext::Html => {
                let svg_element = star_to_svg_jsx(self);
                if let Some((x, y)) = extract_position_from_transform(&self.relative_transform) {
                    wrap_svg_in_positioned_div(svg_element, x, y, &self.size)
                } else {
                    svg_element
                }
            }
            RenderContext::Svg { parent_x, parent_y } => {
                // For SVG context, just wrap in <g> without bounding box
                wrap_svg_element_in_group(star_to_svg_jsx(self), None, parent_x, parent_y)
            }
        }
    }
}

impl ToJsx for RegularPolygonNode {
    fn to_jsx_with_context(&self, context: RenderContext) -> JSXElement {
        match context {
            RenderContext::Html => {
                let svg_element = polygon_to_svg_jsx(self);
                if let Some((x, y)) = extract_position_from_transform(&self.relative_transform) {
                    wrap_svg_in_positioned_div(svg_element, x, y, &self.size)
                } else {
                    svg_element
                }
            }
            RenderContext::Svg { parent_x, parent_y } => {
                // For SVG context, just wrap in <g> without bounding box
                wrap_svg_element_in_group(polygon_to_svg_jsx(self), None, parent_x, parent_y)
            }
        }
    }
}
