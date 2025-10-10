use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GlowFilterProps {
    pub id: String,
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intensity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(
        type = "'linear' | 'easeIn' | 'easeOut' | 'easeInOut' | 'circIn' | 'circOut' | 'backOut'"
    )]
    pub easing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "glowLayers")]
    pub glow_layers: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "maxIntensity")]
    pub max_intensity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pulsing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "onHover")]
    pub on_hover: Option<String>, // Function name as string for serialization
}

impl Default for GlowFilterProps {
    fn default() -> Self {
        Self {
            id: String::new(),
            color: "#ffffff".to_string(),
            intensity: Some(1.0),
            animated: Some(false),
            duration: Some(1.0),
            delay: Some(0.0),
            easing: Some("easeInOut".to_string()),
            glow_layers: Some(3),
            max_intensity: Some(2.0),
            pulsing: Some(false),
            interactive: Some(false),
            on_hover: None,
        }
    }
}

/// A visitor that injects GlowFilter components into SVG elements
#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub struct FilterVisitor {
    pub glow_filters: HashMap<String, GlowFilterProps>,
    /// Track whether we're inside an SVG element
    inside_svg: bool,
    /// Store filters to be added to current SVG
    pending_filters: Vec<GlowFilterProps>,
    /// Track filter ids already present in current SVG <defs>
    #[serde(skip)]
    #[ts(skip)]
    existing_defs_filter_ids: HashSet<String>,
    /// Track filter ids scheduled to be added for current SVG to avoid duplicates
    #[serde(skip)]
    #[ts(skip)]
    scheduled_filter_ids: HashSet<String>,
}

impl FilterVisitor {
    pub fn new(glow_filters: HashMap<String, GlowFilterProps>) -> Self {
        Self {
            glow_filters,
            inside_svg: false,
            pending_filters: Vec::new(),
            existing_defs_filter_ids: HashSet::new(),
            scheduled_filter_ids: HashSet::new(),
        }
    }

    /// Register a glow filter for a specific element ID
    pub fn register_glow_filter(&mut self, element_id: String, filter: GlowFilterProps) {
        self.glow_filters.insert(element_id, filter);
    }

    /// Create a GlowFilter JSX component
    fn create_glow_filter_component(&self, props: &GlowFilterProps) -> JSXElement {
        let mut attrs = vec![
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "id".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: props.id.clone().into(),
                    raw: None,
                }))),
            }),
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "color".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: props.color.clone().into(),
                    raw: None,
                }))),
            }),
        ];

        // Add optional props
        if let Some(intensity) = props.intensity {
            attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "intensity".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num(Number {
                        span: DUMMY_SP,
                        value: intensity,
                        raw: None,
                    })))),
                })),
            }));
        }

        if let Some(animated) = props.animated {
            attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "animated".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Bool(Bool {
                        span: DUMMY_SP,
                        value: animated,
                    })))),
                })),
            }));
        }

        if let Some(duration) = props.duration {
            attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "duration".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num(Number {
                        span: DUMMY_SP,
                        value: duration,
                        raw: None,
                    })))),
                })),
            }));
        }

        if let Some(pulsing) = props.pulsing {
            attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "pulsing".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Bool(Bool {
                        span: DUMMY_SP,
                        value: pulsing,
                    })))),
                })),
            }));
        }

        if let Some(interactive) = props.interactive {
            attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "interactive".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Bool(Bool {
                        span: DUMMY_SP,
                        value: interactive,
                    })))),
                })),
            }));
        }

        if let Some(glow_layers) = props.glow_layers {
            attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(
                    Ident {
                        span: DUMMY_SP,
                        sym: "glowLayers".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }
                    .into(),
                ),
                value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                    span: DUMMY_SP,
                    expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Num(Number {
                        span: DUMMY_SP,
                        value: glow_layers as f64,
                        raw: None,
                    })))),
                })),
            }));
        }

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "GlowFilter".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs,
                self_closing: true,
                type_args: None,
            },
            children: vec![],
            closing: None,
        }
    }

    /// Create or find defs element in SVG and add GlowFilter components
    fn create_defs_with_filters(&self, filters: Vec<JSXElement>) -> JSXElement {
        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "defs".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![],
                self_closing: false,
                type_args: None,
            },
            children: filters
                .into_iter()
                .map(|filter| JSXElementChild::JSXElement(Box::new(filter)))
                .collect(),
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "defs".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        }
    }

    /// Extract a string literal attribute value from a JSX opening element
    fn get_jsx_attr_str(opening: &JSXOpeningElement, attr_name: &str) -> Option<String> {
        for attr in &opening.attrs {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                if let JSXAttrName::Ident(name) = &jsx_attr.name {
                    if name.sym.as_ref() == attr_name {
                        if let Some(JSXAttrValue::Lit(Lit::Str(str_lit))) = &jsx_attr.value {
                            return Some(str_lit.value.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// Collect filter ids already defined in an existing <defs> inside given <svg>
    fn collect_existing_defs_filter_ids(svg_element: &JSXElement) -> HashSet<String> {
        let mut ids = HashSet::new();
        for child in &svg_element.children {
            if let JSXElementChild::JSXElement(defs_el) = child {
                if let JSXElementName::Ident(ident) = &defs_el.opening.name {
                    if ident.sym.as_ref() == "defs" {
                        for defs_child in &defs_el.children {
                            if let JSXElementChild::JSXElement(filter_el) = defs_child {
                                if let JSXElementName::Ident(child_ident) = &filter_el.opening.name
                                {
                                    let name = child_ident.sym.as_ref();
                                    if name == "GlowFilter" || name == "filter" {
                                        if let Some(id) =
                                            Self::get_jsx_attr_str(&filter_el.opening, "id")
                                        {
                                            ids.insert(id);
                                        }
                                    }
                                }
                            }
                        }
                        // Do not traverse deeper; only first-level children of defs are considered
                    }
                }
            }
        }
        ids
    }

    /// Find mutable reference to first existing <defs> inside given <svg>
    fn find_defs_mut(svg_element: &mut JSXElement) -> Option<&mut JSXElement> {
        for child in &mut svg_element.children {
            if let JSXElementChild::JSXElement(defs_el) = child {
                if let JSXElementName::Ident(ident) = &defs_el.opening.name {
                    if ident.sym.as_ref() == "defs" {
                        return Some(defs_el.as_mut());
                    }
                }
            }
        }
        None
    }
}

impl VisitMut for FilterVisitor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        // Debug: Print filter configuration
        // eprintln!(
        //     "FilterVisitor: Processing module with {} glow filters configured",
        //     self.glow_filters.len()
        // );
        // for (key, filter) in &self.glow_filters {
        //     eprintln!(
        //         "FilterVisitor: - Filter '{}' -> GlowFilter(id: '{}', color: '{}')",
        //         key, filter.id, filter.color
        //     );
        // }

        // Add the import for GlowFilter component if we have any glow filters
        if !self.glow_filters.is_empty() {
            let glow_filter_import = ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                span: DUMMY_SP,
                specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                    span: DUMMY_SP,
                    local: Ident {
                        span: DUMMY_SP,
                        sym: "GlowFilter".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    },
                    imported: None,
                    is_type_only: false,
                })],
                src: Box::new(Str {
                    span: DUMMY_SP,
                    value: "@/components/filters/glow-filter".into(), // Adjust path as needed
                    raw: None,
                }),
                type_only: false,
                with: None,
                phase: Default::default(),
            }));

            // Insert the import at the beginning of the module
            module.body.insert(0, glow_filter_import);
        }

        // Continue with the rest of the module
        module.visit_mut_children_with(self);
    }

    fn visit_mut_jsx_element(&mut self, element: &mut JSXElement) {
        // Check if this is an SVG element
        let is_svg = match &element.opening.name {
            JSXElementName::Ident(ident) => ident.sym.as_ref() == "svg",
            _ => false,
        };

        let was_inside_svg = self.inside_svg;
        if is_svg {
            // eprintln!("FilterVisitor: Found SVG element, entering SVG context");
            self.inside_svg = true;
            self.pending_filters.clear();
            // Reset per-SVG dedup tracking and collect existing defs filter ids
            self.scheduled_filter_ids.clear();
            self.existing_defs_filter_ids = Self::collect_existing_defs_filter_ids(element);
            if !self.existing_defs_filter_ids.is_empty() {
                // eprintln!(
                //     "FilterVisitor: Existing defs already contain filter ids: {:?}",
                //     self.existing_defs_filter_ids
                // );
            }
        }

        // Check for elements with IDs that need glow filters
        if let Some(id_attr) = element.opening.attrs.iter().find_map(|attr| {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                if let JSXAttrName::Ident(name) = &jsx_attr.name {
                    if name.sym.as_ref() == "id" {
                        if let Some(JSXAttrValue::Lit(Lit::Str(str_lit))) = &jsx_attr.value {
                            return Some(str_lit.value.to_string());
                        }
                    }
                }
            }
            None
        }) {
            // Debug: Print found element ID and available filter keys
            // eprintln!("FilterVisitor: Found element with ID: '{}'", id_attr);
            // eprintln!(
            //     "FilterVisitor: Available filter keys: {:?}",
            //     self.glow_filters.keys().collect::<Vec<_>>()
            // );

            if let Some(glow_filter) = self.glow_filters.get(&id_attr) {
                // eprintln!(
                //     "FilterVisitor: Applying glow filter '{}' to element '{}'",
                //     glow_filter.id, id_attr
                // );
                // Add filter attribute to element
                element
                    .opening
                    .attrs
                    .push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(
                            Ident {
                                span: DUMMY_SP,
                                sym: "filter".into(),
                                optional: false,
                                ctxt: SyntaxContext::empty(),
                            }
                            .into(),
                        ),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: format!("url(#{})", glow_filter.id).into(),
                            raw: None,
                        }))),
                    }));

                // Store filter to be added to SVG defs
                if self.inside_svg {
                    let filter_id = glow_filter.id.clone();
                    let already_present = self.existing_defs_filter_ids.contains(&filter_id)
                        || self.scheduled_filter_ids.contains(&filter_id);
                    if !already_present {
                        self.pending_filters.push(glow_filter.clone());
                        self.scheduled_filter_ids.insert(filter_id);
                    } else {
                        // eprintln!(
                        //     "FilterVisitor: Skipping duplicate GlowFilter with id '{}' (already present/scheduled)",
                        //     filter_id
                        // );
                    }
                }
            }
        }

        // Continue visiting children
        element.visit_mut_children_with(self);

        // If this was an SVG element and we have pending filters, add defs with GlowFilter components
        if is_svg && !self.pending_filters.is_empty() {
            // eprintln!(
            //     "FilterVisitor: Adding {} filter(s) to SVG defs",
            //     self.pending_filters.len()
            // );
            // for filter in &self.pending_filters {
            //     eprintln!("FilterVisitor: - Adding GlowFilter with id '{}'", filter.id);
            // }

            let filter_components: Vec<JSXElement> = self
                .pending_filters
                .iter()
                .map(|filter| self.create_glow_filter_component(filter))
                .collect();

            // If a <defs> exists, append to it; otherwise create a new <defs>
            if let Some(defs_mut) = Self::find_defs_mut(element) {
                for comp in filter_components {
                    defs_mut
                        .children
                        .push(JSXElementChild::JSXElement(Box::new(comp)));
                }
            } else {
                let defs_element = self.create_defs_with_filters(filter_components);
                // Insert defs at the beginning of SVG children
                element
                    .children
                    .insert(0, JSXElementChild::JSXElement(Box::new(defs_element)));
            }

            self.pending_filters.clear();
            self.scheduled_filter_ids.clear();
        }

        if is_svg {
            self.inside_svg = was_inside_svg;
            // Clear per-SVG collections when leaving current svg
            self.existing_defs_filter_ids.clear();
            self.scheduled_filter_ids.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_common::DUMMY_SP;
    
    use swc_ecma_visit::VisitMutWith;

    fn create_test_svg_with_elements() -> Module {
        // Create a simple SVG with elements that have IDs
        let svg_element = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "svg".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![
                    JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(
                            Ident {
                                span: DUMMY_SP,
                                sym: "width".into(),
                                optional: false,
                                ctxt: SyntaxContext::empty(),
                            }
                            .into(),
                        ),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: "200".into(),
                            raw: None,
                        }))),
                    }),
                    JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(
                            Ident {
                                span: DUMMY_SP,
                                sym: "height".into(),
                                optional: false,
                                ctxt: SyntaxContext::empty(),
                            }
                            .into(),
                        ),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: "200".into(),
                            raw: None,
                        }))),
                    }),
                ],
                self_closing: false,
                type_args: None,
            },
            children: vec![
                // Circle with ID that should get glow filter
                JSXElementChild::JSXElement(Box::new(JSXElement {
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "circle".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                        attrs: vec![
                            JSXAttrOrSpread::JSXAttr(JSXAttr {
                                span: DUMMY_SP,
                                name: JSXAttrName::Ident(
                                    Ident {
                                        span: DUMMY_SP,
                                        sym: "id".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into(),
                                ),
                                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "test-circle".into(),
                                    raw: None,
                                }))),
                            }),
                            JSXAttrOrSpread::JSXAttr(JSXAttr {
                                span: DUMMY_SP,
                                name: JSXAttrName::Ident(
                                    Ident {
                                        span: DUMMY_SP,
                                        sym: "cx".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into(),
                                ),
                                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "100".into(),
                                    raw: None,
                                }))),
                            }),
                            JSXAttrOrSpread::JSXAttr(JSXAttr {
                                span: DUMMY_SP,
                                name: JSXAttrName::Ident(
                                    Ident {
                                        span: DUMMY_SP,
                                        sym: "cy".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into(),
                                ),
                                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "100".into(),
                                    raw: None,
                                }))),
                            }),
                            JSXAttrOrSpread::JSXAttr(JSXAttr {
                                span: DUMMY_SP,
                                name: JSXAttrName::Ident(
                                    Ident {
                                        span: DUMMY_SP,
                                        sym: "r".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into(),
                                ),
                                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "50".into(),
                                    raw: None,
                                }))),
                            }),
                            JSXAttrOrSpread::JSXAttr(JSXAttr {
                                span: DUMMY_SP,
                                name: JSXAttrName::Ident(
                                    Ident {
                                        span: DUMMY_SP,
                                        sym: "fill".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into(),
                                ),
                                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "blue".into(),
                                    raw: None,
                                }))),
                            }),
                        ],
                        self_closing: true,
                        type_args: None,
                    },
                    children: vec![],
                    closing: None,
                })),
                // Rectangle without ID (should not get filter)
                JSXElementChild::JSXElement(Box::new(JSXElement {
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "rect".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                        attrs: vec![
                            JSXAttrOrSpread::JSXAttr(JSXAttr {
                                span: DUMMY_SP,
                                name: JSXAttrName::Ident(
                                    Ident {
                                        span: DUMMY_SP,
                                        sym: "x".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into(),
                                ),
                                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "25".into(),
                                    raw: None,
                                }))),
                            }),
                            JSXAttrOrSpread::JSXAttr(JSXAttr {
                                span: DUMMY_SP,
                                name: JSXAttrName::Ident(
                                    Ident {
                                        span: DUMMY_SP,
                                        sym: "y".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into(),
                                ),
                                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "25".into(),
                                    raw: None,
                                }))),
                            }),
                            JSXAttrOrSpread::JSXAttr(JSXAttr {
                                span: DUMMY_SP,
                                name: JSXAttrName::Ident(
                                    Ident {
                                        span: DUMMY_SP,
                                        sym: "width".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into(),
                                ),
                                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "150".into(),
                                    raw: None,
                                }))),
                            }),
                            JSXAttrOrSpread::JSXAttr(JSXAttr {
                                span: DUMMY_SP,
                                name: JSXAttrName::Ident(
                                    Ident {
                                        span: DUMMY_SP,
                                        sym: "height".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into(),
                                ),
                                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "150".into(),
                                    raw: None,
                                }))),
                            }),
                            JSXAttrOrSpread::JSXAttr(JSXAttr {
                                span: DUMMY_SP,
                                name: JSXAttrName::Ident(
                                    Ident {
                                        span: DUMMY_SP,
                                        sym: "fill".into(),
                                        optional: false,
                                        ctxt: SyntaxContext::empty(),
                                    }
                                    .into(),
                                ),
                                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                    span: DUMMY_SP,
                                    value: "red".into(),
                                    raw: None,
                                }))),
                            }),
                        ],
                        self_closing: true,
                        type_args: None,
                    },
                    children: vec![],
                    closing: None,
                })),
            ],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "svg".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        };

        // Create a simple React component that returns the SVG
        let jsx_expr = JSXExpr::Expr(Box::new(Expr::JSXElement(Box::new(svg_element))));

        let return_stmt = Stmt::Return(ReturnStmt {
            span: DUMMY_SP,
            arg: Some(Box::new(Expr::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "svg".into(),
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
                            sym: "circle".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                        attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(
                                Ident {
                                    span: DUMMY_SP,
                                    sym: "id".into(),
                                    optional: false,
                                    ctxt: SyntaxContext::empty(),
                                }
                                .into(),
                            ),
                            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: "test-circle".into(),
                                raw: None,
                            }))),
                        })],
                        self_closing: true,
                        type_args: None,
                    },
                    children: vec![],
                    closing: None,
                }))],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident {
                        span: DUMMY_SP,
                        sym: "svg".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                }),
            })))),
        });

        Module {
            span: DUMMY_SP,
            body: vec![ModuleItem::Stmt(return_stmt)],
            shebang: None,
        }
    }

    #[test]
    fn test_filter_visitor_adds_import() {
        let mut module = create_test_svg_with_elements();

        // Create a glow filter configuration
        let mut glow_filters = HashMap::new();
        glow_filters.insert(
            "test-circle".to_string(),
            GlowFilterProps {
                id: "test-circle-glow".to_string(),
                color: "#00ff00".to_string(),
                intensity: Some(2.0),
                animated: Some(true),
                duration: Some(1.5),
                pulsing: Some(true),
                glow_layers: Some(4),
                ..Default::default()
            },
        );

        let mut visitor = FilterVisitor::new(glow_filters);
        module.visit_mut_with(&mut visitor);

        // Check that an import was added
        assert!(!module.body.is_empty());

        if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = &module.body[0] {
            assert_eq!(
                import_decl.src.value.as_ref(),
                "@/components/filters/glow-filter"
            );
            assert_eq!(import_decl.specifiers.len(), 1);

            if let ImportSpecifier::Named(named_spec) = &import_decl.specifiers[0] {
                assert_eq!(named_spec.local.sym.as_ref(), "GlowFilter");
            } else {
                panic!("Expected named import specifier");
            }
        } else {
            panic!("Expected import declaration as first module item");
        }
    }

    #[test]
    fn test_filter_visitor_no_import_when_no_filters() {
        let mut module = create_test_svg_with_elements();
        let original_body_len = module.body.len();

        // Create visitor with no glow filters
        let mut visitor = FilterVisitor::new(HashMap::new());
        module.visit_mut_with(&mut visitor);

        // Check that no import was added
        assert_eq!(module.body.len(), original_body_len);
    }

    #[test]
    fn test_glow_filter_props_serialization() {
        let props = GlowFilterProps {
            id: "test-filter".to_string(),
            color: "#ff0000".to_string(),
            intensity: Some(1.5),
            animated: Some(true),
            duration: Some(2.0),
            delay: Some(0.5),
            easing: Some("easeInOut".to_string()),
            glow_layers: Some(3),
            max_intensity: Some(2.5),
            pulsing: Some(false),
            interactive: Some(true),
            on_hover: Some("handleHover".to_string()),
        };

        // Test serialization
        let serialized = serde_json::to_string(&props).expect("Failed to serialize");
        assert!(serialized.contains("\"id\":\"test-filter\""));
        assert!(serialized.contains("\"color\":\"#ff0000\""));
        assert!(serialized.contains("\"intensity\":1.5"));
        assert!(serialized.contains("\"glowLayers\":3")); // Check camelCase conversion
        assert!(serialized.contains("\"maxIntensity\":2.5")); // Check camelCase conversion
        assert!(serialized.contains("\"onHover\":\"handleHover\"")); // Check camelCase conversion

        // Test deserialization
        let deserialized: GlowFilterProps =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(deserialized.id, props.id);
        assert_eq!(deserialized.color, props.color);
        assert_eq!(deserialized.intensity, props.intensity);
        assert_eq!(deserialized.glow_layers, props.glow_layers);
    }

    #[test]
    fn test_glow_filter_props_default() {
        let default_props = GlowFilterProps::default();

        assert_eq!(default_props.id, "");
        assert_eq!(default_props.color, "#ffffff");
        assert_eq!(default_props.intensity, Some(1.0));
        assert_eq!(default_props.animated, Some(false));
        assert_eq!(default_props.duration, Some(1.0));
        assert_eq!(default_props.delay, Some(0.0));
        assert_eq!(default_props.easing, Some("easeInOut".to_string()));
        assert_eq!(default_props.glow_layers, Some(3));
        assert_eq!(default_props.max_intensity, Some(2.0));
        assert_eq!(default_props.pulsing, Some(false));
        assert_eq!(default_props.interactive, Some(false));
        assert_eq!(default_props.on_hover, None);
    }

    #[test]
    fn test_filter_visitor_register_glow_filter() {
        let mut visitor = FilterVisitor::new(HashMap::new());

        let filter_props = GlowFilterProps {
            id: "dynamic-filter".to_string(),
            color: "#00ffff".to_string(),
            intensity: Some(3.0),
            ..Default::default()
        };

        visitor.register_glow_filter("dynamic-element".to_string(), filter_props.clone());

        assert!(visitor.glow_filters.contains_key("dynamic-element"));
        assert_eq!(visitor.glow_filters["dynamic-element"].id, "dynamic-filter");
        assert_eq!(visitor.glow_filters["dynamic-element"].color, "#00ffff");
        assert_eq!(visitor.glow_filters["dynamic-element"].intensity, Some(3.0));
    }

    #[test]
    fn test_create_glow_filter_component() {
        let visitor = FilterVisitor::new(HashMap::new());

        let props = GlowFilterProps {
            id: "test-glow".to_string(),
            color: "#ff00ff".to_string(),
            intensity: Some(2.5),
            animated: Some(true),
            duration: Some(3.0),
            pulsing: Some(false),
            interactive: Some(true),
            glow_layers: Some(5),
            ..Default::default()
        };

        let jsx_element = visitor.create_glow_filter_component(&props);

        // Check that it's a self-closing GlowFilter element
        if let JSXElementName::Ident(ident) = &jsx_element.opening.name {
            assert_eq!(ident.sym.as_ref(), "GlowFilter");
        } else {
            panic!("Expected GlowFilter element name");
        }

        assert!(jsx_element.opening.self_closing);
        assert!(jsx_element.children.is_empty());
        assert!(jsx_element.closing.is_none());

        // Check that required props are present
        let has_id = jsx_element.opening.attrs.iter().any(|attr| {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                if let JSXAttrName::Ident(name) = &jsx_attr.name {
                    return name.sym.as_ref() == "id";
                }
            }
            false
        });
        assert!(has_id, "GlowFilter should have id attribute");

        let has_color = jsx_element.opening.attrs.iter().any(|attr| {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                if let JSXAttrName::Ident(name) = &jsx_attr.name {
                    return name.sym.as_ref() == "color";
                }
            }
            false
        });
        assert!(has_color, "GlowFilter should have color attribute");

        // Check that optional props are present when provided
        let has_intensity = jsx_element.opening.attrs.iter().any(|attr| {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                if let JSXAttrName::Ident(name) = &jsx_attr.name {
                    return name.sym.as_ref() == "intensity";
                }
            }
            false
        });
        assert!(
            has_intensity,
            "GlowFilter should have intensity attribute when provided"
        );
    }

    #[test]
    fn test_dedup_duplicate_filter_ids_in_same_svg() {
        // SVG with two elements that map to filters sharing same underlying filter id
        let svg = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "svg".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![],
                self_closing: false,
                type_args: None,
            },
            children: vec![
                JSXElementChild::JSXElement(Box::new(JSXElement {
                    // first element
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "circle".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                        attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(
                                Ident {
                                    span: DUMMY_SP,
                                    sym: "id".into(),
                                    optional: false,
                                    ctxt: SyntaxContext::empty(),
                                }
                                .into(),
                            ),
                            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: "a".into(),
                                raw: None,
                            }))),
                        })],
                        self_closing: true,
                        type_args: None,
                    },
                    children: vec![],
                    closing: None,
                })),
                JSXElementChild::JSXElement(Box::new(JSXElement {
                    // second element
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "rect".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                        attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(
                                Ident {
                                    span: DUMMY_SP,
                                    sym: "id".into(),
                                    optional: false,
                                    ctxt: SyntaxContext::empty(),
                                }
                                .into(),
                            ),
                            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: "b".into(),
                                raw: None,
                            }))),
                        })],
                        self_closing: true,
                        type_args: None,
                    },
                    children: vec![],
                    closing: None,
                })),
            ],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "svg".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        };

        let mut module = Module {
            span: DUMMY_SP,
            body: vec![ModuleItem::Stmt(Stmt::Return(ReturnStmt {
                span: DUMMY_SP,
                arg: Some(Box::new(Expr::JSXElement(Box::new(svg)))),
            }))],
            shebang: None,
        };

        let mut glow_filters = HashMap::new();
        glow_filters.insert(
            "a".to_string(),
            GlowFilterProps {
                id: "glow-filter".to_string(),
                color: "#fff".to_string(),
                ..Default::default()
            },
        );
        glow_filters.insert(
            "b".to_string(),
            GlowFilterProps {
                id: "glow-filter".to_string(),
                color: "#fff".to_string(),
                ..Default::default()
            },
        );

        let mut visitor = FilterVisitor::new(glow_filters);
        module.visit_mut_with(&mut visitor);

        // Count GlowFilter components inside defs; should be exactly 1
        let mut count = 0usize;
        if let ModuleItem::Stmt(Stmt::Return(ReturnStmt {
            arg: Some(expr), ..
        })) = &module.body[0]
        {
            if let Expr::JSXElement(svg) = expr.as_ref() {
                for child in &svg.children {
                    if let JSXElementChild::JSXElement(defs) = child {
                        if let JSXElementName::Ident(name) = &defs.opening.name {
                            if name.sym.as_ref() == "defs" {
                                for defs_child in &defs.children {
                                    if let JSXElementChild::JSXElement(el) = defs_child {
                                        if let JSXElementName::Ident(n) = &el.opening.name {
                                            if n.sym.as_ref() == "GlowFilter" {
                                                count += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(
            count, 1,
            "Expected exactly one GlowFilter in defs after dedup"
        );
    }

    #[test]
    fn test_append_to_existing_defs() {
        // SVG with an existing defs containing one GlowFilter
        let existing_defs = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "defs".into(),
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
                        sym: "GlowFilter".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(
                            Ident {
                                span: DUMMY_SP,
                                sym: "id".into(),
                                optional: false,
                                ctxt: SyntaxContext::empty(),
                            }
                            .into(),
                        ),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: "existing".into(),
                            raw: None,
                        }))),
                    })],
                    self_closing: true,
                    type_args: None,
                },
                children: vec![],
                closing: None,
            }))],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "defs".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        };

        let svg = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "svg".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![],
                self_closing: false,
                type_args: None,
            },
            children: vec![
                JSXElementChild::JSXElement(Box::new(existing_defs)),
                JSXElementChild::JSXElement(Box::new(JSXElement {
                    // element needing new filter
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "circle".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                        attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(
                                Ident {
                                    span: DUMMY_SP,
                                    sym: "id".into(),
                                    optional: false,
                                    ctxt: SyntaxContext::empty(),
                                }
                                .into(),
                            ),
                            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: "need".into(),
                                raw: None,
                            }))),
                        })],
                        self_closing: true,
                        type_args: None,
                    },
                    children: vec![],
                    closing: None,
                })),
            ],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "svg".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        };

        let mut module = Module {
            span: DUMMY_SP,
            body: vec![ModuleItem::Stmt(Stmt::Return(ReturnStmt {
                span: DUMMY_SP,
                arg: Some(Box::new(Expr::JSXElement(Box::new(svg)))),
            }))],
            shebang: None,
        };

        let mut glow_filters = HashMap::new();
        glow_filters.insert(
            "need".to_string(),
            GlowFilterProps {
                id: "new".to_string(),
                color: "#fff".to_string(),
                ..Default::default()
            },
        );

        let mut visitor = FilterVisitor::new(glow_filters);
        module.visit_mut_with(&mut visitor);

        // Ensure there is only one defs and it contains both existing and new GlowFilter ids
        let mut defs_count = 0usize;
        let mut ids: Vec<String> = Vec::new();
        if let ModuleItem::Stmt(Stmt::Return(ReturnStmt {
            arg: Some(expr), ..
        })) = &module.body[0]
        {
            if let Expr::JSXElement(svg) = expr.as_ref() {
                for child in &svg.children {
                    if let JSXElementChild::JSXElement(defs) = child {
                        if let JSXElementName::Ident(name) = &defs.opening.name {
                            if name.sym.as_ref() == "defs" {
                                defs_count += 1;
                                for defs_child in &defs.children {
                                    if let JSXElementChild::JSXElement(el) = defs_child {
                                        if let Some(id) =
                                            FilterVisitor::get_jsx_attr_str(&el.opening, "id")
                                        {
                                            ids.push(id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        ids.sort();
        assert_eq!(defs_count, 1, "Expected only one defs element");
        assert_eq!(ids, vec!["existing".to_string(), "new".to_string()]);
    }

    #[test]
    fn test_respects_existing_filter_in_defs() {
        // Pre-existing defs has the same GlowFilter id; no new one should be added
        let existing_defs = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "defs".into(),
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
                        sym: "GlowFilter".into(),
                        optional: false,
                        ctxt: SyntaxContext::empty(),
                    }),
                    attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(
                            Ident {
                                span: DUMMY_SP,
                                sym: "id".into(),
                                optional: false,
                                ctxt: SyntaxContext::empty(),
                            }
                            .into(),
                        ),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: "glow-filter".into(),
                            raw: None,
                        }))),
                    })],
                    self_closing: true,
                    type_args: None,
                },
                children: vec![],
                closing: None,
            }))],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "defs".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        };

        let svg = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "svg".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![],
                self_closing: false,
                type_args: None,
            },
            children: vec![
                JSXElementChild::JSXElement(Box::new(existing_defs)),
                JSXElementChild::JSXElement(Box::new(JSXElement {
                    // element needing the same filter id
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: "circle".into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                        attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(
                                Ident {
                                    span: DUMMY_SP,
                                    sym: "id".into(),
                                    optional: false,
                                    ctxt: SyntaxContext::empty(),
                                }
                                .into(),
                            ),
                            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: "need".into(),
                                raw: None,
                            }))),
                        })],
                        self_closing: true,
                        type_args: None,
                    },
                    children: vec![],
                    closing: None,
                })),
            ],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "svg".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
        };

        let mut module = Module {
            span: DUMMY_SP,
            body: vec![ModuleItem::Stmt(Stmt::Return(ReturnStmt {
                span: DUMMY_SP,
                arg: Some(Box::new(Expr::JSXElement(Box::new(svg)))),
            }))],
            shebang: None,
        };

        let mut glow_filters = HashMap::new();
        glow_filters.insert(
            "need".to_string(),
            GlowFilterProps {
                id: "glow-filter".to_string(),
                color: "#fff".to_string(),
                ..Default::default()
            },
        );

        let mut visitor = FilterVisitor::new(glow_filters);
        module.visit_mut_with(&mut visitor);

        // There should still be only one GlowFilter with id "glow-filter" in defs
        let mut count = 0usize;
        if let ModuleItem::Stmt(Stmt::Return(ReturnStmt {
            arg: Some(expr), ..
        })) = &module.body[0]
        {
            if let Expr::JSXElement(svg) = expr.as_ref() {
                for child in &svg.children {
                    if let JSXElementChild::JSXElement(defs) = child {
                        if let JSXElementName::Ident(name) = &defs.opening.name {
                            if name.sym.as_ref() == "defs" {
                                for defs_child in &defs.children {
                                    if let JSXElementChild::JSXElement(el) = defs_child {
                                        if let Some(id) =
                                            FilterVisitor::get_jsx_attr_str(&el.opening, "id")
                                        {
                                            if id == "glow-filter" {
                                                count += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(
            count, 1,
            "Expected exactly one GlowFilter with id 'glow-filter' in defs"
        );
    }
}
