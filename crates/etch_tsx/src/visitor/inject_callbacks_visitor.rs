use log::info;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{AsRefStr, Display, EnumString};
use swc_common::{DUMMY_SP, SyntaxContext};
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

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq, Hash)]
#[ts(export)]
pub struct TypeArgument {
    pub type_expr: TypeExpression,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq, Hash)]
#[ts(export)]
#[serde(tag = "typeKind", content = "value")]
pub enum TypeExpression {
    // Primitive types
    String,
    Number,
    Boolean,
    Null,
    Undefined,
    Any,
    Void,
    Never,
    Unknown,

    // Named type reference (e.g., React.FormEvent, HTMLInputElement)
    Reference(String),

    // Complex types
    Array(Box<TypeExpression>),
    Tuple(Vec<TypeExpression>),
    Union(Vec<TypeExpression>),
    Intersection(Vec<TypeExpression>),

    // Object type with properties
    Object(Vec<ObjectProperty>),

    // Function type
    Function(FunctionType),

    // Generic type
    Generic {
        base: String,
        args: Vec<TypeArgument>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq, Hash)]
#[ts(export)]
pub struct ObjectProperty {
    pub name: String,
    pub type_expr: TypeExpression,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq, Hash)]
#[ts(export)]
pub struct FunctionType {
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Box<TypeExpression>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq, Hash)]
#[ts(export)]
pub struct FunctionParameter {
    pub name: String,
    pub type_expr: TypeExpression,
    pub optional: bool,
    pub rest: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct HandlerFunction {
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: TypeExpression,
    pub code: Option<String>, // Optional inline code if supported
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Callback {
    pub trigger: Event,
    pub handler: HandlerFunction,
}

impl Callback {
    pub fn new(trigger: Event, handler: HandlerFunction) -> Self {
        Self { trigger, handler }
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
        // let mut used_actions = HashSet::new();

        // // From callbacks
        // for callbacks in self.callbacks.values() {
        //     for callback in callbacks {
        //         used_actions.insert(callback.handler.as_ref().to_string());
        //     }
        // }

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

    // Generate a function call based on the handler signature
    let args: Vec<ExprOrSpread> = callback
        .handler
        .parameters
        .iter()
        .map(|param| {
            // You might need additional logic to handle rest parameters
            // and optional parameters differently
            ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Ident(Ident {
                    span: DUMMY_SP,
                    sym: param.name.clone().into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                })),
            }
        })
        .collect();

    stmts.push(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Call(CallExpr {
            ctxt: SyntaxContext::empty(),
            span: DUMMY_SP,
            callee: Callee::Expr(Box::new(Expr::Ident(Ident {
                span: DUMMY_SP,
                sym: callback.handler.name.clone().into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }))),
            args,
            type_args: None,
        })),
    }));

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
            stmts,
        })),
        is_async: false,
        is_generator: false,
        type_params: None,
        return_type: None,
    })
}
