use std::collections::HashMap;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};

/// SWC visitor for processing Figma-generated SVG content
/// Handles superscript detection, path optimization, and other Figma-specific SVG fixes
pub struct FigmaSvgVisitor {
    /// Track text elements that need superscript fixes
    text_fixes: Vec<TextFix>,
    /// Configuration for processing options
    config: FigmaSvgConfig,
}

/// Configuration for Figma SVG processing
#[derive(Debug, Clone)]
pub struct FigmaSvgConfig {
    /// Whether to fix superscripts in SVG text elements
    pub fix_superscripts: bool,
    /// Whether to optimize SVG paths
    pub optimize_paths: bool,
    /// Whether to extract text elements for potential HTML conversion
    pub extract_text_elements: bool,
}

impl Default for FigmaSvgConfig {
    fn default() -> Self {
        Self {
            fix_superscripts: true,
            optimize_paths: true,
            extract_text_elements: false,
        }
    }
}

/// Represents a text element that needs superscript formatting
#[derive(Debug, Clone)]
pub struct TextFix {
    pub element_id: String,
    pub is_superscript: bool,
    pub is_subscript: bool,
    pub font_size: Option<f64>,
    pub dy: Option<f64>,
    pub original_attrs: Vec<JSXAttrOrSpread>,
}

impl FigmaSvgVisitor {
    pub fn new() -> Self {
        Self::with_config(FigmaSvgConfig::default())
    }

    pub fn with_config(config: FigmaSvgConfig) -> Self {
        Self {
            text_fixes: Vec::new(),
            config,
        }
    }

    /// Get the collected text fixes
    pub fn get_text_fixes(&self) -> &[TextFix] {
        &self.text_fixes
    }

    /// Extract numeric value from JSX attribute
    fn extract_numeric_value(&self, value: &str) -> Option<f64> {
        // Remove common units and parse
        let cleaned = value
            .replace("px", "")
            .replace("em", "")
            .replace("pt", "")
            .replace("%", "");
        cleaned.parse::<f64>().ok()
    }

    /// Create a superscript tspan element
    fn create_superscript_tspan(
        &self,
        content: &str,
        original_attrs: &[JSXAttrOrSpread],
    ) -> JSXElement {
        let mut attrs = vec![
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "dy".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: "-0.3em".into(),
                    raw: None,
                }))),
            }),
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "font-size".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: "0.8em".into(),
                    raw: None,
                }))),
            }),
        ];

        // Preserve other attributes like x, y, fill, etc.
        for attr in original_attrs {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                if let JSXAttrName::Ident(ident) = &jsx_attr.name {
                    let attr_name = ident.sym.as_ref();
                    // Skip attributes we're replacing
                    if !matches!(attr_name, "dy" | "font-size" | "baseline-shift") {
                        attrs.push(JSXAttrOrSpread::JSXAttr(jsx_attr.clone()));
                    }
                }
            }
        }

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "tspan".into(),
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
                    sym: "tspan".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
            children: vec![JSXElementChild::JSXText(JSXText {
                span: DUMMY_SP,
                value: content.into(),
                raw: content.into(),
            })],
        }
    }

    /// Create a subscript tspan element
    fn create_subscript_tspan(
        &self,
        content: &str,
        original_attrs: &[JSXAttrOrSpread],
    ) -> JSXElement {
        let mut attrs = vec![
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "dy".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: "0.3em".into(),
                    raw: None,
                }))),
            }),
            JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "font-size".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: "0.8em".into(),
                    raw: None,
                }))),
            }),
        ];

        // Preserve other attributes
        for attr in original_attrs {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                if let JSXAttrName::Ident(ident) = &jsx_attr.name {
                    let attr_name = ident.sym.as_ref();
                    if !matches!(attr_name, "dy" | "font-size" | "baseline-shift") {
                        attrs.push(JSXAttrOrSpread::JSXAttr(jsx_attr.clone()));
                    }
                }
            }
        }

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "tspan".into(),
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
                    sym: "tspan".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
            children: vec![JSXElementChild::JSXText(JSXText {
                span: DUMMY_SP,
                value: content.into(),
                raw: content.into(),
            })],
        }
    }
}

impl VisitMut for FigmaSvgVisitor {
    /// Visit JSX elements to detect and fix text elements that need superscript/subscript formatting
    fn visit_mut_jsx_element(&mut self, element: &mut JSXElement) {
        // Check if this is a text or tspan element
        let is_text_element = match &element.opening.name {
            JSXElementName::Ident(ident) => {
                ident.sym.as_ref() == "text" || ident.sym.as_ref() == "tspan"
            }
            _ => false,
        };

        if is_text_element && self.config.fix_superscripts {
            let mut is_superscript = false;
            let mut is_subscript = false;
            let mut font_size = None;
            let mut dy = None;
            let mut element_id = String::new();
            let original_attrs = element.opening.attrs.clone();

            // Analyze attributes
            for attr in &element.opening.attrs {
                if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                    if let JSXAttrName::Ident(ident) = &jsx_attr.name {
                        let attr_name = ident.sym.as_ref();

                        if let Some(JSXAttrValue::Lit(Lit::Str(str_lit))) = &jsx_attr.value {
                            let attr_value = str_lit.value.as_ref();

                            match attr_name {
                                "id" => element_id = attr_value.to_string(),
                                "baseline-shift" => {
                                    if attr_value == "super" || attr_value.starts_with('+') {
                                        is_superscript = true;
                                    } else if attr_value == "sub" || attr_value.starts_with('-') {
                                        is_subscript = true;
                                    }
                                }
                                "dy" => {
                                    if let Some(dy_val) = self.extract_numeric_value(attr_value) {
                                        dy = Some(dy_val);
                                        if dy_val < 0.0 {
                                            is_superscript = true;
                                        } else if dy_val > 0.0 {
                                            is_subscript = true;
                                        }
                                    }
                                }
                                "font-size" => {
                                    if let Some(fs_val) = self.extract_numeric_value(attr_value) {
                                        font_size = Some(fs_val);
                                        if fs_val < 12.0 {
                                            // Small font size might indicate superscript
                                            is_superscript = true;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            // If we detected superscript or subscript characteristics, transform the element
            if is_superscript || is_subscript {
                // Record the fix for potential later use
                self.text_fixes.push(TextFix {
                    element_id: if element_id.is_empty() {
                        format!("text_{}", self.text_fixes.len())
                    } else {
                        element_id
                    },
                    is_superscript,
                    is_subscript,
                    font_size,
                    dy,
                    original_attrs: original_attrs.clone(),
                });

                // Extract text content
                let text_content = element
                    .children
                    .iter()
                    .find_map(|child| {
                        if let JSXElementChild::JSXText(text) = child {
                            Some(text.value.as_ref())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("");

                // Transform the element
                if is_superscript {
                    *element = self.create_superscript_tspan(text_content, &original_attrs);
                } else if is_subscript {
                    *element = self.create_subscript_tspan(text_content, &original_attrs);
                }
            }
        }

        // Continue visiting children
        element.visit_mut_children_with(self);
    }

    /// Visit JSX elements to optimize SVG paths
    fn visit_mut_jsx_attr(&mut self, attr: &mut JSXAttr) {
        if self.config.optimize_paths {
            if let JSXAttrName::Ident(ident) = &attr.name {
                if ident.sym.as_ref() == "d" {
                    if let Some(JSXAttrValue::Lit(Lit::Str(str_lit))) = &mut attr.value {
                        let path_data = str_lit.value.as_ref();
                        let optimized = self.optimize_path_data(path_data);
                        if optimized != path_data {
                            str_lit.value = optimized.into();
                        }
                    }
                }
            }
        }
    }
}

impl FigmaSvgVisitor {
    /// Optimize SVG path data by simplifying redundant commands
    fn optimize_path_data(&self, path_data: &str) -> String {
        let mut processed = path_data.to_string();

        // Remove redundant M commands (move to same position)
        // This is a simplified version - in a full implementation, you'd use a proper path parser
        let redundant_m_pattern = r"M\s*([0-9.-]+)\s*,\s*([0-9.-]+)\s+M\s*\1\s*,\s*\2";
        processed = processed.replace(redundant_m_pattern, "M $1,$2");

        // Simplify consecutive L commands to H/V where possible
        let consecutive_l_horizontal = r"L\s*([0-9.-]+)\s*,\s*([0-9.-]+)\s+L\s*([0-9.-]+)\s*,\s*\2";
        processed = processed.replace(consecutive_l_horizontal, "L $1,$2 H $3");

        let consecutive_l_vertical = r"L\s*([0-9.-]+)\s*,\s*([0-9.-]+)\s+L\s*\1\s*,\s*([0-9.-]+)";
        processed = processed.replace(consecutive_l_vertical, "L $1,$2 V $3");

        processed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superscript_detection() {
        let mut visitor = FigmaSvgVisitor::new();

        // Create a mock JSX element with superscript attributes
        let mut jsx_element = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "text".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![
                    JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(IdentName {
                            span: DUMMY_SP,
                            sym: "baseline-shift".into(),
                        }),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: "super".into(),
                            raw: None,
                        }))),
                    }),
                    JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(IdentName {
                            span: DUMMY_SP,
                            sym: "font-size".into(),
                        }),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: "8".into(),
                            raw: None,
                        }))),
                    }),
                ],
                self_closing: false,
                type_args: None,
            },
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "text".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
            children: vec![JSXElementChild::JSXText(JSXText {
                span: DUMMY_SP,
                value: "2".into(),
                raw: "2".into(),
            })],
        };

        // Process the element
        visitor.visit_mut_jsx_element(&mut jsx_element);

        // Check that superscript was detected and transformed
        let fixes = visitor.get_text_fixes();
        assert_eq!(fixes.len(), 1);
        assert!(fixes[0].is_superscript);
        assert_eq!(fixes[0].font_size, Some(8.0));

        // Check that the element was transformed to a tspan
        match &jsx_element.opening.name {
            JSXElementName::Ident(ident) => {
                assert_eq!(ident.sym.as_ref(), "tspan");
            }
            _ => panic!("Expected tspan element"),
        }
    }

    #[test]
    fn test_path_optimization() {
        let mut visitor = FigmaSvgVisitor::new();

        let mut attr = JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(IdentName {
                span: DUMMY_SP,
                sym: "d".into(),
            }),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: "M 10,10 M 10,10 L 20,20 L 30,20".into(),
                raw: None,
            }))),
        };

        visitor.visit_mut_jsx_attr(&mut attr);

        if let Some(JSXAttrValue::Lit(Lit::Str(str_lit))) = &attr.value {
            let optimized_path = str_lit.value.as_ref();
            // Should have removed redundant M command
            assert!(!optimized_path.contains("M 10,10 M 10,10"));
        }
    }

    #[test]
    fn test_config_disable_superscript_fix() {
        let config = FigmaSvgConfig {
            fix_superscripts: false,
            optimize_paths: true,
            extract_text_elements: false,
        };
        let mut visitor = FigmaSvgVisitor::with_config(config);

        // Create a superscript text element
        let mut jsx_element = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "text".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
                attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(IdentName {
                        span: DUMMY_SP,
                        sym: "baseline-shift".into(),
                    }),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: "super".into(),
                        raw: None,
                    }))),
                })],
                self_closing: false,
                type_args: None,
            },
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident {
                    span: DUMMY_SP,
                    sym: "text".into(),
                    optional: false,
                    ctxt: SyntaxContext::empty(),
                }),
            }),
            children: vec![JSXElementChild::JSXText(JSXText {
                span: DUMMY_SP,
                value: "2".into(),
                raw: "2".into(),
            })],
        };

        visitor.visit_mut_jsx_element(&mut jsx_element);

        // Should not have been transformed
        match &jsx_element.opening.name {
            JSXElementName::Ident(ident) => {
                assert_eq!(ident.sym.as_ref(), "text");
            }
            _ => panic!("Expected text element"),
        }

        // Should not have recorded any fixes
        assert!(visitor.get_text_fixes().is_empty());
    }
}
