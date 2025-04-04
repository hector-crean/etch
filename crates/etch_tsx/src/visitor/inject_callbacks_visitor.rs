use log::info;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use strum::{AsRefStr, Display, EnumString};
use swc_atoms::Atom;
use swc_atoms::atom;
use swc_common::{DUMMY_SP, Span, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use ts_rs::TS;

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Display, EnumString, AsRefStr, TS, Serialize, Deserialize,
)]
#[ts(export)]
#[strum(serialize_all = "camelCase")]
pub enum Event {
    #[strum(serialize = "onClick")]
    Click,
    #[strum(serialize = "onHover")]
    Hover,
    #[strum(serialize = "onFocus")]
    Focus,
    #[strum(serialize = "onChange")]
    Change,
    #[strum(serialize = "onSubmit")]
    Submit,
    #[strum(serialize = "onHoverStart")]
    HoverStart,
    #[strum(serialize = "onHoverEnd")]
    HoverEnd,
    #[strum(serialize = "onDrag")]
    Drag,
    #[strum(serialize = "onDragStart")]
    DragStart,
    #[strum(serialize = "onDragEnd")]
    DragEnd,
    #[strum(serialize = "onAnimationStart")]
    AnimationStart,
    #[strum(serialize = "onAnimationComplete")]
    AnimationComplete,
    #[strum(serialize = "onViewportEnter")]
    ViewportEnter,
    #[strum(serialize = "onViewportLeave")]
    ViewportLeave,
    #[strum(serialize = "onTap")]
    Tap,
    #[strum(serialize = "onTapStart")]
    TapStart,
    #[strum(serialize = "onTapCancel")]
    TapCancel,
    #[strum(serialize = "onPan")]
    Pan,
    #[strum(serialize = "onPanStart")]
    PanStart,
    #[strum(serialize = "onPanEnd")]
    PanEnd,
    #[strum(serialize = "onMouseEnter")]
    MouseEnter,
    #[strum(serialize = "onMouseLeave")]
    MouseLeave,
    #[strum(serialize = "onKeyDown")]
    KeyDown,
    #[strum(serialize = "onKeyUp")]
    KeyUp,
    #[strum(serialize = "onTouchStart")]
    TouchStart,
    #[strum(serialize = "onTouchEnd")]
    TouchEnd,
    #[strum(serialize = "onTouchMove")]
    TouchMove,
    #[strum(serialize = "onTouchCancel")]
    TouchCancel,
    #[strum(serialize = "onWheel")]
    Wheel,
}

impl Event {
    pub fn to_handler_name(&self) -> String {
        self.as_ref().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct ShowToastOptions {
    pub message: String,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Display, EnumString, AsRefStr, TS, Serialize, Deserialize,
)]
#[ts(export)]
#[strum(serialize_all = "camelCase")]
#[serde(tag = "type")]
pub enum Action {
    // Toast actions
    Toast(ShowToastOptions), // Show a toast notification
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Callback {
    pub trigger: Event,
    pub action: Action,
}

impl Callback {
    pub fn new(trigger: Event, action: Action) -> Self {
        Self { trigger, action }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ElementCallbacks {
    pub element_id: String,
    pub callbacks: Vec<Callback>,
}

/// A visitor that adds event handlers to JSX elements and transforms JSX structure
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub struct InjectCallbacksVisitor {
    pub callbacks: HashMap<String, Vec<Callback>>,
}

impl InjectCallbacksVisitor {
    pub fn new(callbacks: HashMap<String, Vec<Callback>>) -> Self {
        Self { callbacks }
    }

    /// Register a callback function for a specific element ID
    pub fn register_callback(&mut self, id: String, callback: Callback) {
        if let Some(callbacks) = self.callbacks.get_mut(&id) {
            callbacks.push(callback);
        } else {
            self.callbacks.insert(id.clone(), vec![callback]);
        }
    }
}

impl VisitMut for InjectCallbacksVisitor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        // First collect all used actions
        let mut used_actions = HashSet::new();

        // From callbacks
        for callbacks in self.callbacks.values() {
            for callback in callbacks {
                used_actions.insert(callback.action.as_ref().to_string());
            }
        }

        // From component wrappers

        // Process all JSX elements
        module.visit_mut_children_with(self);
    }

    fn visit_mut_jsx_element(&mut self, node: &mut JSXElement) {
        // Process attributes
        let mut element_id = None;

        // First pass: find the ID attribute
        for attr in &node.opening.attrs {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                if let JSXAttrName::Ident(ident) = &jsx_attr.name {
                    if ident.sym.as_ref() == "id" {
                        if let Some(JSXAttrValue::Lit(Lit::Str(str_lit))) = &jsx_attr.value {
                            element_id = Some(str_lit.value.to_string());
                        }
                    }
                }
            }
        }

        let id = element_id.clone();
        if let Some(id) = id {
            if let Some(callbacks) = self.callbacks.get(&id) {
                for callback in callbacks {
                    let event_name = callback.trigger.to_handler_name();

                    // Create an event handler expression
                    let handler = create_event_handler(callback, id.clone());

                    // Add the event handler as a new attribute
                    node.opening.attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(
                            Ident {
                                span: DUMMY_SP,
                                sym: event_name.clone().into(),
                                optional: false,
                                ctxt: SyntaxContext::empty(),
                            }
                            .into(),
                        ),
                        value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                            span: DUMMY_SP,
                            expr: JSXExpr::Expr(Box::new(handler)),
                        })),
                    }));

                    info!("Added {} handler to element with id '{}'", event_name, id);
                }
            }
        }

        // Visit children
        node.visit_mut_children_with(self);
    }
}

/// Creates an event handler expression for the given callback
fn create_event_handler(callback: &Callback, id: String) -> Expr {
    let mut stmts = Vec::new();

    let func_name = callback.action.as_ref().to_string();

    match &callback.action {
        Action::Toast(props) => {
            // Call toast with just the message string instead of a full object
            stmts.push(Stmt::Expr(ExprStmt {
                span: DUMMY_SP,
                expr: Box::new(Expr::Call(CallExpr {
                    ctxt: SyntaxContext::empty(),
                    span: DUMMY_SP,
                    callee: Callee::Expr(Box::new(Expr::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "toast".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }))),
                    args: vec![ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: props.message.clone().into(),
                            raw: None,
                        }))),
                    }],
                    type_args: None,
                })),
            }));
        }
    }

    // Return the constructed arrow function
    Expr::Arrow(ArrowExpr {
        ctxt: SyntaxContext::empty(),
        span: DUMMY_SP,
        params: vec![Pat::Ident(BindingIdent {
            id: Ident {
                span: DUMMY_SP,
                sym: "e".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            },
            type_ann: None,
        })],
        body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
            span: DUMMY_SP,
            ctxt: SyntaxContext::empty(),
            stmts: stmts,
        })),
        is_async: false,
        is_generator: false,
        type_params: None,
        return_type: None,
    })
}
