use crate::conversion::jsx::{RenderContext, ToJsx};
use crate::extensions::tailwind::TailwindStyleExt;
use figma_api::models::FrameNode;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::{
    Expr, Ident, IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement,
    JSXElementName, JSXExpr, JSXExprContainer, JSXOpeningElement, KeyValueProp, Lit, ObjectLit,
    Prop, PropName, PropOrSpread, Str,
};
use swc_ecma_ast::{JSXElement, JSXElementChild};

impl ToJsx for FrameNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        // Frames are always rendered in HTML context (they're containers, not SVG primitives)
        // Get separated class sets for coordinate system separation
        let positioning_classes = self.positioning_classes();
        let layout_content_classes = self.layout_and_content_classes();
        let styles = self.to_tailwind();

        // Always create nested structure to cleanly separate coordinate systems
        // Outer div: handles positioning relative to parent
        // Inner div: handles layout for children and content styling

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

        // Add inline styles to inner div if present
        if !styles.inline_styles.is_empty() {
            let style_props: Vec<PropOrSpread> = styles
                .inline_styles
                .iter()
                .map(|(key, value)| {
                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Str(Str {
                            span: DUMMY_SP,
                            value: key.clone().into(),
                            raw: None,
                        }),
                        value: Box::new(Expr::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: value.clone().into(),
                            raw: None,
                        }))),
                    })))
                })
                .collect();

            inner_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "style".into(),
                }),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Object(ObjectLit {
                        span: DUMMY_SP,
                        props: style_props,
                    }))),
                })),
            }));
        }

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

        // Add data-node-id attribute to outer div for debugging/reference
        outer_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "data-node-id".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: self.id.clone().into(),
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
