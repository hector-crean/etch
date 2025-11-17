use std::collections::HashMap;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};

/// Postprocessor for SVG content from Figma API exports using SWC
pub struct SvgPostprocessor {
    /// Track text elements that need superscript fixes
    text_fixes: Vec<TextFix>,
}

/// Represents a text element that needs superscript formatting
#[derive(Debug, Clone)]
struct TextFix {
    element_id: String,
    is_superscript: bool,
    is_subscript: bool,
    font_size: Option<f64>,
    dy: Option<f64>,
}

/// SWC visitor for processing SVG JSX elements
pub struct SvgTextVisitor {
    /// Track text elements that need superscript fixes
    text_fixes: Vec<TextFix>,
}

impl SvgTextVisitor {
    pub fn new() -> Self {
        Self {
            text_fixes: Vec::new(),
        }
    }

    /// Get the collected text fixes
    pub fn get_text_fixes(&self) -> &[TextFix] {
        &self.text_fixes
    }

    /// Check if a JSX attribute indicates superscript
    fn is_superscript_attr(&self, name: &str, value: &str) -> bool {
        match name {
            "baseline-shift" => value == "super" || value.starts_with('+'),
            "dy" => {
                if let Ok(dy_val) = value.parse::<f64>() {
                    dy_val < 0.0
                } else {
                    false
                }
            }
            "font-size" => {
                if let Ok(font_size) = value.parse::<f64>() {
                    font_size < 12.0 // Small font size might indicate superscript
                } else {
                    false
                }
            }
            _ => false,
        }
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
}

impl VisitMut for SvgTextVisitor {
    /// Visit JSX elements to detect text elements that need superscript fixes
    fn visit_mut_jsx_element(&mut self, element: &mut JSXElement) {
        // Check if this is a text or tspan element
        let is_text_element = match &element.opening.name {
            JSXElementName::Ident(ident) => {
                ident.sym.as_ref() == "text" || ident.sym.as_ref() == "tspan"
            }
            _ => false,
        };

        if is_text_element {
            let mut is_superscript = false;
            let mut is_subscript = false;
            let mut font_size = None;
            let mut dy = None;
            let mut element_id = String::new();

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

            // If we detected superscript or subscript characteristics, record the fix
            if is_superscript || is_subscript {
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
                });
            }
        }

        // Continue visiting children
        element.visit_mut_children_with(self);
    }
}

impl SvgPostprocessor {
    /// Create a new SVG postprocessor
    pub fn new() -> Self {
        Self {
            text_fixes: Vec::new(),
        }
    }

    /// Process SVG content by parsing it as JSX and applying transformations
    pub fn process_svg_content(svg_content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Wrap SVG content in a JSX component for parsing
        let jsx_content = format!("const SvgComponent = () => (\n{}\n);", svg_content);

        // For now, return the original content
        // TODO: Implement proper JSX parsing and transformation using SWC
        Ok(svg_content.to_string())
    }

    /// Fix superscripts in SVG text elements using SWC visitor pattern
    pub fn fix_superscripts(svg_content: &str) -> String {
        // For now, return the original content
        // TODO: Implement SWC-based transformation
        svg_content.to_string()
    }

    /// Process SVG content using SWC visitor to detect and fix superscripts
    pub fn process_with_swc_visitor(
        svg_content: &str,
    ) -> Result<(String, Vec<TextFix>), Box<dyn std::error::Error>> {
        // Wrap SVG content in a JSX component for parsing
        let jsx_content = format!("const SvgComponent = () => (\n{}\n);", svg_content);

        // For now, return the original content with empty fixes
        // TODO: Implement proper SWC parsing and visitor processing
        Ok((svg_content.to_string(), Vec::new()))
    }

    /// Optimize SVG paths by simplifying redundant commands
    pub fn optimize_paths(svg_content: &str) -> String {
        // For now, return the original content
        // TODO: Implement SWC-based path optimization
        svg_content.to_string()
    }

    /// Extract text elements from SVG for potential HTML conversion
    pub fn extract_text_elements(svg_content: &str) -> Vec<TextElement> {
        // For now, return empty vector
        // TODO: Implement SWC-based text element extraction
        Vec::new()
    }

    /// Check if a text element is likely a superscript based on context
    fn is_likely_superscript(_svg_content: &str, _text_element: &str) -> bool {
        // For now, return false
        // TODO: Implement SWC-based superscript detection
        false
    }

    /// Postprocess SVG content with all available optimizations
    pub fn postprocess(svg_content: &str) -> String {
        let mut processed = svg_content.to_string();

        // Apply all postprocessing steps
        processed = Self::fix_superscripts(&processed);
        processed = Self::optimize_paths(&processed);

        processed
    }
}

/// Represents a text element extracted from SVG
#[derive(Debug, Clone)]
pub struct TextElement {
    pub content: String,
    pub x: f64,
    pub y: f64,
    pub font_size: f64,
    pub is_superscript: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_text_visitor_detection() {
        // Test that the SWC visitor can detect superscript attributes
        let mut visitor = SvgTextVisitor::new();

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
            children: vec![],
        };

        // Process the element
        visitor.visit_mut_jsx_element(&mut jsx_element);

        // Check that superscript was detected
        let fixes = visitor.get_text_fixes();
        assert_eq!(fixes.len(), 1);
        assert!(fixes[0].is_superscript);
        assert_eq!(fixes[0].font_size, Some(8.0));
    }

    #[test]
    fn test_svg_postprocessor_new() {
        let processor = SvgPostprocessor::new();
        assert!(processor.text_fixes.is_empty());
    }

    #[test]
    fn test_process_svg_content() {
        let svg = r#"<svg><text>Hello</text></svg>"#;
        let result = SvgPostprocessor::process_svg_content(svg).unwrap();
        assert_eq!(result, svg);
    }

    #[test]
    fn test_fix_superscripts() {
        let svg = r#"<text baseline-shift="super">2</text>"#;
        let result = SvgPostprocessor::fix_superscripts(svg);
        assert_eq!(result, svg); // Currently returns original content
    }

    #[test]
    fn test_optimize_paths() {
        let svg = r#"<path d="M 10,10 L 20,20" />"#;
        let result = SvgPostprocessor::optimize_paths(svg);
        assert_eq!(result, svg); // Currently returns original content
    }

    #[test]
    fn test_extract_text_elements() {
        let svg = r#"<text>Hello</text>"#;
        let elements = SvgPostprocessor::extract_text_elements(svg);
        assert!(elements.is_empty()); // Currently returns empty vector
    }

    #[test]
    fn test_postprocess() {
        let svg = r#"<svg><text>Hello</text></svg>"#;
        let result = SvgPostprocessor::postprocess(svg);
        assert_eq!(result, svg); // Currently returns original content
    }
}
