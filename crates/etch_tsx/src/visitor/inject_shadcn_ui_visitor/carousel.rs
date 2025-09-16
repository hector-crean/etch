use serde::{Deserialize, Serialize};
use swc_atoms::Atom;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct CarouselItem {
    pub id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct CarouselOptions {
    pub id: String,
    pub orientation: Option<String>, // "horizontal" | "vertical"
    pub show_navigation: Option<bool>, // Whether to show previous/next buttons
    pub items: Vec<CarouselItem>,
}

pub fn create_carousel_component(_trigger_element: JSXElement, options: &CarouselOptions) -> JSXElement {
    let mut carousel_attrs: Vec<JSXAttrOrSpread> = Vec::new();

    // Add orientation attribute if specified
    if let Some(orientation) = &options.orientation {
        carousel_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(Ident { 
                span: DUMMY_SP, 
                sym: "orientation".into(), 
                optional: false, 
                ctxt: SyntaxContext::empty() 
            }.into()),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str { 
                span: DUMMY_SP, 
                value: orientation.clone().into(), 
                raw: None 
            }))),
        }));
    }

    // Add className for container query support
    carousel_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident { 
            span: DUMMY_SP, 
            sym: "className".into(), 
            optional: false, 
            ctxt: SyntaxContext::empty() 
        }.into()),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str { 
            span: DUMMY_SP, 
            value: "w-full max-w-xs @container/carousel".into(), 
            raw: None 
        }))),
    }));

    // Create carousel items
    let mut carousel_items: Vec<JSXElementChild> = Vec::new();
    for item in &options.items {
        let item_content = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement { 
                span: DUMMY_SP, 
                name: JSXElementName::Ident(Ident { 
                    span: DUMMY_SP, 
                    sym: "div".into(), 
                    optional: false, 
                    ctxt: SyntaxContext::empty() 
                }), 
                attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(Ident { 
                        span: DUMMY_SP, 
                        sym: "className".into(), 
                        optional: false, 
                        ctxt: SyntaxContext::empty() 
                    }.into()),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str { 
                        span: DUMMY_SP, 
                        value: "p-1".into(), 
                        raw: None 
                    }))),
                })], 
                self_closing: false, 
                type_args: None 
            },
            children: vec![JSXElementChild::JSXText(JSXText { 
                span: DUMMY_SP, 
                value: item.content.clone().into(), 
                raw: Atom::default() 
            })],
            closing: Some(JSXClosingElement { 
                span: DUMMY_SP, 
                name: JSXElementName::Ident(Ident { 
                    span: DUMMY_SP, 
                    sym: "div".into(), 
                    optional: false, 
                    ctxt: SyntaxContext::empty() 
                }) 
            }),
        };

        let carousel_item = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident { 
                    span: DUMMY_SP, 
                    sym: "CarouselItem".into(), 
                    optional: false, 
                    ctxt: SyntaxContext::empty() 
                }),
                attrs: vec![],
                self_closing: false,
                type_args: None,
            },
            children: vec![JSXElementChild::JSXElement(Box::new(item_content))],
            closing: Some(JSXClosingElement { 
                span: DUMMY_SP, 
                name: JSXElementName::Ident(Ident { 
                    span: DUMMY_SP, 
                    sym: "CarouselItem".into(), 
                    optional: false, 
                    ctxt: SyntaxContext::empty() 
                }) 
            }),
        };

        carousel_items.push(JSXElementChild::JSXElement(Box::new(carousel_item)));
    }

    // Create CarouselContent
    let carousel_content = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement { 
            span: DUMMY_SP, 
            name: JSXElementName::Ident(Ident { 
                span: DUMMY_SP, 
                sym: "CarouselContent".into(), 
                optional: false, 
                ctxt: SyntaxContext::empty() 
            }), 
            attrs: vec![], 
            self_closing: false, 
            type_args: None 
        },
        children: carousel_items,
        closing: Some(JSXClosingElement { 
            span: DUMMY_SP, 
            name: JSXElementName::Ident(Ident { 
                span: DUMMY_SP, 
                sym: "CarouselContent".into(), 
                optional: false, 
                ctxt: SyntaxContext::empty() 
            }) 
        }),
    };

    // Create navigation buttons if enabled
    let mut carousel_children = vec![JSXElementChild::JSXElement(Box::new(carousel_content))];
    
    if options.show_navigation.unwrap_or(true) {
        // Create CarouselPrevious button
        let previous_button = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement { 
                span: DUMMY_SP, 
                name: JSXElementName::Ident(Ident { 
                    span: DUMMY_SP, 
                    sym: "CarouselPrevious".into(), 
                    optional: false, 
                    ctxt: SyntaxContext::empty() 
                }), 
                attrs: vec![], 
                self_closing: true, 
                type_args: None 
            },
            children: vec![],
            closing: None,
        };

        // Create CarouselNext button
        let next_button = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement { 
                span: DUMMY_SP, 
                name: JSXElementName::Ident(Ident { 
                    span: DUMMY_SP, 
                    sym: "CarouselNext".into(), 
                    optional: false, 
                    ctxt: SyntaxContext::empty() 
                }), 
                attrs: vec![], 
                self_closing: true, 
                type_args: None 
            },
            children: vec![],
            closing: None,
        };

        carousel_children.push(JSXElementChild::JSXElement(Box::new(previous_button)));
        carousel_children.push(JSXElementChild::JSXElement(Box::new(next_button)));
    }

    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement { 
            span: DUMMY_SP, 
            name: JSXElementName::Ident(Ident { 
                span: DUMMY_SP, 
                sym: "Carousel".into(), 
                optional: false, 
                ctxt: SyntaxContext::empty() 
            }), 
            attrs: carousel_attrs, 
            self_closing: false, 
            type_args: None 
        },
        children: carousel_children,
        closing: Some(JSXClosingElement { 
            span: DUMMY_SP, 
            name: JSXElementName::Ident(Ident { 
                span: DUMMY_SP, 
                sym: "Carousel".into(), 
                optional: false, 
                ctxt: SyntaxContext::empty() 
            }) 
        }),
    }
}
