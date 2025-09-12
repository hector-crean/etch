use serde::{Deserialize, Serialize};
use swc_atoms::Atom;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct ButtonOptions {
    pub id: String,
    pub label: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

impl Default for ButtonOptions {
    fn default() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            variant: None,
            action: None,
        }
    }
}

pub fn create_button_component(trigger_element: JSXElement, options: &ButtonOptions) -> JSXElement {
    let mut button_attrs: Vec<JSXAttrOrSpread> = Vec::new();
    
    // Add variant attribute if specified
    if let Some(variant) = &options.variant {
        button_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "variant".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: variant.clone().into(),
                raw: None,
            }))),
        }));
    }
    
    // Create button content - use the label from options or preserve original element content
    let button_content = if !options.label.is_empty() {
        vec![JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: options.label.clone().into(),
            raw: Atom::default(),
        })]
    } else {
        // If no label, wrap the original element
        vec![JSXElementChild::JSXElement(Box::new(trigger_element))]
    };
    
    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Button".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: button_attrs,
            self_closing: false,
            type_args: None,
        },
        children: button_content,
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Button".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    }
}
