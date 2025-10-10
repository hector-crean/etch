use figma_api::models::{TextNode, TypeStyle};
use std::collections::HashMap;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::{
    Expr, Ident, IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement,
    JSXElement, JSXElementChild, JSXElementName, JSXExpr, JSXExprContainer, JSXOpeningElement,
    JSXText, KeyValueProp, Lit, ObjectLit, Prop, PropName, PropOrSpread, Str,
};

/// Extension trait for TextNode to provide JSX conversion
pub trait TextNodeExt {
    fn to_jsx(&self) -> JSXElement;
}

impl TextNodeExt for TextNode {
    fn to_jsx(&self) -> JSXElement {
        use crate::tailwind_ext::TailwindStyleExt;

        // Get base Tailwind styles
        let styles = self.to_tailwind();
        let mut attrs = Vec::new();

        // Add className attribute
        if !styles.classes.is_empty() {
            attrs.push(create_text_attr("className", &styles.class_string()));
        }

        // Add inline styles if present
        if !styles.inline_styles.is_empty() {
            let style_props: Vec<PropOrSpread> = styles
                .inline_styles
                .iter()
                .map(|(key, value)| {
                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Str(Str {
                            span: DUMMY_SP,
                            value: key.clone().into(),
                            raw: None,
                        }),
                        value: Box::new(Expr::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: value.clone().into(),
                            raw: None,
                        }))),
                    })))
                })
                .collect();

            attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "style".into(),
                }),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Object(ObjectLit {
                        span: DUMMY_SP,
                        props: style_props,
                    }))),
                })),
            }));
        }

        // Add data attributes
        attrs.push(create_text_attr("data-name", &self.name));
        attrs.push(create_text_attr("data-text", "true"));

        // Build children with style overrides
        let children = build_text_children(self);

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
                attrs,
                self_closing: false,
                type_args: None,
            },
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "div".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
            children,
        }
    }
}

/// Build text children with style overrides
fn build_text_children(text: &TextNode) -> Vec<JSXElementChild> {
    let characters = &text.characters;
    let overrides = &text.character_style_overrides;
    let style_table = &text.style_override_table;

    // If no overrides, return simple text
    if overrides.is_empty() || style_table.is_empty() {
        return vec![JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: characters.clone().into(),
            raw: characters.clone().into(),
        })];
    }

    // Group consecutive characters with same style
    let mut children = Vec::new();
    let mut current_text = String::new();
    let mut current_style_id: Option<String> = None;

    for (i, ch) in characters.chars().enumerate() {
        // Get style ID for this character
        let style_id = if i < overrides.len() {
            let override_val = overrides[i];
            if override_val > 0.0 {
                Some(override_val.to_string())
            } else {
                None
            }
        } else {
            None
        };

        // If style changes, create a span for previous text
        if style_id != current_style_id {
            if !current_text.is_empty() {
                children.push(create_styled_span(
                    &current_text,
                    current_style_id.as_deref(),
                    style_table,
                ));
                current_text.clear();
            }
            current_style_id = style_id;
        }

        current_text.push(ch);
    }

    // Add remaining text
    if !current_text.is_empty() {
        children.push(create_styled_span(
            &current_text,
            current_style_id.as_deref(),
            style_table,
        ));
    }

    children
}

/// Create a styled span element
fn create_styled_span(
    text: &str,
    style_id: Option<&str>,
    style_table: &HashMap<String, TypeStyle>,
) -> JSXElementChild {
    // If no style override, return plain text
    if style_id.is_none() {
        return JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: text.into(),
            raw: text.into(),
        });
    }

    // Get the style from table
    let style = style_id.and_then(|id| style_table.get(id));

    let mut attrs = Vec::new();

    // Apply style overrides using Tailwind classes
    if let Some(type_style) = style {
        let mut classes = Vec::new();

        // Font family
        if let Some(font_family) = &type_style.font_family {
            let clean_font_family = font_family.trim().trim_matches(|c| c == '\'' || c == '"');
            classes.push(format!(
                "font-['{}']",
                clean_font_family.replace('\'', "\\'")
            ));
        }

        // Font size
        if let Some(font_size) = type_style.font_size {
            classes.push(format!("text-[{}px]", font_size));
        }

        // Font weight
        if let Some(font_weight) = type_style.font_weight {
            let weight_class = match font_weight as u16 {
                100 => "font-thin",
                200 => "font-extralight",
                300 => "font-light",
                400 => "font-normal",
                500 => "font-medium",
                600 => "font-semibold",
                700 => "font-bold",
                800 => "font-extrabold",
                900 => "font-black",
                _ => "font-normal",
            };
            classes.push(weight_class.to_string());
        }

        // Italic
        if let Some(italic) = type_style.italic {
            if italic {
                classes.push("italic".to_string());
            }
        }

        // Letter spacing
        if let Some(letter_spacing) = type_style.letter_spacing {
            let em_value = letter_spacing / type_style.font_size.unwrap_or(16.0);
            classes.push(format!("tracking-[{}em]", em_value));
        }

        // Text color
        if let Some(fills) = &type_style.fills {
            if let Some(color) = crate::tsx::paint::extract_first_paint_color(fills) {
                classes.push(format!("text-[{}]", color));
            }
        }

        // Add className attribute if we have classes
        if !classes.is_empty() {
            attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "className".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: classes.join(" ").into(),
                    raw: None,
                }))),
            }));
        }
    }

    let span_element = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "span".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs,
            self_closing: false,
            type_args: None,
        },
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "span".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
        children: vec![JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: text.into(),
            raw: text.into(),
        })],
    };

    JSXElementChild::JSXElement(Box::new(span_element))
}

/// Helper to create a text attribute
fn create_text_attr(name: &str, value: &str) -> JSXAttrOrSpread {
    JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(IdentName {
            span: DUMMY_SP,
            sym: name.into(),
        }),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: value.into(),
            raw: None,
        }))),
    })
}
