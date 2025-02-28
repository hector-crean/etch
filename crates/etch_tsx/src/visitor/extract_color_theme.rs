use std::collections::HashSet;
use swc_common::{FileName, SourceMap, Span};
use swc_ecma_ast::*;
use swc_ecma_visit::{Visit, VisitMut, VisitWith};
use regex::Regex;
use swc_ecma_visit::VisitMutWith;

/// A visitor that updates Tailwind-style inline color definitions in JSX className attributes
pub struct ColorThemeVisitor {
    colors: HashSet<String>,
    pattern: Regex,
    update_fn: Box<dyn Fn(&str) -> String>,
}

impl ColorThemeVisitor {
    pub fn new(pattern: &str, update_fn: impl Fn(&str) -> String + 'static) -> Self {
        Self {
            colors: HashSet::new(),
            pattern: Regex::new(pattern).unwrap(),
            update_fn: Box::new(update_fn),
        }
    }

    pub fn colors(&self) -> &HashSet<String> {
        &self.colors
    }

    fn transform_class_string(&mut self, input: &str) -> String {
        let mut result = input.to_string();
        
        for cap in self.pattern.captures_iter(input) {
            if let Some(color_match) = cap.get(1) {
                let original = color_match.as_str();
                self.colors.insert(original.to_lowercase());
                
                let updated = (self.update_fn)(original);
                
                // Replace the full match with updated version
                if let Some(full_match) = cap.get(0) {
                    let full = full_match.as_str();
                    let replacement = full.replace(original, &updated);
                    result = result.replace(full, &replacement);
                }
            }
        }
        
        result
    }
}

impl VisitMut for ColorThemeVisitor {
    
    
    fn visit_mut_jsx_element(&mut self, node: &mut JSXElement) {
        // Process attributes
        for attr in &mut node.opening.attrs {
            if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                if let JSXAttrName::Ident(ident) = &attr.name {
                    if ident.sym.as_ref() == "className" {
                        if let Some(JSXAttrValue::Lit(Lit::Str(str_lit))) = &mut attr.value {
                            let updated = self.transform_class_string(&str_lit.value);
                            str_lit.value = updated.into();
                            str_lit.raw = None;
                        }
                    }
                }
            }
        }

        // Visit children
        node.visit_mut_children_with(self);
    }
}
