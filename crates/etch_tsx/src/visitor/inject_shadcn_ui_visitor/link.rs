use serde::{Deserialize, Serialize};
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum RoutingLibrary {
    NextJs,
    Wouter,
    ReactRouter,
    Native, // Standard HTML <a> tag
}

impl Default for RoutingLibrary {
    fn default() -> Self {
        RoutingLibrary::NextJs
    }
}

// Helper function to check if a boolean is false (for skipping serialization)
fn is_false(b: &bool) -> bool {
    !b
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct LinkOptions {
    pub id: String,
    pub href: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,
    
    #[serde(default, skip_serializing_if = "is_false")]
    pub as_button: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing_library: Option<RoutingLibrary>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_name: Option<String>,
    
    #[serde(default, skip_serializing_if = "is_false")]
    pub replace: bool,
    
    #[serde(default, skip_serializing_if = "is_false")]
    pub prefetch: bool, // For Next.js Link prefetch
}

impl Default for LinkOptions {
    fn default() -> Self {
        Self {
            id: String::new(),
            href: String::new(),
            target: None,
            rel: None,
            as_button: false,
            variant: None,
            size: None,
            routing_library: None,
            class_name: None,
            replace: false,
            prefetch: false,
        }
    }
}

pub fn create_link_component(trigger_element: JSXElement, options: &LinkOptions) -> JSXElement {
    let routing_lib = options.routing_library.as_ref().unwrap_or(&RoutingLibrary::NextJs);
    
    match routing_lib {
        RoutingLibrary::NextJs => create_nextjs_link(trigger_element, options),
        RoutingLibrary::Wouter => create_wouter_link(trigger_element, options),
        RoutingLibrary::ReactRouter => create_react_router_link(trigger_element, options),
        RoutingLibrary::Native => create_native_link(trigger_element, options),
    }
}

fn create_nextjs_link(trigger_element: JSXElement, options: &LinkOptions) -> JSXElement {
    let mut link_attrs: Vec<JSXAttrOrSpread> = Vec::new();
    
    // Add href attribute
    link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident {
            span: DUMMY_SP,
            sym: "href".into(),
            optional: false,
            ctxt: SyntaxContext::empty(),
        }.into()),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: options.href.clone().into(),
            raw: None,
        }))),
    }));
    
    // Add prefetch attribute if true
    if options.prefetch {
        link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "prefetch".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                span: DUMMY_SP,
                expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Bool(Bool {
                    span: DUMMY_SP,
                    value: true,
                })))),
            })),
        }));
    }
    
    // Add replace attribute if true
    if options.replace {
        link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "replace".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                span: DUMMY_SP,
                expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Bool(Bool {
                    span: DUMMY_SP,
                    value: true,
                })))),
            })),
        }));
    }
    
    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Link".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: link_attrs,
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Link".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    }
}

fn create_wouter_link(trigger_element: JSXElement, options: &LinkOptions) -> JSXElement {
    let mut link_attrs: Vec<JSXAttrOrSpread> = Vec::new();
    
    // Add href attribute (Wouter uses 'href' for the path)
    link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident {
            span: DUMMY_SP,
            sym: "href".into(),
            optional: false,
            ctxt: SyntaxContext::empty(),
        }.into()),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: options.href.clone().into(),
            raw: None,
        }))),
    }));
    
    // Add replace attribute if true
    if options.replace {
        link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "replace".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                span: DUMMY_SP,
                expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Bool(Bool {
                    span: DUMMY_SP,
                    value: true,
                })))),
            })),
        }));
    }
    
    // Add className if specified
    if let Some(class_name) = &options.class_name {
        link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "className".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: class_name.clone().into(),
                raw: None,
            }))),
        }));
    }
    
    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Link".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: link_attrs,
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Link".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    }
}

fn create_react_router_link(trigger_element: JSXElement, options: &LinkOptions) -> JSXElement {
    let mut link_attrs: Vec<JSXAttrOrSpread> = Vec::new();
    
    // Add to attribute (React Router uses 'to' instead of 'href')
    link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident {
            span: DUMMY_SP,
            sym: "to".into(),
            optional: false,
            ctxt: SyntaxContext::empty(),
        }.into()),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: options.href.clone().into(),
            raw: None,
        }))),
    }));
    
    // Add replace attribute if true
    if options.replace {
        link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "replace".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                span: DUMMY_SP,
                expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Bool(Bool {
                    span: DUMMY_SP,
                    value: true,
                })))),
            })),
        }));
    }
    
    // Add className if specified
    if let Some(class_name) = &options.class_name {
        link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "className".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: class_name.clone().into(),
                raw: None,
            }))),
        }));
    }
    
    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Link".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: link_attrs,
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Link".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    }
}

fn create_native_link(trigger_element: JSXElement, options: &LinkOptions) -> JSXElement {
    let mut link_attrs: Vec<JSXAttrOrSpread> = Vec::new();
    
    // Add href attribute
    link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident {
            span: DUMMY_SP,
            sym: "href".into(),
            optional: false,
            ctxt: SyntaxContext::empty(),
        }.into()),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: options.href.clone().into(),
            raw: None,
        }))),
    }));
    
    // Add target attribute if specified
    if let Some(target) = &options.target {
        link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "target".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: target.clone().into(),
                raw: None,
            }))),
        }));
    }
    
    // Add rel attribute if specified
    if let Some(rel) = &options.rel {
        link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "rel".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: rel.clone().into(),
                raw: None,
            }))),
        }));
    }
    
    // Add className if specified
    if let Some(class_name) = &options.class_name {
        link_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "className".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: class_name.clone().into(),
                raw: None,
            }))),
        }));
    }
    
    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "a".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: link_attrs,
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "a".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    }
}


