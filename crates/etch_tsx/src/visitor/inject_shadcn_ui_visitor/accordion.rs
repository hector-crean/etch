use serde::{Deserialize, Serialize};
use swc_atoms::Atom;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct AccordionItem {
    pub id: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct AccordionOptions {
    pub id: String,
    pub type_: Option<String>, // "single" | "multiple"
    pub collapsible: Option<bool>,
    pub items: Vec<AccordionItem>,
}

pub fn create_accordion_component(_trigger_element: JSXElement, options: &AccordionOptions) -> JSXElement {
    let mut accordion_attrs: Vec<JSXAttrOrSpread> = Vec::new();

    if let Some(type_) = &options.type_ {
        accordion_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident { span: DUMMY_SP, sym: "type".into(), optional: false, ctxt: SyntaxContext::empty() }.into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str { span: DUMMY_SP, value: type_.clone().into(), raw: None }))),
        }));
    }
    if let Some(collapsible) = options.collapsible {
        accordion_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident { span: DUMMY_SP, sym: "collapsible".into(), optional: false, ctxt: SyntaxContext::empty() }.into()),
            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                span: DUMMY_SP,
                expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Bool(Bool { span: DUMMY_SP, value: collapsible })) ),),
            })),
        }));
    }

    let mut accordion_children: Vec<JSXElementChild> = Vec::new();
    for item in &options.items {
        let header_text = JSXElementChild::JSXText(JSXText { span: DUMMY_SP, value: item.title.clone().into(), raw: Atom::default() });
        let content_div = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "AccordionContent".into(), optional: false, ctxt: SyntaxContext::empty() }), attrs: vec![], self_closing: false, type_args: None },
            children: vec![JSXElementChild::JSXText(JSXText { span: DUMMY_SP, value: item.content.clone().into(), raw: Atom::default() })],
            closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "AccordionContent".into(), optional: false, ctxt: SyntaxContext::empty() }) }),
        };

        let trigger = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "AccordionTrigger".into(), optional: false, ctxt: SyntaxContext::empty() }), attrs: vec![], self_closing: false, type_args: None },
            children: vec![header_text],
            closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "AccordionTrigger".into(), optional: false, ctxt: SyntaxContext::empty() }) }),
        };

        let item_el = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "AccordionItem".into(), optional: false, ctxt: SyntaxContext::empty() }),
                attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(Ident { span: DUMMY_SP, sym: "value".into(), optional: false, ctxt: SyntaxContext::empty() }.into()),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str { span: DUMMY_SP, value: item.id.clone().into(), raw: None }))),
                })],
                self_closing: false,
                type_args: None,
            },
            children: vec![JSXElementChild::JSXElement(Box::new(trigger)), JSXElementChild::JSXElement(Box::new(content_div))],
            closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "AccordionItem".into(), optional: false, ctxt: SyntaxContext::empty() }) }),
        };

        accordion_children.push(JSXElementChild::JSXElement(Box::new(item_el)));
    }

    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "Accordion".into(), optional: false, ctxt: SyntaxContext::empty() }), attrs: accordion_attrs, self_closing: false, type_args: None },
        children: accordion_children,
        closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "Accordion".into(), optional: false, ctxt: SyntaxContext::empty() }) }),
    }
}


