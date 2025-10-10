use figma_api::models::VectorNode;
use swc_atoms::Atom;
use swc_ecma_ast::JSXElement;
use crate::tsx::{ToJsx, figma_svg_export::FigmaSvgExporter};

/// Enhanced vector node conversion with SVG export support
impl ToJsx for VectorNode {
    fn to_jsx(&self) -> JSXElement {
        // This is a placeholder - in practice, this would be called
        // from the visitor with access to the SVG exporter
        create_vector_jsx_placeholder(self)
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
        JSXOpeningElement, JSXClosingElement, JSXElementName, JSXAttr, JSXAttrName,
        JSXAttrValue, JSXAttrOrSpread, IdentName, Ident, Str, Lit, JSXElementChild
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
            value: format!("<!-- External SVG: {} -->", 
                svg_export.external_path.as_ref().unwrap_or(&"unknown".to_string())).into(),
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

/// Creates a placeholder JSX element for vector nodes
fn create_vector_jsx_placeholder(vector: &VectorNode) -> JSXElement {
    use swc_common::{DUMMY_SP, SyntaxContext};
    use swc_ecma_ast::{
        JSXOpeningElement, JSXClosingElement, JSXElementName, JSXAttr, JSXAttrName,
        JSXAttrValue, JSXAttrOrSpread, IdentName, Ident, Str, Lit
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
fn parse_svg_content_to_jsx(svg_content: &str) -> Vec<swc_ecma_ast::JSXElementChild> {
    // This is a simplified implementation
    // In practice, you'd want to parse the SVG content and convert each element
    // to appropriate JSX elements
    
    // For now, return a placeholder
    vec![swc_ecma_ast::JSXElementChild::JSXText(swc_ecma_ast::JSXText {
        span: swc_common::DUMMY_SP,
        value: "<!-- SVG content would be parsed here -->".into(),
        raw: Atom::new(""),
    })]
}
