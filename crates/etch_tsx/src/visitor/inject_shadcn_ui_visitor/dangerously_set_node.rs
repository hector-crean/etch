use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use swc_atoms::Atom;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use ts_rs::TS;

/// Creates a JSX element with dangerouslySetInnerHTML set to the provided content
pub fn dangerous_html_node(content: String) -> JSXElement {
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
            attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "dangerouslySetInnerHTML".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Object(ObjectLit {
                        span: DUMMY_SP,
                        props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(
                                Ident {
                                    span: DUMMY_SP,
                                    sym: "__html".into(),
                                    optional: false,
                                    ctxt: SyntaxContext::empty(),
                                }
                                .into(),
                            ),
                            value: Box::new(Expr::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: content.clone().into(),
                                raw: None,
                            }))),
                        })))],
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
    }
}
