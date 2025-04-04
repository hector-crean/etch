use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub struct FramerMotionVisitor {
    pub animations: HashMap<String, AnimationConfig>,
    // Add a counter to track the position in the SVG hierarchy
    custom_counter: usize,
    // Track current parent element for animation inheritance
    current_parent_id: Option<String>,
    current_parent_config: Option<AnimationConfig>,
    // Track whether we're inside an SVG element
    inside_svg: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnimationConfig {
    pub element_id: String,
    pub animation_type: AnimationType,
    pub custom_delay: Option<f64>,
    pub stroke_color: Option<String>,
    // Add a flag to control whether animations should be inherited by children
    #[serde(default = "default_inherit_children")]
    pub inherit_children: bool,
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

impl FramerMotionVisitor {
    pub fn new(animations: HashMap<String, AnimationConfig>) -> Self {
        Self {
            animations,
            custom_counter: 0,
            current_parent_id: None,
            current_parent_config: None,
            inside_svg: false,
        }
    }

    pub fn register_animation(&mut self, element_id: String, config: AnimationConfig) {
        self.animations.insert(element_id, config);
    }

    // Helper to create the draw variants object
    fn create_draw_variants_expr(&self) -> Expr {
        // Create the draw variants object with hidden and visible states
        Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props: vec![
                // hidden: { pathLength: 0, opacity: 0 }
                PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "hidden".into(),
                        optional: false,
                        ctxt: Default::default(),
                    }.into()),
                    value: Box::new(Expr::Object(ObjectLit {
                        span: DUMMY_SP,
                        props: vec![
                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                key: PropName::Ident(Ident {
                                    span: DUMMY_SP,
                                    sym: "pathLength".into(),
                                    optional: false,
                                    ctxt: Default::default(),
                                }.into()),
                                value: Box::new(Expr::Lit(Lit::Num(Number {
                                    span: DUMMY_SP,
                                    value: 0.0,
                                    raw: None,
                                }))),
                            }))),
                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                key: PropName::Ident(Ident {
                                    span: DUMMY_SP,
                                    sym: "opacity".into(),
                                    optional: false,
                                    ctxt: Default::default(),
                                }.into()),
                                value: Box::new(Expr::Lit(Lit::Num(Number {
                                    span: DUMMY_SP,
                                    value: 0.0,
                                    raw: None,
                                }))),
                            }))),
                        ],
                    })),
                }))),
                 // visible: (i) => { ... }
                 PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "visible".into(),
                        optional: false,
                        ctxt: Default::default(),
                    }.into()),
                    value: Box::new(Expr::Arrow(ArrowExpr {
                        ctxt: Default::default(),
                        span: DUMMY_SP,
                        params: vec![Pat::Ident(BindingIdent {
                            id: Ident {
                                span: DUMMY_SP,
                                sym: "i".into(),
                                optional: false,
                                ctxt: Default::default(),
                            },
                            type_ann: None,
                        })],
                        body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                            ctxt: Default::default(),
                            span: DUMMY_SP,
                            stmts: vec![
                                // const delay = i * 0.5
                                Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                    ctxt: Default::default(),
                                    span: DUMMY_SP,
                                    kind: VarDeclKind::Const,
                                    declare: false,
                                    decls: vec![VarDeclarator {
                                        span: DUMMY_SP,
                                        name: Pat::Ident(BindingIdent {
                                            id: Ident {
                                                span: DUMMY_SP,
                                                sym: "delay".into(),
                                                optional: false,
                                                ctxt: Default::default(),
                                            },
                                            type_ann: None,
                                        }),
                                        init: Some(Box::new(Expr::Bin(BinExpr {
                                            span: DUMMY_SP,
                                            op: BinaryOp::Mul,
                                            left: Box::new(Expr::Ident(Ident {
                                                span: DUMMY_SP,
                                                sym: "i".into(),
                                                optional: false,
                                                ctxt: Default::default(),
                                            })),
                                            right: Box::new(Expr::Lit(Lit::Num(Number {
                                                span: DUMMY_SP,
                                                value: 0.5,
                                                raw: None,
                                            }))),
                                        }))),
                                        definite: false,
                                    }],
                                }))),
                                // return { ... }
                                Stmt::Return(ReturnStmt {
                                    span: DUMMY_SP,
                                    arg: Some(Box::new(Expr::Object(ObjectLit {
                                        span: DUMMY_SP,
                                        props: vec![
                                            // pathLength: 1
                                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                key: PropName::Ident(Ident {
                                                    span: DUMMY_SP,
                                                    sym: "pathLength".into(),
                                                    optional: false,
                                                    ctxt: Default::default(),
                                                }.into()),
                                                value: Box::new(Expr::Lit(Lit::Num(Number {
                                                    span: DUMMY_SP,
                                                    value: 1.0,
                                                    raw: None,
                                                }))),
                                            }))),
                                            // opacity: 1
                                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                key: PropName::Ident(Ident {
                                                    span: DUMMY_SP,
                                                    sym: "opacity".into(),
                                                    optional: false,
                                                    ctxt: Default::default(),
                                                }.into()),
                                                value: Box::new(Expr::Lit(Lit::Num(Number {
                                                    span: DUMMY_SP,
                                                    value: 1.0,
                                                    raw: None,
                                                }))),
                                            }))),
                                            // transition: { ... }
                                            PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                key: PropName::Ident(Ident {
                                                    span: DUMMY_SP,
                                                    sym: "transition".into(),
                                                    optional: false,
                                                    ctxt: Default::default(),
                                                }.into()),
                                                value: Box::new(Expr::Object(ObjectLit {
                                                    span: DUMMY_SP,
                                                    props: vec![
                                                        // pathLength: { delay, type: "spring", duration: 1.5, bounce: 0 }
                                                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                            key: PropName::Ident(Ident {
                                                                span: DUMMY_SP,
                                                                sym: "pathLength".into(),
                                                                optional: false,
                                                                ctxt: Default::default(),
                                                            }.into()),
                                                            value: Box::new(Expr::Object(ObjectLit {
                                                                span: DUMMY_SP,
                                                                props: vec![
                                                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                                        key: PropName::Ident(Ident {
                                                                            span: DUMMY_SP,
                                                                            sym: "delay".into(),
                                                                            optional: false,
                                                                            ctxt: Default::default(),
                                                                        }.into()),
                                                                        value: Box::new(Expr::Ident(Ident {
                                                                            span: DUMMY_SP,
                                                                            sym: "delay".into(),
                                                                            optional: false,
                                                                            ctxt: Default::default(),
                                                                        })),
                                                                    }))),
                                                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                                        key: PropName::Ident(Ident {
                                                                            span: DUMMY_SP,
                                                                            sym: "type".into(),
                                                                            optional: false,
                                                                            ctxt: Default::default(),
                                                                        }.into()),
                                                                        value: Box::new(Expr::Lit(Lit::Str(Str {
                                                                            span: DUMMY_SP,
                                                                            value: "spring".into(),
                                                                            raw: None,
                                                                        }))),
                                                                    }))),
                                                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                                        key: PropName::Ident(Ident {
                                                                            span: DUMMY_SP,
                                                                            sym: "duration".into(),
                                                                            optional: false,
                                                                            ctxt: Default::default(),
                                                                        }.into()),
                                                                        value: Box::new(Expr::Lit(Lit::Num(Number {
                                                                            span: DUMMY_SP,
                                                                            value: 1.5,
                                                                            raw: None,
                                                                        }))),
                                                                    }))),
                                                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                                        key: PropName::Ident(Ident {
                                                                            span: DUMMY_SP,
                                                                            sym: "bounce".into(),
                                                                            optional: false,
                                                                            ctxt: Default::default(),
                                                                        }.into()),
                                                                        value: Box::new(Expr::Lit(Lit::Num(Number {
                                                                            span: DUMMY_SP,
                                                                            value: 0.0,
                                                                            raw: None,
                                                                        }))),
                                                                    }))),
                                                                ],
                                                            })),
                                                        }))),
                                                        // opacity: { delay, duration: 0.01 }
                                                        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                            key: PropName::Ident(Ident {
                                                                span: DUMMY_SP,
                                                                sym: "opacity".into(),
                                                                optional: false,
                                                                ctxt: Default::default(),
                                                            }.into()),
                                                            value: Box::new(Expr::Object(ObjectLit {
                                                                span: DUMMY_SP,
                                                                props: vec![
                                                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                                        key: PropName::Ident(Ident {
                                                                            span: DUMMY_SP,
                                                                            sym: "delay".into(),
                                                                            optional: false,
                                                                            ctxt: Default::default(),
                                                                        }.into()),
                                                                        value: Box::new(Expr::Ident(Ident {
                                                                            span: DUMMY_SP,
                                                                            sym: "delay".into(),
                                                                            optional: false,
                                                                            ctxt: Default::default(),
                                                                        })),
                                                                    }))),
                                                                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                                                        key: PropName::Ident(Ident {
                                                                            span: DUMMY_SP,
                                                                            sym: "duration".into(),
                                                                            optional: false,
                                                                            ctxt: Default::default(),
                                                                        }.into()),
                                                                        value: Box::new(Expr::Lit(Lit::Num(Number {
                                                                            span: DUMMY_SP,
                                                                            value: 0.01,
                                                                            raw: None,
                                                                        }))),
                                                                    }))),
                                                                ],
                                                            })),
                                                        }))),
                                                    ],
                                                })),
                                            }))),
                                        ],
                                    }))),
                                }),
                            ],
                        })),
                        is_async: false,
                        is_generator: false,
                        type_params: None,
                        return_type: None,
                    })),
                }))),
            ],
        })
    }

    // Helper to create the shape style object - we'll keep this for the module declaration
    // but won't apply it directly to elements
    fn create_shape_style_expr(&self) -> Expr {
        Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props: vec![
                PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "strokeWidth".into(),
                        optional: false,
                        ctxt: Default::default(),
                    }.into()),
                    value: Box::new(Expr::Lit(Lit::Num(Number {
                        span: DUMMY_SP,
                        value: 10.0,
                        raw: None,
                    }))),
                }))),
                PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "strokeLinecap".into(),
                        optional: false,
                        ctxt: Default::default(),
                    }.into()),
                    value: Box::new(Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: "round".into(),
                        raw: None,
                    }))),
                }))),
                PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "fill".into(),
                        optional: false,
                        ctxt: Default::default(),
                    }.into()),
                    value: Box::new(Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: "transparent".into(),
                        raw: None,
                    }))),
                }))),
            ],
        })
    }
}

impl VisitMut for FramerMotionVisitor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        // Reset the counter and parent tracking when starting a new module
        self.custom_counter = 0;
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
                value: "motion/react".into(),
                raw: None,
            }),
            type_only: false,
            with: None,
            phase: Default::default(),
        }));

        // Add the draw variants declaration
        let draw_variants_decl = ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
            ctxt: Default::default(),
            span: DUMMY_SP,
            kind: VarDeclKind::Const,
            declare: false,
            decls: vec![VarDeclarator {
                span: DUMMY_SP,
                name: Pat::Ident(BindingIdent {
                    id: Ident {
                        span: DUMMY_SP,
                        sym: "draw".into(),
                        optional: false,
                        ctxt: Default::default(),
                    },
                    type_ann: None,
                }),
                init: Some(Box::new(self.create_draw_variants_expr())),
                definite: false,
            }],
        }))));

        // Add the shape style declaration - we'll keep this for reference
        // but won't apply it directly to elements
        let shape_style_decl = ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
            ctxt: Default::default(),
            span: DUMMY_SP,
            kind: VarDeclKind::Const,
            declare: false,
            decls: vec![VarDeclarator {
                span: DUMMY_SP,
                name: Pat::Ident(BindingIdent {
                    id: Ident {
                        span: DUMMY_SP,
                        sym: "shape".into(),
                        optional: false,
                        ctxt: Default::default(),
                    },
                    type_ann: None,
                }),
                init: Some(Box::new(self.create_shape_style_expr())),
                definite: false,
            }],
        }))));

        // Insert imports and declarations at the beginning of the module
        module.body.insert(0, motion_import);
        module.body.insert(1, draw_variants_decl);
        module.body.insert(2, shape_style_decl);

        // Continue with the rest of the module
        module.visit_mut_children_with(self);
    }

    fn visit_mut_jsx_element(&mut self, node: &mut JSXElement) {
        // Process attributes to find the element ID
        let mut element_id = None;
        let mut is_svg_element = false;
        let mut is_svg_child = false;
        let mut is_group_element = false;

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

        // Check if this is an SVG element or a child of SVG
        if let JSXElementName::Ident(ident) = &node.opening.name {
            let tag_name = ident.sym.as_ref();
            is_svg_element = tag_name == "svg";
            is_group_element = tag_name == "g";
            is_svg_child = tag_name == "path" || tag_name == "circle" || tag_name == "rect" 
                || tag_name == "line" || tag_name == "polyline" || tag_name == "polygon" || tag_name == "g";
        }

        // If this is an SVG element, transform it to motion.svg and update our state
        if is_svg_element {
            self.inside_svg = true;
            
            // Change the element name to motion.svg
            if let JSXElementName::Ident(ident) = &mut node.opening.name {
                ident.sym = "motion.svg".into();
            }
            
            if let Some(closing) = &mut node.closing {
                if let JSXElementName::Ident(ident) = &mut closing.name {
                    ident.sym = "motion.svg".into();
                }
            }

            // Add initial and animate attributes
            node.opening.attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "initial".into(),
                    optional: false,
                    ctxt: Default::default(),
                }.into()),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: "hidden".into(),
                    raw: None,
                }))),
            }));

            node.opening.attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "animate".into(),
                    optional: false,
                    ctxt: Default::default(),
                }.into()),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: "visible".into(),
                    raw: None,
                }))),
            }));
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
        
        // If this element has a direct animation config, update the current parent
        if has_direct_config  {
            if let Some(id) = &element_id {
                if let Some(config) = self.animations.get(id).cloned() {
                    // Only set as parent if inheritance is enabled
                    if config.inherit_children {
                        self.current_parent_id = Some(id.clone());
                        self.current_parent_config = Some(config.clone());
                    }
                }
            }
           
        }
        
        // If this is an SVG child element that needs animation (either directly or inherited)
        if self.inside_svg && is_svg_child {
            // Increment the counter for each SVG child element
            self.custom_counter += 1;
            let current_counter = self.custom_counter;
            
            // Determine if this element should be animated
            let should_animate = has_direct_config || 
                // Apply parent's animation if we're inside an SVG and this is a child element
                (self.current_parent_config.is_some() && 
                 // Don't apply parent animation to elements with their own config
                 !has_direct_config);
            
            // Only apply animation if it's configured (directly or inherited)
            if should_animate {
                // Change the element name to motion.{tagName}
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

                // Add variants attribute
                node.opening.attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "variants".into(),
                        optional: false,
                        ctxt: Default::default(),
                    }.into() ),
                    value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                        span: DUMMY_SP,
                        expr: JSXExpr::Expr(Box::new(Expr::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "draw".into(),
                            optional: false,
                            ctxt: Default::default(),
                        }))),
                    })),
                }));
                
                // Get the appropriate config (direct or inherited)
                let config = if has_direct_config {
                    if let Some(id) = &element_id {
                        self.animations.get(id).cloned()
                    } else {
                        None
                    }
                } else {
                    self.current_parent_config.clone()
                };
                
                // Apply the animation config
                if let Some(config) = config {
                    // Add custom delay if specified
                    if let Some(delay) = config.custom_delay {
                        node.opening.attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(Ident {
                                span: DUMMY_SP,
                                sym: "custom".into(),
                                optional: false,
                                ctxt: Default::default(),
                            }.into()),
                            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                span: DUMMY_SP,
                                expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num(Number {
                                    span: DUMMY_SP,
                                    value: delay,
                                    raw: None,
                                })))),
                            })),
                        }));
                    } else {
                        // Use the counter value instead of default 1.0
                        node.opening.attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(Ident {
                                span: DUMMY_SP,
                                sym: "custom".into(),
                                optional: false,
                                ctxt: Default::default(),
                            }.into()),
                            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                span: DUMMY_SP,
                                expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num(Number {
                                    span: DUMMY_SP,
                                    value: current_counter as f64,
                                    raw: None,
                                })))),
                            })),
                        }));
                    }

                    // Add stroke color if specified
                    if let Some(color) = &config.stroke_color {
                        node.opening.attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(Ident {
                                span: DUMMY_SP,
                                sym: "stroke".into(),
                                optional: false,
                                ctxt: Default::default(),
                            }.into()),
                            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: color.clone().into(),
                                raw: None,
                            }))),
                        }));
                    }
                }
            }
        }

        // Continue with children
        node.visit_mut_children_with(self);
        
        // Restore the previous parent state after processing children
        self.current_parent_id = previous_parent_id;
        self.current_parent_config = previous_parent_config;
        
        // If we're exiting an SVG element, update our state
        if is_svg_element {
            self.inside_svg = false;
        }
    }
}