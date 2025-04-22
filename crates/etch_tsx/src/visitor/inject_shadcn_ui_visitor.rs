use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use swc_atoms::Atom;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct OpenModalOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct CloseModalOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct ToggleModalOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct OpenSheetOptions {
    pub id: String,
    pub side: Option<String>, // "top", "right", "bottom", "left"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct CloseSheetOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct ShowToastOptions {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct SelectTabOptions {
    pub tabGroupId: String,
    pub tabId: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct ToggleAccordionOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct OpenDropdownOptions {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct CloseDropdownOptions {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum ComponentWrapper {
    Dialog(DialogOptions),
    HoverCard(HoverCardOptions),
    Popover(PopoverOptions),
    Sheet(SheetOptions),
    Tooltip(TooltipOptions),
    Link(LinkOptions),
    Drawer(DrawerOptions),
    Button(ButtonOptions),
}

/// A visitor that adds event handlers to JSX elements and transforms JSX structure
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub struct InjectShadcnUiVisitor {
    pub component_wrappers: HashMap<String, ComponentWrapper>, // Generic wrapper mapping
    pub action_imports: HashMap<String, HashSet<String>>,      // Maps import paths to action names
}

impl InjectShadcnUiVisitor {
    pub fn new(
        component_wrappers: HashMap<String, ComponentWrapper>,
        action_imports: HashMap<String, HashSet<String>>,
    ) -> Self {
        Self {
            component_wrappers,
            action_imports,
        }
    }

    /// Register an element to be wrapped with a UI component
    pub fn register_component_wrapper(&mut self, id: String, wrapper: ComponentWrapper) {
        self.component_wrappers.insert(id, wrapper);
    }

    /// Helper to check if element has any wrapper and return it
    fn get_component_wrapper(&self, id: &str) -> Option<&ComponentWrapper> {
        self.component_wrappers.get(id)
    }

    /// Register an action with its import path
    pub fn register_action_import(&mut self, action_name: &str, import_path: &str) {
        self.action_imports
            .entry(import_path.to_string())
            .or_default()
            .insert(action_name.to_string());
    }

    /// Register default import paths for all actions
    pub fn register_default_imports(&mut self) {
        // UI component actions organized by component type

        // Dialog components
        let dialog_path = "@/components/ui/dialog";
        self.register_action_import("Dialog", dialog_path);
        self.register_action_import("DialogTrigger", dialog_path);
        self.register_action_import("DialogContent", dialog_path);
        self.register_action_import("DialogHeader", dialog_path);
        self.register_action_import("DialogTitle", dialog_path);
        self.register_action_import("DialogDescription", dialog_path);
        self.register_action_import("DialogFooter", dialog_path);

        // Sheet/Drawer actions

        // Sheet/Drawer actions
        let sheet_path = "@/components/ui/sheet";
        self.register_action_import("Sheet", sheet_path);
        self.register_action_import("SheetClose", sheet_path);
        self.register_action_import("SheetContent", sheet_path);
        self.register_action_import("SheetDescription", sheet_path);
        self.register_action_import("SheetHeader", sheet_path);
        self.register_action_import("SheetTitle", sheet_path);
        self.register_action_import("SheetTrigger", sheet_path);
        self.register_action_import("SheetHeader", sheet_path);

        let drawer_path = "@/components/ui/drawer";
        self.register_action_import("Drawer", drawer_path);
        self.register_action_import("DrawerClose", drawer_path);
        self.register_action_import("DrawerContent", drawer_path);
        self.register_action_import("DrawerDescription", drawer_path);
        self.register_action_import("DrawerFooter", drawer_path);
        self.register_action_import("DrawerHeader", drawer_path);
        self.register_action_import("DrawerTitle", drawer_path);
        self.register_action_import("DrawerTrigger", drawer_path);

        // Popover actions
        let popover_path = "@/components/ui/popover";
        self.register_action_import("Popover", popover_path);
        self.register_action_import("PopoverContent", popover_path);
        self.register_action_import("PopoverTrigger", popover_path);

        //HoverCard actions
        let hovercard_path = "@/components/ui/hover-card";
        self.register_action_import("HoverCard", hovercard_path);
        self.register_action_import("HoverCardContent", hovercard_path);
        self.register_action_import("HoverCardTrigger", hovercard_path);

        // Toast actions
        let sonner_path = "sonner";
        self.register_action_import("toast", sonner_path);

        // Also register Button if not already registered
        let button_path = "@/components/ui/button";
        self.register_action_import("Button", button_path);

        // Add more action imports as needed
        // For example, navigation actions might come from a different path
        // self.register_action_import("navigate", "@/lib/navigation");
    }

    /// Get all required import statements based on used actions
    fn get_action_imports(&self, used_actions: &HashSet<String>) -> Vec<ModuleItem> {
        let mut imports = Vec::new();

        for (import_path, actions) in &self.action_imports {
            // Filter to only include actions that are actually used
            let mut used_from_this_path: Vec<String> = actions
                .iter()
                .filter(|action| used_actions.contains(*action))
                .cloned()
                .collect();

            if !used_from_this_path.is_empty() {
                // Sort for consistent output
                used_from_this_path.sort();

                // Create named imports
                let named_imports = used_from_this_path
                    .iter()
                    .map(|action| {
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
                    })
                    .collect();

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

impl VisitMut for InjectShadcnUiVisitor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        // Set up default imports if none configured
        if self.action_imports.is_empty() {
            self.register_default_imports();
        }

        // First collect all used actions
        let mut used_actions = HashSet::new();

        // From component wrappers
        for wrapper in self.component_wrappers.values() {
            match wrapper {
                ComponentWrapper::Dialog(_) => {
                    // Add dialog-related components to used actions
                    used_actions.insert("Dialog".to_string());
                    used_actions.insert("DialogTrigger".to_string());
                    used_actions.insert("DialogContent".to_string());
                    used_actions.insert("DialogHeader".to_string());
                    used_actions.insert("DialogTitle".to_string());
                    used_actions.insert("DialogDescription".to_string());
                    used_actions.insert("DialogFooter".to_string());
                    used_actions.insert("Button".to_string());
                }
                ComponentWrapper::Drawer(_) => {
                    // Add drawer-related components to used actions
                    used_actions.insert("Drawer".to_string());
                    used_actions.insert("DrawerClose".to_string());
                    used_actions.insert("DrawerContent".to_string());
                    used_actions.insert("DrawerDescription".to_string());
                    used_actions.insert("DrawerFooter".to_string());
                    used_actions.insert("DrawerHeader".to_string());
                    used_actions.insert("DrawerTitle".to_string());
                    used_actions.insert("DrawerTrigger".to_string());
                    used_actions.insert("Button".to_string());
                }

                ComponentWrapper::Tooltip(_) => {
                    // Add tooltip-related components to used actions
                    used_actions.insert("Tooltip".to_string());
                    used_actions.insert("TooltipTrigger".to_string());
                    used_actions.insert("TooltipContent".to_string());
                    used_actions.insert("Button".to_string());
                }
                ComponentWrapper::Link(_) => {
                    // Add link-related components to used actions
                    used_actions.insert("Link".to_string());
                    used_actions.insert("Button".to_string());
                }
                ComponentWrapper::Sheet(_) => {
                    // Add sheet-related components to used actions
                    used_actions.insert("Sheet".to_string());
                    used_actions.insert("SheetTrigger".to_string());
                    used_actions.insert("SheetContent".to_string());
                    used_actions.insert("SheetHeader".to_string());
                    used_actions.insert("SheetTitle".to_string());
                    used_actions.insert("SheetDescription".to_string());
                    used_actions.insert("Button".to_string());
                }
                ComponentWrapper::Popover(_) => {
                    // Add popover-related components to used actions
                    used_actions.insert("Popover".to_string());
                    used_actions.insert("PopoverTrigger".to_string());
                    used_actions.insert("PopoverContent".to_string());
                    used_actions.insert("Button".to_string());
                    used_actions.insert("Button".to_string());
                }
                ComponentWrapper::HoverCard(_) => {
                    // Add hover card-related components to used actions
                    used_actions.insert("HoverCard".to_string());
                    used_actions.insert("HoverCardTrigger".to_string());
                    used_actions.insert("HoverCardContent".to_string());
                    used_actions.insert("Button".to_string());
                }
                ComponentWrapper::Button(_) => {
                    // Add button-related components to used actions
                    used_actions.insert("Button".to_string());
                } // Add more component types as needed
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

        // Check if this element has any component wrapper
        if let Some(id) = element_id.as_ref() {
            if let Some(wrapper) = self.get_component_wrapper(id) {
                // Transform the element based on wrapper type
                let original_element = std::mem::replace(node, create_empty_jsx_element());

                match wrapper {
                    ComponentWrapper::Dialog(options) => {
                        *node = options.generate_component(original_element);
                    }
                    ComponentWrapper::HoverCard(options) => {
                        *node = options.generate_component(original_element);
                    }
                    ComponentWrapper::Popover(options) => {
                        *node = options.generate_component(original_element);
                    }
                    ComponentWrapper::Sheet(options) => {
                        *node = options.generate_component(original_element);
                    }
                    ComponentWrapper::Tooltip(options) => {
                        *node = options.generate_component(original_element);
                    }
                    ComponentWrapper::Link(options) => {
                        *node = options.generate_component(original_element);
                    }
                    ComponentWrapper::Drawer(options) => {
                        *node = options.generate_component(original_element);
                    }

                    // Handle other component types
                    _ => {
                        *node = original_element; // Just restore if not implemented
                    }
                }

                // Don't add event handlers after wrapping
                return;
            }
        }

        // Visit children
        node.visit_mut_children_with(self);
    }
}

/// Creates an empty JSX element as a placeholder
fn create_empty_jsx_element() -> JSXElement {
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
            attrs: vec![],
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

/// Trait for component generators
pub trait ComponentGenerator {
    fn generate_component(&self, trigger_element: JSXElement) -> JSXElement;
}

impl ComponentGenerator for DialogOptions {
    fn generate_component(&self, trigger_element: JSXElement) -> JSXElement {
        create_dialog_component(trigger_element, self)
    }
}

impl ComponentGenerator for HoverCardOptions {
    fn generate_component(&self, trigger_element: JSXElement) -> JSXElement {
        create_hover_card_component(trigger_element, self)
    }
}

impl ComponentGenerator for PopoverOptions {
    fn generate_component(&self, trigger_element: JSXElement) -> JSXElement {
        create_popover_component(trigger_element, self)
    }
}

impl ComponentGenerator for SheetOptions {
    fn generate_component(&self, trigger_element: JSXElement) -> JSXElement {
        create_sheet_component(trigger_element, self)
    }
}

impl ComponentGenerator for TooltipOptions {
    fn generate_component(&self, trigger_element: JSXElement) -> JSXElement {
        create_tooltip_component(trigger_element, self)
    }
}

impl ComponentGenerator for LinkOptions {
    fn generate_component(&self, trigger_element: JSXElement) -> JSXElement {
        create_link_component(trigger_element, self)
    }
}

impl ComponentGenerator for DrawerOptions {
    fn generate_component(&self, trigger_element: JSXElement) -> JSXElement {
        create_drawer_component(trigger_element, self)
    }
}

/// Creates a Dialog component with the original element as the trigger
fn create_dialog_component(trigger_element: JSXElement, options: &DialogOptions) -> JSXElement {
    // 1. Create Dialog root element
    let mut dialog_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Dialog".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        },
        children: vec![],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Dialog".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // 2. Create DialogTrigger with asChild prop
    let trigger_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "DialogTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "asChild".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: None, // Boolean attribute with no value
            })],
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "DialogTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // 3. Create DialogContent
    let mut content_children = Vec::new();

    // Add DialogHeader with Title and Description
    if options.title.is_some() || options.description.is_some() {
        let mut header_children = Vec::new();

        // Add DialogTitle if provided
        if let Some(title) = &options.title {
            header_children.push(JSXElementChild::JSXElement(Box::new(dangerous_html_node(title.clone()))));
        }

        // Add DialogDescription if provided
        if let Some(description) = &options.description {
            header_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "DialogDescription".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: vec![],
                    self_closing: false,
                    type_args: None,
                },
                children: vec![JSXElementChild::JSXText(JSXText {
                    span: DUMMY_SP,
                    value: description.clone().into(),
                    raw: Atom::default(),
                })],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "DialogDescription".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                }),
            })));
        }

        // Add the header to content
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "DialogHeader".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![],
                self_closing: false,
                type_args: None,
            },
            children: header_children,
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "DialogHeader".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        })));
    }

    // Add custom content if provided
    if let Some(content) = &options.content {
        content_children.push(JSXElementChild::JSXElement(Box::new(dangerous_html_node(content.clone()))));
    }

    // Add footer with buttons if specified
    if options.has_footer.unwrap_or(false) {
        let mut footer_children = Vec::new();

        if let Some(buttons) = &options.footer_buttons {
            for button in buttons {
                footer_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "Button".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                        attrs: if let Some(variant) = &button.variant {
                            vec![
                                JSXAttrOrSpread::JSXAttr(JSXAttr {
                                    span: DUMMY_SP,
                                    name: JSXAttrName::Ident(
                                        Ident {
                                            span: DUMMY_SP,
                                            sym: "variant".into(),
                                            optional: false,
                                            ctxt: SyntaxContext::empty(),
                                        }
                                        .into(),
                                    ),
                                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                        span: DUMMY_SP,
                                        value: variant.clone().into(),
                                        raw: None,
                                    }))),
                                }),
                                JSXAttrOrSpread::JSXAttr(JSXAttr {
                                    span: DUMMY_SP,
                                    name: JSXAttrName::Ident(
                                        Ident {
                                            span: DUMMY_SP,
                                            sym: "type".into(),
                                            optional: false,
                                            ctxt: SyntaxContext::empty(),
                                        }
                                        .into(),
                                    ),
                                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                        span: DUMMY_SP,
                                        value: "button".into(),
                                        raw: None,
                                    }))),
                                }),
                            ]
                        } else {
                            vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                                span: DUMMY_SP,
                                name: JSXAttrName::Ident(
                                    Ident {
                                        span: DUMMY_SP,
                                        sym: "type".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into(),
                                ),
                                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "button".into(),
                                    raw: None,
                                }))),
                            })]
                        },
                        self_closing: false,
                        type_args: None,
                    },
                    children: vec![JSXElementChild::JSXText(JSXText {
                        span: DUMMY_SP,
                        value: button.label.clone().into(),
                        raw: Atom::default(),
                    })],
                    closing: Some(JSXClosingElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "Button".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                    }),
                })));
            }
        }

        // Add the footer to content
        if !footer_children.is_empty() {
            content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "DialogFooter".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: vec![],
                    self_closing: false,
                    type_args: None,
                },
                children: footer_children,
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "DialogFooter".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                }),
            })));
        }
    }

    // Create DialogContent element with all content
    let content_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "DialogContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "className".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: "sm:max-w-[425px]".into(),
                    raw: None,
                }))),
            })],
            self_closing: false,
            type_args: None,
        },
        children: content_children,
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "DialogContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Add the trigger and content to the dialog
    dialog_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(trigger_jsx)));
    dialog_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(content_jsx)));

    dialog_jsx
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct DialogButton {
    pub label: String,
    pub variant: Option<String>, // "default", "destructive", "outline", etc.
    pub action: Option<String>,  // Function to call when clicked
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct DialogOptions {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub has_footer: Option<bool>,
    pub footer_buttons: Option<Vec<DialogButton>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct HoverCardOptions {
    pub id: String,
    pub trigger_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub open_delay: Option<u32>,
    pub close_delay: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct PopoverOptions {
    pub id: String,
    pub trigger_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub alignment: Option<String>, // "start", "center", "end"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct SheetOptions {
    pub id: String,
    pub trigger_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub side: Option<String>, // "top", "right", "bottom", "left"
    pub content: Option<String>,
    pub has_footer: Option<bool>,
    pub footer_buttons: Option<Vec<SheetButton>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct SheetButton {
    pub label: String,
    pub variant: Option<String>, // "default", "destructive", "outline", etc.
    pub action: Option<String>,  // Function to call when clicked
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct SonnerOptions {
    pub id: String,
    pub position: Option<String>, // "top-left", "top-right", "bottom-left", "bottom-right"
    pub theme: Option<String>,    // "light", "dark", "system"
    pub expand: Option<bool>,
    pub close_button: Option<bool>,
    pub offset: Option<String>, // Custom offset from edges
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct ToastOptions {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub variant: Option<String>, // "default", "destructive", etc.
    pub action_label: Option<String>,
    pub action: Option<String>, // Function to call when action button clicked
    pub duration: Option<u32>,  // Duration in milliseconds
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct TooltipOptions {
    pub id: String,
    pub trigger_id: Option<String>,
    pub content: String,
    pub side: Option<String>,  // "top", "right", "bottom", "left"
    pub align: Option<String>, // "start", "center", "end"
    pub delay_duration: Option<u32>,
    pub skip_delay_duration: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct LinkOptions {
    pub id: String,
    pub href: String,
    pub target: Option<String>,  // "_blank", "_self", etc.
    pub rel: Option<String>,     // "noopener", "noreferrer", etc.
    pub as_button: Option<bool>, // Whether to style as a button
    pub variant: Option<String>, // If as_button is true: "default", "destructive", etc.
    pub size: Option<String>,    // If as_button is true: "default", "sm", "lg"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct ButtonOptions {
    pub id: String,
    pub label: String,
    pub variant: Option<String>, // "default", "destructive", "outline", etc.
    pub action: Option<String>,  // Function to call when clicked
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct DrawerOptions {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

/// Creates a HoverCard component with the original element as the trigger
fn create_hover_card_component(
    trigger_element: JSXElement,
    options: &HoverCardOptions,
) -> JSXElement {
    // Create HoverCard root element
    let mut hover_card_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "HoverCard".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        },
        children: vec![],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "HoverCard".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Add delay attributes if specified
    if let Some(open_delay) = options.open_delay {
        hover_card_jsx
            .opening
            .attrs
            .push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "openDelay".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num(Number {
                        span: DUMMY_SP,
                        value: open_delay as f64,
                        raw: None,
                    })))),
                })),
            }));
    }

    if let Some(close_delay) = options.close_delay {
        hover_card_jsx
            .opening
            .attrs
            .push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "closeDelay".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num(Number {
                        span: DUMMY_SP,
                        value: close_delay as f64,
                        raw: None,
                    })))),
                })),
            }));
    }

    // Create HoverCardTrigger with asChild prop
    let trigger_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "HoverCardTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "asChild".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: None, // Boolean attribute with no value
            })],
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "HoverCardTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Create HoverCardContent
    let mut content_children = Vec::new();

    // Add title if provided
    if let Some(title) = &options.title {
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
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
                            sym: "className".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }
                        .into(),
                    ),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: "font-medium".into(),
                        raw: None,
                    }))),
                })],
                self_closing: false,
                type_args: None,
            },
            children: vec![JSXElementChild::JSXText(JSXText {
                span: DUMMY_SP,
                value: title.clone().into(),
                raw: Atom::default(),
            })],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "div".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        })));
    }

    // Add description if provided
    if let Some(description) = &options.description {
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "p".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(
                        Ident {
                            span: DUMMY_SP,
                            sym: "className".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }
                        .into(),
                    ),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: "text-sm text-muted-foreground".into(),
                        raw: None,
                    }))),
                })],
                self_closing: false,
                type_args: None,
            },
            children: vec![JSXElementChild::JSXText(JSXText {
                span: DUMMY_SP,
                value: description.clone().into(),
                raw: Atom::default(),
            })],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "p".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        })));
    }

    // Add custom content if provided
    if let Some(content) = &options.content {
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
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
                            props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                KeyValueProp {
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
                                },
                            )))],
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
        })));
    }

    // Create content element with all content
    let content_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "HoverCardContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "className".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: "w-80 p-4".into(),
                    raw: None,
                }))),
            })],
            self_closing: false,
            type_args: None,
        },
        children: content_children,
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "HoverCardContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Add the trigger and content to the hover card
    hover_card_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(trigger_jsx)));
    hover_card_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(content_jsx)));

    hover_card_jsx
}

/// Creates a Popover component with the original element as the trigger
fn create_popover_component(trigger_element: JSXElement, options: &PopoverOptions) -> JSXElement {
    // Create Popover root element
    let mut popover_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Popover".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        },
        children: vec![],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Popover".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Create PopoverTrigger with asChild prop
    let trigger_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "PopoverTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "asChild".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: None, // Boolean attribute with no value
            })],
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "PopoverTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Create PopoverContent
    let mut content_children = Vec::new();

    // Add title if provided
    if let Some(title) = &options.title {
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
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
                            sym: "className".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }
                        .into(),
                    ),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: "font-medium pb-2".into(),
                        raw: None,
                    }))),
                })],
                self_closing: false,
                type_args: None,
            },
            children: vec![JSXElementChild::JSXText(JSXText {
                span: DUMMY_SP,
                value: title.clone().into(),
                raw: Atom::default(),
            })],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "div".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        })));
    }

    // Add description if provided
    if let Some(description) = &options.description {
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "p".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(
                        Ident {
                            span: DUMMY_SP,
                            sym: "className".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }
                        .into(),
                    ),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: "text-sm text-muted-foreground".into(),
                        raw: None,
                    }))),
                })],
                self_closing: false,
                type_args: None,
            },
            children: vec![JSXElementChild::JSXText(JSXText {
                span: DUMMY_SP,
                value: description.clone().into(),
                raw: Atom::default(),
            })],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "p".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        })));
    }

    // Add custom content if provided
    if let Some(content) = &options.content {
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
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
                            props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                KeyValueProp {
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
                                },
                            )))],
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
        })));
    }

    // Create content element with alignment
    let mut content_attrs = vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(
            Ident {
                span: DUMMY_SP,
                sym: "className".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }
            .into(),
        ),
        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: "w-80 p-4".into(),
            raw: None,
        }))),
    })];

    // Add alignment if specified
    if let Some(alignment) = &options.alignment {
        content_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(
                Ident {
                    span: DUMMY_SP,
                    sym: "align".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }
                .into(),
            ),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: alignment.clone().into(),
                raw: None,
            }))),
        }));
    }

    let content_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "PopoverContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: content_attrs,
            self_closing: false,
            type_args: None,
        },
        children: content_children,
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "PopoverContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Add the trigger and content to the popover
    popover_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(trigger_jsx)));
    popover_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(content_jsx)));

    popover_jsx
}

/// Creates a Sheet component with the original element as the trigger
fn create_sheet_component(trigger_element: JSXElement, options: &SheetOptions) -> JSXElement {
    // Create Sheet root element
    let mut sheet_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Sheet".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        },
        children: vec![],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Sheet".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Create SheetTrigger with asChild prop
    let trigger_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "SheetTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "asChild".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: None, // Boolean attribute with no value
            })],
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "SheetTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Create content attributes with side if specified
    let mut content_attrs = vec![];

    if let Some(side) = &options.side {
        content_attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(
                Ident {
                    span: DUMMY_SP,
                    sym: "side".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }
                .into(),
            ),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: side.clone().into(),
                raw: None,
            }))),
        }));
    }

    // Create content with header and body
    let mut content_children = Vec::new();

    // Add SheetHeader with Title and Description
    if options.title.is_some() || options.description.is_some() {
        let mut header_children = Vec::new();

        if let Some(title) = &options.title {
            header_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "SheetTitle".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: vec![],
                    self_closing: false,
                    type_args: None,
                },
                children: vec![JSXElementChild::JSXText(JSXText {
                    span: DUMMY_SP,
                    value: title.clone().into(),
                    raw: Atom::default(),
                })],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "SheetTitle".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                }),
            })));
        }

        if let Some(description) = &options.description {
            header_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "SheetDescription".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: vec![],
                    self_closing: false,
                    type_args: None,
                },
                children: vec![JSXElementChild::JSXText(JSXText {
                    span: DUMMY_SP,
                    value: description.clone().into(),
                    raw: Atom::default(),
                })],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "SheetDescription".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                }),
            })));
        }

        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "SheetHeader".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![],
                self_closing: false,
                type_args: None,
            },
            children: header_children,
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "SheetHeader".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        })));
    }

    // Add custom content if provided
    if let Some(content) = &options.content {
        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
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
                            props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                KeyValueProp {
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
                                },
                            )))],
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
        })));
    }

    // Create content element with all content
    let content_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "SheetContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: content_attrs,
            self_closing: false,
            type_args: None,
        },
        children: content_children,
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "SheetContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Add the trigger and content to the sheet
    sheet_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(trigger_jsx)));
    sheet_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(content_jsx)));

    sheet_jsx
}

/// Creates a Tooltip component with the original element as the trigger
fn create_tooltip_component(trigger_element: JSXElement, options: &TooltipOptions) -> JSXElement {
    // Implementation would create a Tooltip component with the given options
    // For now just returning a stub
    create_empty_jsx_element()
}

/// Creates a Link component with the original element as the trigger
fn create_link_component(trigger_element: JSXElement, options: &LinkOptions) -> JSXElement {
    // Implementation would create a Link component with the given options
    // For now just returning a stub
    create_empty_jsx_element()
}

/// Creates a Drawer component with the original element as the trigger
fn create_drawer_component(trigger_element: JSXElement, options: &DrawerOptions) -> JSXElement {
    // Create Drawer root element
    let mut drawer_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Drawer".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        },
        children: vec![],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "Drawer".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Create DrawerTrigger with asChild prop
    let trigger_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "DrawerTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "asChild".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: None, // Boolean attribute with no value
            })],
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(trigger_element))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "DrawerTrigger".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Create content with header and body
    let mut content_children = Vec::new();

    // Add DrawerHeader with Title and Description
    if options.title.is_some() || options.description.is_some() {
        let mut header_children = Vec::new();

        if let Some(title) = &options.title {
            header_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "DrawerTitle".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: vec![],
                    self_closing: false,
                    type_args: None,
                },
                children: vec![JSXElementChild::JSXText(JSXText {
                    span: DUMMY_SP,
                    value: title.clone().into(),
                    raw: Atom::default(),
                })],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "DrawerTitle".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                }),
            })));
        }

        if let Some(description) = &options.description {
            header_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "DrawerDescription".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: vec![],
                    self_closing: false,
                    type_args: None,
                },
                children: vec![JSXElementChild::JSXText(JSXText {
                    span: DUMMY_SP,
                    value: description.clone().into(),
                    raw: Atom::default(),
                })],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "DrawerDescription".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                }),
            })));
        }

        content_children.push(JSXElementChild::JSXElement(Box::new(JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "DrawerHeader".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![],
                self_closing: false,
                type_args: None,
            },
            children: header_children,
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "DrawerHeader".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        })));
    }

    // Add a basic footer with a close button
    let footer_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "DrawerFooter".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        },
        children: vec![JSXElementChild::JSXElement(Box::new(JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "DrawerClose".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(
                        Ident {
                            span: DUMMY_SP,
                            sym: "asChild".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }
                        .into(),
                    ),
                    value: None,
                })],
                self_closing: false,
                type_args: None,
            },
            children: vec![JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "Button".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(
                            Ident {
                                span: DUMMY_SP,
                                sym: "variant".into(),
                                optional: false,
                                ctxt: SyntaxContext::empty(),
                            }
                            .into(),
                        ),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: "outline".into(),
                            raw: None,
                        }))),
                    })],
                    self_closing: false,
                    type_args: None,
                },
                children: vec![JSXElementChild::JSXText(JSXText {
                    span: DUMMY_SP,
                    value: "Close".into(),
                    raw: Atom::default(),
                })],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "Button".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                }),
            }))],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "DrawerClose".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        }))],
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "DrawerFooter".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    content_children.push(JSXElementChild::JSXElement(Box::new(footer_jsx)));

    // Create content element with all content
    let content_jsx = JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "DrawerContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![],
            self_closing: false,
            type_args: None,
        },
        children: content_children,
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: "DrawerContent".into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    };

    // Add the trigger and content to the drawer
    drawer_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(trigger_jsx)));
    drawer_jsx
        .children
        .push(JSXElementChild::JSXElement(Box::new(content_jsx)));

    drawer_jsx
}

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
                        props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(
                            KeyValueProp {
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
                            },
                        )))],
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
