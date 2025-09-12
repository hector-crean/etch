use serde::{Deserialize, Serialize};
use swc_atoms::Atom;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct DrawerOptions {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

pub fn create_drawer_component(trigger_element: JSXElement, options: &DrawerOptions) -> JSXElement {
    let mut drawer_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "Drawer".into(), optional: false, ctxt: SyntaxContext::empty() }),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        },
        children: vec![],
        closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "Drawer".into(), optional: false, ctxt: SyntaxContext::empty() }) }),
    };

    let trigger_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerTrigger".into(), optional: false, ctxt: SyntaxContext::empty() }),
            attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr { span: DUMMY_SP, name: JSXAttrName::Ident(Ident { span: DUMMY_SP, sym: "asChild".into(), optional: false, ctxt: SyntaxContext::empty() }.into()), value: None })],
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerTrigger".into(), optional: false, ctxt: SyntaxContext::empty() }) }),
    };

    let mut content_children = Vec::new();

    if options.title.is_some() || options.description.is_some() {
        let mut header_children = Vec::new();
        if let Some(title) = &options.title {
            header_children.push(JSXElementChild::JSXElement(Box::new(JSXElement { span: DUMMY_SP, opening: JSXOpeningElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerTitle".into(), optional: false, ctxt: SyntaxContext::empty() }), attrs: vec![], self_closing: false, type_args: None }, children: vec![JSXElementChild::JSXText(JSXText { span: DUMMY_SP, value: title.clone().into(), raw: Atom::default() })], closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerTitle".into(), optional: false, ctxt: SyntaxContext::empty() }) }) })));
        }
        if let Some(description) = &options.description {
            header_children.push(JSXElementChild::JSXElement(Box::new(JSXElement { span: DUMMY_SP, opening: JSXOpeningElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerDescription".into(), optional: false, ctxt: SyntaxContext::empty() }), attrs: vec![], self_closing: false, type_args: None }, children: vec![JSXElementChild::JSXText(JSXText { span: DUMMY_SP, value: description.clone().into(), raw: Atom::default() })], closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerDescription".into(), optional: false, ctxt: SyntaxContext::empty() }) }) })));
        }
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement { span: DUMMY_SP, opening: JSXOpeningElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerHeader".into(), optional: false, ctxt: SyntaxContext::empty() }), attrs: vec![], self_closing: false, type_args: None }, children: header_children, closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerHeader".into(), optional: false, ctxt: SyntaxContext::empty() }) }) })));
    }

    let footer_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerFooter".into(), optional: false, ctxt: SyntaxContext::empty() }), attrs: vec![], self_closing: false, type_args: None },
        children: vec![JSXElementChild::JSXElement(Box::new(JSXElement { span: DUMMY_SP, opening: JSXOpeningElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerClose".into(), optional: false, ctxt: SyntaxContext::empty() }), attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr { span: DUMMY_SP, name: JSXAttrName::Ident(Ident { span: DUMMY_SP, sym: "asChild".into(), optional: false, ctxt: SyntaxContext::empty() }.into()), value: None })], self_closing: false, type_args: None }, children: vec![JSXElementChild::JSXElement(Box::new(JSXElement { span: DUMMY_SP, opening: JSXOpeningElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "Button".into(), optional: false, ctxt: SyntaxContext::empty() }), attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr { span: DUMMY_SP, name: JSXAttrName::Ident(Ident { span: DUMMY_SP, sym: "variant".into(), optional: false, ctxt: SyntaxContext::empty() }.into()), value: Some(JSXAttrValue::Lit(Lit::Str(Str { span: DUMMY_SP, value: "outline".into(), raw: None }))) })], self_closing: false, type_args: None }, children: vec![JSXElementChild::JSXText(JSXText { span: DUMMY_SP, value: "Close".into(), raw: Atom::default() })], closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "Button".into(), optional: false, ctxt: SyntaxContext::empty() }) }) }))], closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerClose".into(), optional: false, ctxt: SyntaxContext::empty() }) }) }))],
        closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerFooter".into(), optional: false, ctxt: SyntaxContext::empty() }) }),
    };

    content_children.push(JSXElementChild::JSXElement(Box::new(footer_jsx)));

    let content_jsx = JSXElement { span: DUMMY_SP, opening: JSXOpeningElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerContent".into(), optional: false, ctxt: SyntaxContext::empty() }), attrs: vec![], self_closing: false, type_args: None }, children: content_children, closing: Some(JSXClosingElement { span: DUMMY_SP, name: JSXElementName::Ident(Ident { span: DUMMY_SP, sym: "DrawerContent".into(), optional: false, ctxt: SyntaxContext::empty() }) }) };

    drawer_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(trigger_jsx)));
    drawer_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(content_jsx)));

    drawer_jsx
}


