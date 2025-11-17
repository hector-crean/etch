//! ToJsx implementations for miscellaneous/less common Figma node types
//!
//! These implementations provide placeholder divs for node types that don't have
//! full conversion support yet.

use crate::conversion::jsx::{RenderContext, ToJsx};
use figma_api::models::{
    BooleanOperationNode, ComponentNode, ComponentSetNode, ConnectorNode, EmbedNode,
    InstanceNode, LinkUnfurlNode, SectionNode, ShapeWithTextNode, SliceNode, StickyNode,
    TableCellNode, TableNode, TextPathNode, TransformGroupNode, WidgetNode,
};
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::{
    Ident, IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement,
    JSXElement, JSXElementChild, JSXElementName, JSXOpeningElement, JSXText, Lit, Str,
};
use swc_atoms::Atom;

/// Helper to create a placeholder JSX div with node type and name
fn create_placeholder_div(node_type: &str, name: &str) -> JSXElement {
    let attrs = vec![
        JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "className".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: format!("figma-{}", node_type.to_lowercase()).into(),
                raw: None,
            }))),
        }),
        JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "data-name".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: name.into(),
                raw: None,
            }))),
        }),
        JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "data-type".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: node_type.into(),
                raw: None,
            }))),
        }),
    ];

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
        children: vec![JSXElementChild::JSXText(JSXText {
            span: DUMMY_SP,
            value: format!("{} ({})", name, node_type).into(),
            raw: Atom::new(""),
        })],
    }
}

impl ToJsx for ComponentNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("Component", &self.name)
    }
}

impl ToJsx for ComponentSetNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("ComponentSet", &self.name)
    }
}

impl ToJsx for InstanceNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("Instance", &self.name)
    }
}

impl ToJsx for SectionNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("Section", &self.name)
    }
}

impl ToJsx for BooleanOperationNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("BooleanOperation", &self.name)
    }
}

impl ToJsx for TableNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("Table", &self.name)
    }
}

impl ToJsx for TransformGroupNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("TransformGroup", &self.name)
    }
}

impl ToJsx for TableCellNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("TableCell", &self.name)
    }
}

impl ToJsx for ShapeWithTextNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("ShapeWithText", &self.name)
    }
}

impl ToJsx for TextPathNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("TextPath", &self.name)
    }
}

impl ToJsx for StickyNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("Sticky", &self.name)
    }
}

impl ToJsx for ConnectorNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("Connector", &self.name)
    }
}

impl ToJsx for EmbedNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("Embed", &self.name)
    }
}

impl ToJsx for LinkUnfurlNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("LinkUnfurl", &self.name)
    }
}

impl ToJsx for SliceNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("Slice", &self.name)
    }
}

impl ToJsx for WidgetNode {
    fn to_jsx_with_context(&self, _context: RenderContext) -> JSXElement {
        create_placeholder_div("Widget", &self.name)
    }
}

