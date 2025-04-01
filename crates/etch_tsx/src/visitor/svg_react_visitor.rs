use etch_svg::SvgParser;
use log::info;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;
use std::collections::HashSet;
use strum::{AsRefStr, Display, EnumString};
use swc_common::{DUMMY_SP, Span, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, EnumString, AsRefStr, TS)]
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
pub struct OpenModalOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
pub struct CloseModalOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
pub struct ToggleModalOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
pub struct OpenSheetOptions {
    pub id: String,
    pub side: Option<String>, // "top", "right", "bottom", "left"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
pub struct CloseSheetOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
pub struct ShowToastOptions {
    pub title: String,
    pub description: Option<String>,
    pub variant: Option<String>, // "default", "destructive", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
pub struct SelectTabOptions {
    pub tabGroupId: String,
    pub tabId: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
pub struct ToggleAccordionOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
pub struct OpenDropdownOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
pub struct CloseDropdownOptions {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, EnumString, AsRefStr, TS)]
#[ts(export)]
#[strum(serialize_all = "camelCase")]
pub enum Action {
    // Modal actions
    OpenModal(OpenModalOptions),          // Open a modal with the specified ID
    CloseModal(CloseModalOptions),        // Close a modal with the specified ID
    ToggleModal(ToggleModalOptions),      // Toggle a modal's open/closed state
    
    // Sheet/Drawer actions
    OpenSheet(OpenSheetOptions),          // Open a sheet/drawer with the specified ID and side
    CloseSheet(CloseSheetOptions),        // Close a sheet/drawer with the specified ID
    
    // Toast actions
    ShowToast(ShowToastOptions),          // Show a toast notification
    
    // Tab actions
    SelectTab(SelectTabOptions),          // Select a specific tab in a tab group
    
    // Accordion actions
    ToggleAccordion(ToggleAccordionOptions), // Toggle an accordion's expanded/collapsed state
    
    // Dropdown/Select actions
    OpenDropdown(OpenDropdownOptions),    // Open a dropdown menu
    CloseDropdown(CloseDropdownOptions),  // Close a dropdown menu
}

#[derive(Clone)]
pub struct Callback {
    pub trigger: Event,
    pub action: Action,
}

impl Callback {
    pub fn new(trigger: Event, action: Action) -> Self {
        Self { trigger, action }
    }
}

// Replace CustomEvent with this
pub struct ElementCallbacks {
    pub element_id: String,
    pub callbacks: Vec<Callback>,
}

/// A visitor that adds event handlers to JSX elements based on ID attributes
pub struct FigmaExportVisitor {
    functions: HashMap<String, Vec<Callback>>,
    action_imports: HashMap<String, HashSet<String>>, // Maps import paths to action names
}

impl FigmaExportVisitor {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            action_imports: HashMap::new(),
        }
    }

    /// Register a callback function for a specific element ID
    pub fn register_callback(&mut self, id: String, callback: Callback) {
        let trigger_event = callback.trigger.to_handler_name();
        
        // Register the callback
        if let Some(callbacks) = self.functions.get_mut(&id) {
            callbacks.push(callback);
        } else {
            self.functions.insert(id.clone(), vec![callback]);
        }
    }
    
    /// Register an action with its import path
    pub fn register_action_import(&mut self, action_name: &str, import_path: &str) {
        self.action_imports
            .entry(import_path.to_string())
            .or_insert_with(HashSet::new)
            .insert(action_name.to_string());
    }
    
    /// Register default import paths for all actions
    pub fn register_default_imports(&mut self) {
        // UI component actions organized by component type
        
        // Modal actions
        let modal_path = "@/components/ui/modal";
        self.register_action_import("openModal", modal_path);
        self.register_action_import("closeModal", modal_path);
        self.register_action_import("toggleModal", modal_path);
        
        // Sheet/Drawer actions
        let sheet_path = "@/components/ui/sheet";
        self.register_action_import("openSheet", sheet_path);
        self.register_action_import("closeSheet", sheet_path);
        
        // Toast actions
        let toast_path = "@/components/ui/toast";
        self.register_action_import("showToast", toast_path);
        
        // Tab actions
        let tabs_path = "@/components/ui/tabs";
        self.register_action_import("selectTab", tabs_path);
        
        // Accordion actions
        let accordion_path = "@/components/ui/accordion";
        self.register_action_import("toggleAccordion", accordion_path);
        
        // Dropdown/Select actions
        let dropdown_path = "@/components/ui/dropdown-menu";
        self.register_action_import("openDropdown", dropdown_path);
        self.register_action_import("closeDropdown", dropdown_path);
        
        // Add more action imports as needed
        // For example, navigation actions might come from a different path
        // self.register_action_import("navigate", "@/lib/navigation");
    }
    
    /// Get all required import statements based on used actions
    fn get_action_imports(&self, used_actions: &HashSet<String>) -> Vec<ModuleItem> {
        let mut imports = Vec::new();
        
        for (import_path, actions) in &self.action_imports {
            // Filter to only include actions that are actually used
            let mut used_from_this_path: Vec<String> = actions.iter()
                .filter(|action| used_actions.contains(*action))
                .cloned()
                .collect();
            
            if !used_from_this_path.is_empty() {
                // Sort for consistent output
                used_from_this_path.sort();
                
                // Create named imports
                let named_imports = used_from_this_path.iter().map(|action| {
                    ImportSpecifier::Named(ImportNamedSpecifier {
                        span: DUMMY_SP,
                        local: Ident {
                            span: DUMMY_SP,
                            sym: action.clone().into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        },
                        imported: None,
                        is_type_only: false,
                    })
                }).collect();
                
                // Create import declaration
                imports.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    span: DUMMY_SP,
                    phase: ImportPhase::Evaluation,
                    specifiers: named_imports,
                    src: Box::new(Str {
                        span: DUMMY_SP,
                        value: import_path.clone().into(),
                        raw: None,
                    }),
                    type_only: false,
                    with: None,
                })));
            }
        }
        
        imports
    }
}

impl VisitMut for FigmaExportVisitor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        // Set up default imports if none configured
        if self.action_imports.is_empty() {
            self.register_default_imports();
        }
        
        // First collect all used actions
        let mut used_actions = HashSet::new();
        
        for callbacks in self.functions.values() {
            for callback in callbacks {
                used_actions.insert(callback.action.as_ref().to_string());
            }
        }
        
        // Process all JSX elements
        module.visit_mut_children_with(self);
        
        // Add imports for the used actions
        let imports = self.get_action_imports(&used_actions);
        
        // Insert at the beginning of the module, in reverse order to maintain desired order
        for import in imports.into_iter().rev() {
            module.body.insert(0, import);
        }
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

        // Second pass: add event handlers if we have a matching ID
        if let Some(id) = element_id {
            if let Some(callbacks) = self.functions.get(&id) {
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
    Action::OpenModal(props) => {
        stmts.push(create_function_with_struct_stmt(&func_name, &props));
    }
    Action::CloseModal(props) => {
        stmts.push(create_function_with_struct_stmt(&func_name, &props));
    }
    Action::ToggleModal(props) => {
        stmts.push(create_function_with_struct_stmt(&func_name, &props));
    }
    Action::OpenSheet(props) => {
        stmts.push(create_function_with_struct_stmt(&func_name, &props));
    }
    Action::CloseSheet(props) => {
        stmts.push(create_function_with_struct_stmt(&func_name, &props));
    }
    Action::ShowToast(props) => {
        stmts.push(create_function_with_struct_stmt(&func_name, &props));
    }
    Action::SelectTab(props) => {
        stmts.push(create_function_with_struct_stmt(&func_name, &props));
    }
    Action::ToggleAccordion(props) => {
        stmts.push(create_function_with_struct_stmt(&func_name, &props));
    }
    Action::OpenDropdown(props) => {
        stmts.push(create_function_with_struct_stmt(&func_name, &props));
    }
    Action::CloseDropdown(props) => {
        stmts.push(create_function_with_struct_stmt(&func_name, &props));
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



// Helper function to create an object expression with properties
fn create_object_expr(properties: Vec<(String, Expr)>) -> Expr {
    let props = properties
        .into_iter()
        .map(|(key, value)| {
            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: key.into(),
                }),
                value: Box::new(value),
            })))
        })
        .collect();

    Expr::Object(ObjectLit {
        span: DUMMY_SP,
        props,
    })
}

/// Creates a statement calling a function with an object of properties as its argument
/// Example: functionName({ prop1: "value1", prop2: 123, prop3: true })
fn create_function_with_props_stmt(function_name: &str, props: Vec<(String, Expr)>) -> Stmt {
    Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Call(CallExpr {
            ctxt: SyntaxContext::empty(),
            span: DUMMY_SP,
            callee: Callee::Expr(Box::new(Expr::Ident(Ident {
                span: DUMMY_SP,
                sym: function_name.into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }))),
            args: vec![ExprOrSpread {
                spread: None,
                expr: Box::new(create_object_expr(props)),
            }],
            type_args: None,
        })),
    })
}

/// Converts a serde_json::Value to an SWC Expr
fn json_value_to_expr(value: &JsonValue) -> Expr {
    match value {
        JsonValue::Null => Expr::Lit(Lit::Null(Null { span: DUMMY_SP })),
        JsonValue::Bool(b) => Expr::Lit(Lit::Bool(Bool {
            span: DUMMY_SP,
            value: *b,
        })),
        JsonValue::Number(n) => {
            // Handle potential loss of precision for f64 values
            if let Some(i) = n.as_i64() {
                Expr::Lit(Lit::Num(Number {
                    span: DUMMY_SP,
                    value: i as f64,
                    raw: None,
                }))
            } else if let Some(f) = n.as_f64() {
                Expr::Lit(Lit::Num(Number {
                    span: DUMMY_SP,
                    value: f,
                    raw: None,
                }))
            } else {
                // Fallback
                Expr::Lit(Lit::Num(Number {
                    span: DUMMY_SP,
                    value: 0.0,
                    raw: None,
                }))
            }
        }
        JsonValue::String(s) => Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: s.clone().into(),
            raw: None,
        })),
        JsonValue::Array(arr) => {
            let elements = arr
                .iter()
                .map(|v| {
                    Some(ExprOrSpread {
                        spread: None,
                        expr: Box::new(json_value_to_expr(v)),
                    })
                })
                .collect();

            Expr::Array(ArrayLit {
                span: DUMMY_SP,
                elems: elements,
            })
        }
        JsonValue::Object(obj) => {
            let props = obj
                .iter()
                .map(|(k, v)| {
                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(IdentName {
                            span: DUMMY_SP,
                            sym: k.clone().into(),
                        }),
                        value: Box::new(json_value_to_expr(v)),
                    })))
                })
                .collect();

            Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props,
            })
        }
    }
}

/// Converts a serde_json::Value to Vec<(String, Expr)> for object properties
fn json_to_props(json_value: &JsonValue) -> Vec<(String, Expr)> {
    match json_value {
        JsonValue::Object(obj) => obj
            .iter()
            .map(|(k, v)| (k.clone(), json_value_to_expr(v)))
            .collect(),
        _ => vec![], // Return empty vec if not an object
    }
}

/// Converts a serializable struct to Vec<(String, Expr)> for object properties
fn struct_to_props<T: Serialize>(value: &T) -> Vec<(String, Expr)> {
    match serde_json::to_value(value) {
        Ok(json_value) => json_to_props(&json_value),
        Err(_) => vec![], // Return empty vec on serialization error
    }
}

/// Creates a statement calling a function with a serializable struct as its argument
fn create_function_with_struct_stmt<T: Serialize>(function_name: &str, value: &T) -> Stmt {
    let props = struct_to_props(value);
    create_function_with_props_stmt(function_name, props)
}



// Helper function to convert strings to PascalCase
fn convert_to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '-' || c == '_' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}
