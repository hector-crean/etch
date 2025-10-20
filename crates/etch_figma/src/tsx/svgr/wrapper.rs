use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::{
    Ident, IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement,
    JSXElement, JSXElementChild, JSXElementName, JSXOpeningElement, Lit, Str,
};
use crate::analyzers::svg_container::SvgWrapperConfig;

/// Creates an SVG wrapper element with responsive configuration
pub fn create_svg_wrapper(
    children: Vec<JSXElementChild>,
    config: &SvgWrapperConfig,
) -> JSXElement {
    let mut attrs = Vec::new();

    // Add responsive attributes
    if config.responsive {
        attrs.push(create_attr("width", "100%"));
        attrs.push(create_attr("height", "100%"));
    }

    // Add viewBox if specified
    if let Some(viewbox) = &config.viewbox {
        let viewbox_str = format!("{} {} {} {}", viewbox.0, viewbox.1, viewbox.2, viewbox.3);
        attrs.push(create_attr("viewBox", &viewbox_str));
    }

    // Add preserveAspectRatio
    attrs.push(create_attr("preserveAspectRatio", &config.preserve_aspect_ratio));

    // Add fill and stroke defaults
    attrs.push(create_attr("fill", "none"));
    attrs.push(create_attr("stroke", "none"));

    // Add data attributes for debugging
    attrs.push(create_attr("data-svg-wrapper", "true"));

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
            attrs,
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
        children,
    }
}

/// Creates a responsive SVG wrapper with calculated viewBox
pub fn create_responsive_svg_wrapper(
    children: Vec<JSXElementChild>,
    bounds: Option<(f64, f64, f64, f64)>,
) -> JSXElement {
    let config = SvgWrapperConfig {
        responsive: true,
        viewbox: bounds,
        preserve_aspect_ratio: "xMidYMid meet".to_string(),
    };

    create_svg_wrapper(children, &config)
}

/// Creates a fixed-size SVG wrapper
pub fn create_fixed_svg_wrapper(
    children: Vec<JSXElementChild>,
    width: f64,
    height: f64,
) -> JSXElement {
    let mut attrs = Vec::new();

    // Add fixed dimensions
    attrs.push(create_attr("width", &width.to_string()));
    attrs.push(create_attr("height", &height.to_string()));
    attrs.push(create_attr("viewBox", &format!("0 0 {} {}", width, height)));

    // Add other standard attributes
    attrs.push(create_attr("fill", "none"));
    attrs.push(create_attr("stroke", "none"));
    attrs.push(create_attr("data-svg-wrapper", "true"));

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
            attrs,
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
        children,
    }
}

/// Creates a mixed container with SVG wrapper and HTML overlay
pub fn create_mixed_container(
    svg_children: Vec<JSXElementChild>,
    html_children: Vec<JSXElementChild>,
    config: &SvgWrapperConfig,
) -> JSXElement {
    let mut container_attrs = Vec::new();
    container_attrs.push(create_attr("className", "relative"));
    container_attrs.push(create_attr("data-mixed-container", "true"));

    // Create SVG wrapper
    let svg_wrapper = create_svg_wrapper(svg_children, config);

    // Create HTML overlay container
    let mut overlay_attrs = Vec::new();
    overlay_attrs.push(create_attr("className", "absolute inset-0 pointer-events-none"));
    overlay_attrs.push(create_attr("data-html-overlay", "true"));

    let html_overlay = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "div".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: overlay_attrs,
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
        children: html_children,
    };

    // Combine SVG and HTML overlay
    let all_children = vec![
        JSXElementChild::JSXElement(Box::new(svg_wrapper)),
        JSXElementChild::JSXElement(Box::new(html_overlay)),
    ];

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
            attrs: container_attrs,
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
        children: all_children,
    }
}

/// Helper to create a JSX attribute
fn create_attr(name: &str, value: &str) -> JSXAttrOrSpread {
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

/// Calculate bounds from a list of JSX children (placeholder implementation)
pub fn calculate_bounds_from_children(_children: &[JSXElementChild]) -> Option<(f64, f64, f64, f64)> {
    // This would analyze the children to determine the bounding box
    // For now, return a default viewBox
    Some((0.0, 0.0, 100.0, 100.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_ecma_ast::JSXText;

    #[test]
    fn test_create_svg_wrapper() {
        let children = vec![JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: "test".into(),
            raw: "test".into(),
        })];

        let config = SvgWrapperConfig {
            responsive: true,
            viewbox: Some((0.0, 0.0, 100.0, 100.0)),
            preserve_aspect_ratio: "xMidYMid meet".to_string(),
        };

        let svg = create_svg_wrapper(children, &config);

        // Check that it's an SVG element
        match &svg.opening.name {
            JSXElementName::Ident(ident) => {
                assert_eq!(ident.sym.as_ref(), "svg");
            }
            _ => panic!("Expected SVG element"),
        }

        // Check attributes
        let attrs = &svg.opening.attrs;
        assert!(attrs.len() >= 5); // width, height, viewBox, preserveAspectRatio, fill, stroke, data-svg-wrapper
    }

    #[test]
    fn test_create_responsive_svg_wrapper() {
        let children = vec![JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: "test".into(),
            raw: "test".into(),
        })];

        let bounds = Some((10.0, 20.0, 200.0, 150.0));
        let svg = create_responsive_svg_wrapper(children, bounds);

        // Should have responsive attributes
        let attrs = &svg.opening.attrs;
        assert!(attrs.len() >= 5);
    }

    #[test]
    fn test_create_fixed_svg_wrapper() {
        let children = vec![JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: "test".into(),
            raw: "test".into(),
        })];

        let svg = create_fixed_svg_wrapper(children, 200.0, 150.0);

        // Should have fixed dimensions
        let attrs = &svg.opening.attrs;
        assert!(attrs.len() >= 5);
    }

    #[test]
    fn test_create_mixed_container() {
        let svg_children = vec![JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: "svg".into(),
            raw: "svg".into(),
        })];

        let html_children = vec![JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: "html".into(),
            raw: "html".into(),
        })];

        let config = SvgWrapperConfig {
            responsive: true,
            viewbox: Some((0.0, 0.0, 100.0, 100.0)),
            preserve_aspect_ratio: "xMidYMid meet".to_string(),
        };

        let container = create_mixed_container(svg_children, html_children, &config);

        // Should be a div container
        match &container.opening.name {
            JSXElementName::Ident(ident) => {
                assert_eq!(ident.sym.as_ref(), "div");
            }
            _ => panic!("Expected div element"),
        }

        // Should have two children (SVG + HTML overlay)
        assert_eq!(container.children.len(), 2);
    }
}
