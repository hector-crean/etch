use super::dangerously_set_node::dangerous_html_node;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use swc_atoms::Atom;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use ts_rs::TS;
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct DialogButton {
    pub label: String,
    pub variant: Option<String>, // "default", "destructive", "outline", etc.
    pub action: Option<String>,  // Function to call when clicked
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, TS)]
#[ts(export)]
pub struct DialogOptions {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, String>>,
}

impl DialogOptions {
    /// Creates a Dialog component with the original element as the trigger
    pub fn jsx_element(&self, trigger_element: JSXElement) -> JSXElement {
        // 1. Create Dialog root element
        let mut dialog_jsx = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "Dialog".into(),
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
                    sym: "Dialog".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        };

        // 2. Create DialogTrigger with asChild prop
        let trigger_jsx = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "DialogTrigger".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(
                        Ident {
                            span: DUMMY_SP,
                            sym: "asChild".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }
                        .into(),
                    ),
                    value: None, // Boolean attribute with no value
                })],
                self_closing: false,
                type_args: None,
            },
            children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "DialogTrigger".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        };

        // 3. Create DialogContent
        let mut content_children = Vec::new();

        // Add DialogHeader with Title and Description
        if self.title.is_some() || self.description.is_some() {
            let mut header_children = Vec::new();

            // Add DialogTitle if provided
            if let Some(title) = &self.title {
                header_children.push(JSXElementChild::JSXElement(Box::new(dangerous_html_node(
                    title.clone(),
                ))));
            }

            // Add DialogDescription if provided
            if let Some(description) = &self.description {
                header_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "DialogDescription".into(),
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
                            sym: "DialogDescription".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                    }),
                })));
            }

            // Add the header to content
            content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "DialogHeader".into(),
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
                        sym: "DialogHeader".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                }),
            })));
        }

        // Add custom content if provided
        if let Some(content) = &self.content {
            content_children.push(JSXElementChild::JSXElement(Box::new(dangerous_html_node(
                content.clone(),
            ))));
        }

        // Create DialogContent element with all content
        let mut content_attrs = vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(
                Ident {
                    span: DUMMY_SP,
                    sym: "className".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }
                .into(),
            ),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: "sm:max-w-[425px]".into(),
                raw: None,
            }))),
        })];

        // Add custom attributes to the DialogContent element if provided
        if let Some(attributes) = &self.attributes {
            for (key, value) in attributes {
                content_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(
                        Ident {
                            span: DUMMY_SP,
                            sym: key.clone().into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }
                        .into(),
                    ),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: value.clone().into(),
                        raw: None,
                    }))),
                }));
            }
        }

        let content_jsx = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "DialogContent".into(),
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
                    sym: "DialogContent".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        };

        // Add the trigger and content to the dialog
        dialog_jsx
            .children
            .push(JSXElementChild::JSXElement(Box::new(trigger_jsx)));
        dialog_jsx
            .children
            .push(JSXElementChild::JSXElement(Box::new(content_jsx)));

        dialog_jsx
    }
}
