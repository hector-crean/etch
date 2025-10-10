pub mod visitor;
pub mod generator;
pub mod text;
pub mod vector;
pub mod frame;
pub mod svg_strategy;
pub mod figma_svg_export;


use figma_api::models::{
    FrameNode, TextNode, RectangleNode, 
    GroupNode
};
use swc_ecma_ast::JSXElement;



/// JSX-specific conversion traits
pub trait ToJsx {
    fn to_jsx(&self) -> JSXElement;
}


// JSX implementations for major node types
impl ToJsx for FrameNode {
    fn to_jsx(&self) -> JSXElement {
        use crate::tailwind_ext::TailwindStyleExt;
        use swc_common::{DUMMY_SP, SyntaxContext};
        use swc_ecma_ast::{
            JSXOpeningElement, JSXClosingElement, JSXElementName, JSXAttr, JSXAttrName,
            JSXAttrValue, JSXExpr, JSXExprContainer, JSXAttrOrSpread, IdentName, Ident, Str, Expr, Lit, 
            ObjectLit, PropOrSpread, Prop, KeyValueProp, PropName
        };

        let styles = self.to_tailwind();
        let mut attrs = Vec::new();
        
        // Add className attribute
        if !styles.classes.is_empty() {
            attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "className".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: styles.class_string().into(),
                    raw: None,
                }))),
            }));
        }
        
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
        
        // Add data-node-id attribute for debugging/reference
        attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
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
        
        // Add style attribute if there are inline styles
        if !styles.inline_styles.is_empty() {
            let style_props: Vec<PropOrSpread> = styles.inline_styles.iter()
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
                
            attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
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
            children: vec![], // Children will be populated by the walker
        }
    }
}

impl ToJsx for TextNode {
    fn to_jsx(&self) -> JSXElement {
        // Use the text module's conversion
        use crate::tsx::text::TextNodeExt;
        TextNodeExt::to_jsx(self)
    }
}

// VectorNode ToJsx implementation moved to vector.rs to avoid conflicts

impl ToJsx for RectangleNode {
    fn to_jsx(&self) -> JSXElement {
        // Create a simple rectangle SVG element
        use swc_common::{DUMMY_SP, SyntaxContext};
        use swc_ecma_ast::{
            JSXOpeningElement, JSXClosingElement, JSXElementName, JSXAttr, JSXAttrName,
            JSXAttrValue, JSXAttrOrSpread, IdentName, Ident, Str, Lit
        };

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
        
        // Add data-rectangle attribute
        attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "data-rectangle".into(),
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
}

impl ToJsx for GroupNode {
    fn to_jsx(&self) -> JSXElement {
        use swc_common::{DUMMY_SP, SyntaxContext};
        use swc_ecma_ast::{
            JSXOpeningElement, JSXClosingElement, JSXElementName, JSXAttr, JSXAttrName,
            JSXAttrValue, JSXAttrOrSpread, IdentName, Ident, Str, Lit
        };

        let mut attrs = Vec::new();
        
        // Groups are typically containers
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
        
        attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
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
            children: vec![], // Children will be populated by the walker
        }
    }
}

