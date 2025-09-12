use serde::{Deserialize, Serialize};
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct TooltipOptions {
    pub id: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_id: Option<String>,
    
    pub content: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_duration: Option<u32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_delay_duration: Option<u32>,
}

impl Default for TooltipOptions {
    fn default() -> Self {
        Self {
            id: String::new(),
            trigger_id: None,
            content: String::new(),
            side: None,
            align: None,
            delay_duration: None,
            skip_delay_duration: None,
        }
    }
}

pub fn create_tooltip_component(trigger_element: JSXElement, options: &TooltipOptions) -> JSXElement {
    use swc_atoms::Atom;
    use swc_common::SyntaxContext;
    
    let mut tooltip_content_attrs: Vec<JSXAttrOrSpread> = Vec::new();
    
    // Add side attribute if specified
    if let Some(side) = &options.side {
        tooltip_content_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "side".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: side.clone().into(),
                raw: None,
            }))),
        }));
    }
    
    // Add align attribute if specified
    if let Some(align) = &options.align {
        tooltip_content_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident {
                span: DUMMY_SP,
                sym: "align".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }.into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: align.clone().into(),
                raw: None,
            }))),
        }));
    }
    
    // Create TooltipTrigger with asChild
    let trigger = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "TooltipTrigger".into(),
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
                }.into()),
                value: None, // asChild is a boolean prop without value
            })],
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "TooltipTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };
    
    // Create TooltipContent
    let content = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "TooltipContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: tooltip_content_attrs,
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: options.content.clone().into(),
            raw: Atom::default(),
        })],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "TooltipContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };
    
    // Create the main Tooltip wrapper
    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Tooltip".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        },
        children: vec![
            JSXElementChild::JSXElement(Box::new(trigger)),
            JSXElementChild::JSXElement(Box::new(content)),
        ],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Tooltip".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    }
}


