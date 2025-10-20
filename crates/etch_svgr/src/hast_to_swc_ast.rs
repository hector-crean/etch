use std::{borrow::Cow, collections::HashMap};

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use swc_atoms::Atom;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use swc_xml::visit::{Visit, VisitWith};

lazy_static! {
    static ref ATTR_MAPPINGS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("class", "className");
        m.insert("for", "htmlFor");
        m.insert("accept-charset", "acceptCharset");
        m.insert("http-equiv", "httpEquiv");
        m
    };
}

fn kebab_case(str: &str) -> Cow<str> {
    lazy_static! {
        static ref KEBAB_REGEX: Regex = Regex::new(r"[A-Z\u00C0-\u00D6\u00D8-\u00DE]").unwrap();
    }
    KEBAB_REGEX.replace_all(str, |caps: &Captures| {
        format!("-{}", &caps[0].to_lowercase())
    })
}

fn convert_aria_attribute(kebab_key: &str) -> String {
    let parts: Vec<&str> = kebab_key.split('-').collect();
    let aria = parts[0];
    let lowercase_parts: String = parts[1..].join("").to_lowercase();
    format!("{}-{}", aria, lowercase_parts)
}

fn replace_spaces(s: &str) -> Cow<str> {
    lazy_static! {
        static ref SPACES_REGEX: Regex = Regex::new(r"[\t\r\n\u0085\u2028\u2029]+").unwrap();
    }
    SPACES_REGEX.replace_all(s, |_: &Captures| " ")
}

fn decode_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn is_numeric(value: &str) -> bool {
    value.parse::<f64>().is_ok()
}

fn get_value(attr_name: &str, value: &str) -> JSXAttrValue {
    if attr_name == "style" {
        // For now, just treat style as a string
        // In a full implementation, you'd parse it into an object
        return JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: value.into(),
            raw: None,
        }));
    }

    if is_numeric(value) {
        return JSXAttrValue::JSXExprContainer(JSXExprContainer {
            span: DUMMY_SP,
            expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num(Number {
                span: DUMMY_SP,
                value: value.parse().unwrap(),
                raw: None,
            })))),
        });
    }

    JSXAttrValue::Lit(Lit::Str(Str {
        span: DUMMY_SP,
        value: replace_spaces(value).into(),
        raw: None,
    }))
}

fn text(n: &swc_xml::ast::Text) -> Option<JSXElementChild> {
    lazy_static! {
        static ref SPACE_REGEX: Regex = Regex::new(r"^\s+$").unwrap();
    }

    let value = n.data.as_str();
    if SPACE_REGEX.is_match(value) {
        return None;
    }

    Some(JSXElementChild::JSXText(JSXText {
        span: DUMMY_SP,
        value: decode_xml(value).into(),
        raw: Atom::new(""),
    }))
}

pub struct HastVisitor {
    jsx: Option<JSXElement>,
    attr_mappings: &'static HashMap<&'static str, &'static str>,
}

impl HastVisitor {
    fn new() -> Self {
        Self {
            jsx: None,
            attr_mappings: &ATTR_MAPPINGS,
        }
    }

    pub fn take_jsx(&mut self) -> Option<JSXElement> {
        self.jsx.take()
    }

    fn element(&self, n: &swc_xml::ast::Element) -> JSXElement {
        let attrs = n
            .attributes
            .iter()
            .map(|attr| {
                let value = attr
                    .value
                    .as_ref()
                    .map(|v| get_value(&attr.name, v.as_str()));
                JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(self.get_key(&attr.name, &n.tag_name).into()),
                    value,
                })
            })
            .collect::<Vec<JSXAttrOrSpread>>();

        let name = JSXElementName::Ident(Ident::new(
            n.tag_name.clone(),
            DUMMY_SP,
            SyntaxContext::empty(),
        ));
        let children = self.all(&n.children);

        let closing = if !children.is_empty() {
            Some(JSXClosingElement {
                span: DUMMY_SP,
                name: name.clone(),
            })
        } else {
            None
        };

        let opening = JSXOpeningElement {
            span: DUMMY_SP,
            name,
            attrs,
            self_closing: children.is_empty(),
            type_args: None,
        };

        JSXElement {
            span: DUMMY_SP,
            opening,
            children,
            closing,
        }
    }

    fn all(&self, children: &[swc_xml::ast::Child]) -> Vec<JSXElementChild> {
        children
            .iter()
            .filter_map(|n| match n {
                swc_xml::ast::Child::Element(e) => {
                    Some(JSXElementChild::JSXElement(Box::new(self.element(e))))
                }
                swc_xml::ast::Child::Text(t) => text(t),
                _ => None,
            })
            .collect()
    }

    fn get_key(&self, attr_name: &str, tag_name: &str) -> Ident {
        let lower_case_name = attr_name.to_lowercase();
        let rc_key = {
            match tag_name {
                "input" => match lower_case_name.as_str() {
                    "checked" => Some("defaultChecked"),
                    "value" => Some("defaultValue"),
                    "maxlength" => Some("maxLength"),
                    _ => None,
                },
                "form" => match lower_case_name.as_str() {
                    "enctype" => Some("encType"),
                    _ => None,
                },
                _ => None,
            }
        };

        if let Some(k) = rc_key {
            return Ident {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                sym: (*k).into(),
                optional: false,
            };
        }

        let mapped_attr = self.attr_mappings.get(lower_case_name.as_str());
        if let Some(k) = mapped_attr {
            return Ident {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                sym: (*k).into(),
                optional: false,
            };
        }

        let kebab_key = kebab_case(attr_name);

        if kebab_key.starts_with("aria-") {
            return Ident {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                sym: convert_aria_attribute(attr_name).into(),
                optional: false,
            };
        }

        if kebab_key.starts_with("data-") {
            return Ident {
                span: DUMMY_SP,
                ctxt: SyntaxContext::empty(),
                sym: attr_name.into(),
                optional: false,
            };
        }

        Ident {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            sym: attr_name.into(),
            optional: false,
        }
    }
}

impl Visit for HastVisitor {
    fn visit_element(&mut self, n: &swc_xml::ast::Element) {
        self.jsx = Some(self.element(n));
    }
}

pub fn to_swc_ast(hast: swc_xml::ast::Document) -> Option<JSXElement> {
    let mut v = HastVisitor::new();
    hast.visit_with(&mut v);
    v.take_jsx()
}
