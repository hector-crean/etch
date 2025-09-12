use serde::{Deserialize, Serialize};
use swc_atoms::Atom;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct PopoverOptions {
    pub id: String,
    pub trigger_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub alignment: Option<String>, // "start", "center", "end"
}

pub fn create_popover_component(trigger_element: JSXElement, options: &PopoverOptions) -> JSXElement {
    let mut popover_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Popover".into(),
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
                sym: "Popover".into(),
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
                sym: "PopoverTrigger".into(),
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
                sym: "PopoverTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    let mut content_children = Vec::new();

    if let Some(title) = &options.title {
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
                        sym: "className".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into()),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: "font-medium pb-2".into(),
                        raw: None,
                    }))),
                })],
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
                    sym: "div".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        })));
    }

    if let Some(description) = &options.description {
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "p".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "className".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into()),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: "text-sm text-muted-foreground".into(),
                        raw: None,
                    }))),
                })],
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
                    sym: "p".into(),
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

    let mut content_attrs = vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident {
            span: DUMMY_SP,
            sym: "className".into(),
            optional: false,
            ctxt: SyntaxContext::empty(),
        }
        .into()),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: "w-80 p-4".into(),
            raw: None,
        }))),
    })];

    if let Some(alignment) = &options.alignment {
        content_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "align".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }
            .into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: alignment.clone().into(),
                raw: None,
            }))),
        }));
    }

    let content_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "PopoverContent".into(),
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
                sym: "PopoverContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    popover_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(trigger_jsx)));
    popover_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(content_jsx)));

    popover_jsx
}


