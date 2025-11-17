use crate::conversion::jsx::{RenderContext, ToJsx};
use crate::extensions::tailwind::TailwindStyleExt;
use figma_api::models::GroupNode;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::{
    Ident, IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement,
    JSXElementName, JSXOpeningElement, Lit, Str,
};
use swc_ecma_ast::{JSXElement, JSXElementChild};

impl ToJsx for GroupNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        // Groups are always rendered in HTML context (they're containers, not SVG primitives)
        // Get separated class sets for coordinate system separation
        let positioning_classes = self.positioning_classes();
        let layout_content_classes = self.layout_and_content_classes();

        // Always create nested structure to cleanly separate coordinate systems
        // Outer div: handles positioning relative to parent
        // Inner div: handles layout for children (relative coordinate system)

        // Create inner div with layout and content classes
        let mut inner_attrs = Vec::new();

        if !layout_content_classes.is_empty() {
            inner_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "className".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: layout_content_classes.join(" ").into(),
                    raw: None,
                }))),
            }));
        }

        // Add data-group marker to inner div
        inner_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "data-group".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: "true".into(),
                raw: None,
            }))),
        }));

        let inner_div = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "div".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: inner_attrs,
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
            children: vec![], // Children will be populated by the walker
        };

        // Create outer div with positioning classes and data attributes
        let mut outer_attrs = Vec::new();

        if !positioning_classes.is_empty() {
            outer_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "className".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: positioning_classes.join(" ").into(),
                    raw: None,
                }))),
            }));
        }

        // Add data-name attribute to outer div
        outer_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
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
                attrs: outer_attrs,
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
            children: vec![JSXElementChild::JSXElement(Box::new(inner_div))],
        }
    }
}
