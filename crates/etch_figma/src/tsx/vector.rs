use crate::tsx::svg_strategy::PathComplexity;
use crate::tsx::{ToJsx, figma_svg_export::FigmaSvgExporter};
use figma_api::models::VectorNode;
use swc_atoms::Atom;
use swc_ecma_ast::JSXElement;

/// Enhanced vector node conversion with SVG export support
impl ToJsx for VectorNode {
    fn to_jsx(&self) -> JSXElement {
        // Determine strategy: inline simple paths, external for complex
        if let Some(fill_geometry) = &self.fill_geometry {
            if !fill_geometry.is_empty() {
                let path_data = &fill_geometry[0].path;
                let complexity = PathComplexity::from_path_data(path_data);

                match complexity {
                    PathComplexity::Simple => {
                        // Inline simple paths directly
                        create_inline_vector_jsx(self, path_data)
                    }
                    PathComplexity::Complex | PathComplexity::VeryComplex => {
                        // For complex paths, we'll need external reference
                        // For now, still inline but mark for future optimization
                        create_inline_vector_jsx(self, path_data)
                    }
                }
            } else {
                create_vector_jsx_placeholder(self)
            }
        } else {
            // No geometry available, create placeholder
            create_vector_jsx_placeholder(self)
        }
    }
}

/// Creates a JSX element for a vector node with SVG export
pub async fn create_vector_jsx_with_svg_export(
    vector: &VectorNode,
    exporter: &mut FigmaSvgExporter,
    file_key: &str,
) -> Result<JSXElement, crate::tsx::figma_svg_export::SvgExportError> {
    use swc_common::{DUMMY_SP, SyntaxContext};
    use swc_ecma_ast::{
        Ident, IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement,
        JSXElementChild, JSXElementName, JSXOpeningElement, Lit, Str,
    };

    // Export the vector as SVG
    let svg_export = exporter.export_vector_node(vector, file_key).await?;

    let mut attrs = Vec::new();

    // Add data attributes
    attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(IdentName {
            span: DUMMY_SP,
            sym: "data-name".into(),
        }),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: vector.name.clone().into(),
            raw: None,
        }))),
    }));

    attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(IdentName {
            span: DUMMY_SP,
            sym: "data-vector".into(),
        }),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: "true".into(),
            raw: None,
        }))),
    }));

    // Add viewBox if available
    if let Some((x, y, width, height)) = svg_export.viewbox {
        attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "viewBox".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: format!("{} {} {} {}", x, y, width, height).into(),
                raw: None,
            }))),
        }));
    }

    // Handle external vs inline content
    let children = if svg_export.is_external {
        // Create a reference to external SVG
        vec![JSXElementChild::JSXText(swc_ecma_ast::JSXText {
            span: DUMMY_SP,
            value: format!(
                "<!-- External SVG: {} -->",
                svg_export
                    .external_path
                    .as_ref()
                    .unwrap_or(&"unknown".to_string())
            )
            .into(),
            raw: Atom::new(""),
        })]
    } else {
        // Parse and include SVG content inline
        parse_svg_content_to_jsx(&svg_export.svg_content)
    };

    Ok(JSXElement {
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
    })
}

/// Creates an inline SVG element with path data directly embedded
fn create_inline_vector_jsx(vector: &VectorNode, path_data: &str) -> JSXElement {
    use crate::tsx::paint::{fill_to_svg_attr, stroke_to_svg_attrs};
    use swc_common::{DUMMY_SP, SyntaxContext};
    use swc_ecma_ast::{
        Ident, JSXClosingElement, JSXElementChild, JSXElementName, JSXOpeningElement,
    };

    let mut svg_attrs = Vec::new();

    // Add size attributes if available
    if let Some(size) = &vector.size {
        svg_attrs.push(create_vector_attr("width", &size.x.to_string()));
        svg_attrs.push(create_vector_attr("height", &size.y.to_string()));
        svg_attrs.push(create_vector_attr(
            "viewBox",
            &format!("0 0 {} {}", size.x, size.y),
        ));
    }

    // Add data attributes
    svg_attrs.push(create_vector_attr("data-name", &vector.name));
    svg_attrs.push(create_vector_attr("data-node-id", &vector.id));
    svg_attrs.push(create_vector_attr("data-vector", "true"));

    // Create path element
    let mut path_attrs = vec![create_vector_attr("d", path_data)];

    // Add fill
    if let Some(fill_attr) = fill_to_svg_attr(&vector.fills) {
        path_attrs.push(create_vector_attr(fill_attr.0, &fill_attr.1));
    } else {
        path_attrs.push(create_vector_attr("fill", "currentColor"));
    }

    // Add stroke
    for (key, value) in stroke_to_svg_attrs(&vector.strokes, vector.stroke_weight) {
        path_attrs.push(create_vector_attr(key, &value));
    }

    // Add opacity to path if present
    if let Some(opacity) = vector.opacity {
        if opacity < 1.0 {
            path_attrs.push(create_vector_attr("opacity", &opacity.to_string()));
        }
    }

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

    // Wrap in SVG element
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

/// Creates a placeholder JSX element for vector nodes
fn create_vector_jsx_placeholder(vector: &VectorNode) -> JSXElement {
    use swc_common::{DUMMY_SP, SyntaxContext};
    use swc_ecma_ast::{
        Ident, IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement,
        JSXElementName, JSXOpeningElement, Lit, Str,
    };

    let mut attrs = Vec::new();

    // Add data attributes
    attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(IdentName {
            span: DUMMY_SP,
            sym: "data-name".into(),
        }),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: vector.name.clone().into(),
            raw: None,
        }))),
    }));

    attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(IdentName {
            span: DUMMY_SP,
            sym: "data-vector".into(),
        }),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: "true".into(),
            raw: None,
        }))),
    }));

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
        children: vec![], // Will be populated with actual SVG content
    }
}

/// Parses SVG content and converts it to JSX children
fn parse_svg_content_to_jsx(_svg_content: &str) -> Vec<swc_ecma_ast::JSXElementChild> {
    // This is a simplified implementation
    // In practice, you'd want to parse the SVG content and convert each element
    // to appropriate JSX elements

    // For now, return a placeholder
    vec![swc_ecma_ast::JSXElementChild::JSXText(
        swc_ecma_ast::JSXText {
            span: swc_common::DUMMY_SP,
            value: "<!-- SVG content would be parsed here -->".into(),
            raw: Atom::new(""),
        },
    )]
}

/// Helper to create a JSX attribute for vector elements
fn create_vector_attr(name: &str, value: &str) -> swc_ecma_ast::JSXAttrOrSpread {
    use swc_common::DUMMY_SP;
    use swc_ecma_ast::{IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, Lit, Str};

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
