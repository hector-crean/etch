use super::dangerously_set_node::dangerous_html_node;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use swc_atoms::Atom;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export)]
#[serde(tag = "type", content = "props")]
pub enum DialogContent {
    /// Inline HTML string that will be embedded
    RawHtml(String),
    /// Import a TSX/TS component and render it
    /// If import_name is None, we will import the default export
    TsxImport {
        import_path: String,
        import_name: Option<String>,
    },
    /// Generic URI. If an SVG (.svg) is provided, we will try to inline it.
    /// Otherwise this may be rendered as a basic <img src> fallback in the future.
    Uri(String),
}

/// Derive a stable local identifier for an import based on path and optional named export
pub fn derive_import_local_name(import_path: &str, import_name: Option<&str>) -> String {
    if let Some(name) = import_name {
        return name.to_string();
    }

    let last_segment = import_path.split('/').last().unwrap_or("Component");

    let stem = last_segment.split('.').next().unwrap_or(last_segment);

    // Convert to PascalCase
    let mut result = String::new();
    let mut capitalize = true;
    for ch in stem.chars() {
        if ch == '-' || ch == '_' || ch == ' ' {
            capitalize = true;
            continue;
        }
        if capitalize {
            result.extend(ch.to_uppercase());
            capitalize = false;
        } else {
            result.push(ch);
        }
    }
    if result.is_empty() {
        "Component".to_string()
    } else {
        result
    }
}
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
    pub content: Option<DialogContent>,
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
            if !header_children.is_empty() {
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
        }

        // Add custom content if provided
        if let Some(content) = &self.content {
            match content {
                DialogContent::RawHtml(html) => {
                    content_children.push(JSXElementChild::JSXElement(Box::new(
                        dangerous_html_node(html.clone()),
                    )));
                }
                DialogContent::TsxImport {
                    import_path,
                    import_name,
                } => {
                    let local =
                        derive_import_local_name(import_path.as_str(), import_name.as_deref());
                    content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                        span: DUMMY_SP,
                        opening: JSXOpeningElement {
                            span: DUMMY_SP,
                            name: JSXElementName::Ident(Ident {
                                span: DUMMY_SP,
                                sym: local.into(),
                                optional: false,
                                ctxt: SyntaxContext::empty(),
                            }),
                            attrs: vec![],
                            self_closing: true,
                            type_args: None,
                        },
                        children: vec![],
                        closing: None,
                    })));
                }
                DialogContent::Uri(uri) => {
                    if uri.ends_with(".svg") {
                        if let Ok(svg_str) = std::fs::read_to_string(uri) {
                            content_children.push(JSXElementChild::JSXElement(Box::new(
                                dangerous_html_node(svg_str),
                            )));
                        }
                    } else if uri.ends_with(".tsx") || uri.ends_with(".jsx") {
                        // Render the component; import will be injected elsewhere
                        let local = derive_import_local_name(uri.as_str(), None);
                        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                            span: DUMMY_SP,
                            opening: JSXOpeningElement {
                                span: DUMMY_SP,
                                name: JSXElementName::Ident(Ident {
                                    span: DUMMY_SP,
                                    sym: local.into(),
                                    optional: false,
                                    ctxt: SyntaxContext::empty(),
                                }),
                                attrs: vec![],
                                self_closing: true,
                                type_args: None,
                            },
                            children: vec![],
                            closing: None,
                        })));
                    }
                }
            }
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
