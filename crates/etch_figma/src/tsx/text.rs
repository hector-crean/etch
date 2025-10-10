use figma_api::models::TextNode;
use swc_ecma_ast::{JSXElement, JSXElementChild, JSXText, JSXOpeningElement, JSXClosingElement, JSXElementName, JSXAttr, JSXAttrName, JSXAttrValue, JSXAttrOrSpread, IdentName, Ident, Str, Lit,};
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_atoms::Atom;


/// Extension trait for TextNode to provide JSX conversion
pub trait TextNodeExt {
    fn to_jsx(&self) -> JSXElement;
}

impl TextNodeExt for TextNode {
    fn to_jsx(&self) -> JSXElement {
        let mut attrs = Vec::new();
        
        // Add data-name attribute
        attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "data-name".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: self.name.clone().into(),
                raw: None,
            }))),
        }));
        
        // Add data-text attribute
        attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "data-text".into(),
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
                    sym: "p".into(),
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
                    sym: "p".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
            children: vec![JSXElementChild::JSXText(JSXText {
                span: DUMMY_SP,
                value: self.characters.clone().into(),
                raw: Atom::new(""),
            })],
        }
    }
}
