use serde::{Deserialize, Serialize};
use swc_atoms::Atom;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct SheetOptions {
    pub id: String,
    pub trigger_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub side: Option<String>, // "top", "right", "bottom", "left"
    pub content: Option<String>,
    pub has_footer: Option<bool>,
    pub footer_buttons: Option<Vec<SheetButton>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct SheetButton {
    pub label: String,
    pub variant: Option<String>,
    pub action: Option<String>,
}

pub fn create_sheet_component(trigger_element: JSXElement, options: &SheetOptions) -> JSXElement {
    let mut sheet_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Sheet".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        },
        children: vec![],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Sheet".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    let trigger_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "SheetTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "asChild".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }
                .into()),
                value: None,
            })],
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "SheetTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    let mut content_attrs = vec![];
    if let Some(side) = &options.side {
        content_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "side".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }
            .into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: side.clone().into(),
                raw: None,
            }))),
        }));
    }

    let mut content_children = Vec::new();

    if options.title.is_some() || options.description.is_some() {
        let mut header_children = Vec::new();
        if let Some(title) = &options.title {
            header_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "SheetTitle".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: vec![],
                    self_closing: false,
                    type_args: None,
                },
                children: vec![JSXElementChild::JSXText(JSXText {
                    span: DUMMY_SP,
                    value: title.clone().into(),
                    raw: Atom::default(),
                })],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "SheetTitle".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                }),
            })));
        }
        if let Some(description) = &options.description {
            header_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "SheetDescription".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: vec![],
                    self_closing: false,
                    type_args: None,
                },
                children: vec![JSXElementChild::JSXText(JSXText {
                    span: DUMMY_SP,
                    value: description.clone().into(),
                    raw: Atom::default(),
                })],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "SheetDescription".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                }),
            })));
        }
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "SheetHeader".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![],
                self_closing: false,
                type_args: None,
            },
            children: header_children,
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "SheetHeader".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        })));
    }

    if let Some(content) = &options.content {
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "div".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "dangerouslySetInnerHTML".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into()),
                    value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                        span: DUMMY_SP,
                        expr: JSXExpr::Expr(Box::new(Expr::Object(ObjectLit {
                            span: DUMMY_SP,
                            props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                KeyValueProp {
                                    key: PropName::Ident(Ident {
                                        span: DUMMY_SP,
                                        sym: "__html".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into()),
                                    value: Box::new(Expr::Lit(Lit::Str(Str {
                                        span: DUMMY_SP,
                                        value: content.clone().into(),
                                        raw: None,
                                    }))),
                                },
                            )))],
                        }))),
                    })),
                })],
                self_closing: false,
                type_args: None,
            },
            children: vec![],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "div".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        })));
    }

    let content_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "SheetContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: content_attrs,
            self_closing: false,
            type_args: None,
        },
        children: content_children,
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "SheetContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    sheet_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(trigger_jsx)));
    sheet_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(content_jsx)));

    sheet_jsx
}


