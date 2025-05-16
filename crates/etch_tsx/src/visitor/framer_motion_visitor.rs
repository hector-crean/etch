use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnimationConfig<
    Initial = serde_json::Value,
    Animate = serde_json::Value,
    Exit = serde_json::Value,
    Variant = serde_json::Value,
    Transition = serde_json::Value,
> where
    Initial: TS,
    Animate: TS,
    Exit: TS,
    Variant: TS,
    Transition: TS,
{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial: Option<Initial>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animate: Option<Animate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit: Option<Exit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Variant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,
    // Add a flag to control whether animations should be inherited by children
    #[serde(default = "default_inherit_children")]
    pub inherit_children: bool,
}

impl<Initial, Animate, Exit, Variant, Transition> Default 
    for AnimationConfig<Initial, Animate, Exit, Variant, Transition>
where
    Initial: TS,
    Animate: TS,
    Exit: TS,
    Variant: TS,
    Transition: TS,
{
    fn default() -> Self {
        Self {
            initial: None,
            animate: None,
            exit: None,
            variants: None,
            transition: None,
            inherit_children: default_inherit_children(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub struct FramerMotionVisitor<
    Initial = serde_json::Value,
    Animate = serde_json::Value,
    Exit = serde_json::Value,
    Variant = serde_json::Value,
    Transition = serde_json::Value,
> where
    Initial: TS,
    Animate: TS,
    Exit: TS,
    Variant: TS,
    Transition: TS,
{
    pub animations: HashMap<String, AnimationConfig<Initial, Animate, Exit, Variant, Transition>>,
    // Track current parent element for animation inheritance
    current_parent_id: Option<String>,
    current_parent_config: Option<AnimationConfig<Initial, Animate, Exit, Variant, Transition>>,
    // Track whether we're inside an SVG element
    inside_svg: bool,
}

// Default function for inherit_children field
fn default_inherit_children() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum AnimationType {
    PathDrawing,
    FadeIn,
    Scale,
    // Add more animation types as needed
}

type ElementId = String;

impl<Initial, Animate, Exit, Variant, Transition>
    FramerMotionVisitor<Initial, Animate, Exit, Variant, Transition>
where
    Initial: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Animate: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Exit: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Variant: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Transition: TS + Clone + Serialize + for<'de> Deserialize<'de>,
{
    pub fn new(
        animations: HashMap<ElementId, AnimationConfig<Initial, Animate, Exit, Variant, Transition>>,
    ) -> Self {
        Self {
            animations,
            current_parent_id: None,
            current_parent_config: None,
            inside_svg: false,
        }
    }

    pub fn register_animation(
        &mut self,
        element_id: String,
        config: AnimationConfig<Initial, Animate, Exit, Variant, Transition>,
    ) {
        self.animations.insert(element_id, config);
    }
}

impl<Initial, Animate, Exit, Variant, Transition> VisitMut
    for FramerMotionVisitor<Initial, Animate, Exit, Variant, Transition>
where
    Initial: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Animate: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Exit: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Variant: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Transition: TS + Clone + Serialize + for<'de> Deserialize<'de>,
{
    fn visit_mut_module(&mut self, module: &mut Module) {
        // Reset the counter and parent tracking when starting a new module
        self.current_parent_id = None;
        self.current_parent_config = None;
        self.inside_svg = false;

        // First, add the import for framer motion
        let motion_import = ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
            span: DUMMY_SP,
            specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                span: DUMMY_SP,
                local: Ident {
                    span: DUMMY_SP,
                    sym: "motion".into(),
                    optional: false,
                    ctxt: Default::default(),
                },
                imported: None,
                is_type_only: false,
            })],
            src: Box::new(Str {
                span: DUMMY_SP,
                value: "framer-motion".into(),
                raw: None,
            }),
            type_only: false,
            with: None,
            phase: Default::default(),
        }));

        // Insert imports and declarations at the beginning of the module
        module.body.insert(0, motion_import);

        // Continue with the rest of the module
        module.visit_mut_children_with(self);
    }

    fn visit_mut_jsx_element(&mut self, node: &mut JSXElement) {
        // Process attributes to find the element ID
        let mut element_id = None;
       
        // First pass: find the ID attribute and check if it's an SVG element
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

        // Save the previous parent state to restore after processing children
        let previous_parent_id = self.current_parent_id.clone();
        let previous_parent_config = self.current_parent_config.clone();

        // Check if this element has a specific animation config
        let has_direct_config = if let Some(id) = &element_id {
            self.animations.contains_key(id)
        } else {
            false
        };

        // Only apply animations if this element has a matching ID in our animations HashMap
        if has_direct_config {
            if let Some(id) = &element_id {
                if let Some(config) = self.animations.get(id).cloned() {
                    // Change the element name to motion.<element_name>
                    if let JSXElementName::Ident(ident) = &mut node.opening.name {
                        let tag_name = ident.sym.as_ref().to_string();
                        ident.sym = format!("motion.{}", tag_name).into();
                    }

                    if let Some(closing) = &mut node.closing {
                        if let JSXElementName::Ident(ident) = &mut closing.name {
                            let tag_name = ident.sym.as_ref().to_string();
                            ident.sym = format!("motion.{}", tag_name).into();
                        }
                    }

                    // Apply animation properties to this element
                    self.apply_animation_to_element(node, &config);

                    // Only set as parent if inheritance is enabled
                    if config.inherit_children {
                        self.current_parent_id = Some(id.clone());
                        self.current_parent_config = Some(config);
                    }
                }
            }
        } else if self.current_parent_config.is_some() {
            // Apply parent's animation if this element doesn't have its own config
            // and we're in a parent with inheritance enabled
            if let Some(config) = &self.current_parent_config {
                // Change the element name to motion.<element_name>
                if let JSXElementName::Ident(ident) = &mut node.opening.name {
                    let tag_name = ident.sym.as_ref().to_string();
                    ident.sym = format!("motion.{}", tag_name).into();
                }

                if let Some(closing) = &mut node.closing {
                    if let JSXElementName::Ident(ident) = &mut closing.name {
                        let tag_name = ident.sym.as_ref().to_string();
                        ident.sym = format!("motion.{}", tag_name).into();
                    }
                }
                
                self.apply_animation_to_element(node, config);
            }
        }

        // Continue with children
        node.visit_mut_children_with(self);
        
        // Restore the previous parent state after processing children
        self.current_parent_id = previous_parent_id;
        self.current_parent_config = previous_parent_config;
    }
}

// Add helper method to apply animation properties
impl<Initial, Animate, Exit, Variant, Transition>
    FramerMotionVisitor<Initial, Animate, Exit, Variant, Transition>
where
    Initial: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Animate: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Exit: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Variant: TS + Clone + Serialize + for<'de> Deserialize<'de>,
    Transition: TS + Clone + Serialize + for<'de> Deserialize<'de>,
{
    fn apply_animation_to_element(
        &self,
        node: &mut JSXElement,
        config: &AnimationConfig<Initial, Animate, Exit, Variant, Transition>,
    ) {
        // Convert the element to a motion element
        if let JSXElementName::Ident(ident) = &mut node.opening.name {
            let tag_name = ident.sym.as_ref().to_string();
            if !tag_name.starts_with("motion.") {
                ident.sym = format!("motion.{}", tag_name).into();
            }
        }

        if let Some(closing) = &mut node.closing {
            if let JSXElementName::Ident(ident) = &mut closing.name {
                let tag_name = ident.sym.as_ref().to_string();
                if !tag_name.starts_with("motion.") {
                    ident.sym = format!("motion.{}", tag_name).into();
                }
            }
        }

        // Add animation properties as JSX attributes
        if let Some(initial) = &config.initial {
            self.add_json_attribute(node, "initial", initial);
        }

        if let Some(animate) = &config.animate {
            self.add_json_attribute(node, "animate", animate);
        }

        if let Some(exit) = &config.exit {
            self.add_json_attribute(node, "exit", exit);
        }

        if let Some(transition) = &config.transition {
            self.add_json_attribute(node, "transition", transition);
        }
    }

    fn add_json_attribute<T: Serialize>(&self, node: &mut JSXElement, name: &str, value: &T) {
        // Convert the value to a JSX expression directly
        if let Ok(json_value) = serde_json::to_value(value) {
            let expr = self.json_value_to_expr(&json_value);
            
            // Add the attribute with the direct expression
            node.opening.attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: name.into(),
                        optional: false,
                        ctxt: Default::default(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(expr)),
                })),
            }));
        }
    }

    // Helper method to convert serde_json::Value to swc_ecma_ast::Expr
    fn json_value_to_expr(&self, value: &serde_json::Value) -> Expr {
        match value {
            serde_json::Value::Null => Expr::Lit(Lit::Null(Null { span: DUMMY_SP })),
            serde_json::Value::Bool(b) => Expr::Lit(Lit::Bool(Bool { span: DUMMY_SP, value: *b })),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Expr::Lit(Lit::Num(Number { span: DUMMY_SP, value: i as f64, raw: None }))
                } else if let Some(f) = n.as_f64() {
                    Expr::Lit(Lit::Num(Number { span: DUMMY_SP, value: f, raw: None }))
                } else {
                    // Fallback
                    Expr::Lit(Lit::Num(Number { span: DUMMY_SP, value: 0.0, raw: None }))
                }
            },
            serde_json::Value::String(s) => Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: s.clone().into(),
                raw: None,
            })),
            serde_json::Value::Array(arr) => {
                let elements = arr.iter()
                    .map(|v| Some(ExprOrSpread {
                        spread: None,
                        expr: Box::new(self.json_value_to_expr(v)),
                    }))
                    .collect();
                
                Expr::Array(ArrayLit {
                    span: DUMMY_SP,
                    elems: elements,
                })
            },
            serde_json::Value::Object(obj) => {
                let props = obj.iter()
                    .map(|(k, v)| PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Str(Str {
                            span: DUMMY_SP,
                            value: k.clone().into(),
                            raw: None,
                        }),
                        value: Box::new(self.json_value_to_expr(v)),
                    }))))
                    .collect();
                
                Expr::Object(ObjectLit {
                    span: DUMMY_SP,
                    props,
                })
            },
        }
    }
}
