use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use log::info;

/// A visitor that transforms SVG content into a React component
pub struct SvgReactVisitor {
    svg_content: String,
    component_name: String,
    use_motion: bool,
}

impl SvgReactVisitor {
    pub fn new(svg_content: String, component_name: String, use_motion: bool) -> Self {
        Self {
            svg_content,
            component_name,
            use_motion,
        }
    }

    /// Generate a React component module from SVG content
    pub fn generate_component(&self) -> Result<String, Box<dyn std::error::Error>> {
        let parser = SvgParser::new(&self.svg_content);
        
        if self.use_motion {
            // Use the existing motion functionality
            return parser.to_motion_tsx();
        } else {
            // Generate a standard React component
            let svg = parser.parse()?;
            
            let mut jsx = String::new();
            jsx.push_str(&format!("import React from 'react';\n\n"));
            jsx.push_str(&format!("export function {}(props) {{\n", self.component_name));
            jsx.push_str("    return (\n");
            jsx.push_str(&self.generate_jsx_for_element(&svg, 2));
            jsx.push_str("    );\n");
            jsx.push_str("}\n");
            
            Ok(jsx)
        }
    }

    /// Generate JSX from the parsed SVG element
    fn generate_jsx_for_element(&self, element: &etch_svg::SvgElement, indent_level: usize) -> String {
        let indent = " ".repeat(indent_level * 4);
        let tag_name = &element.tag;

        let attrs_str = self.attributes_to_string(&element.attributes);
        if element.children.is_empty() {
            // self-closing tag
            format!("{}<{}{} />\n", indent, tag_name, attrs_str)
        } else {
            // opening tag
            let mut result = format!("{}<{}{}>\n", indent, tag_name, attrs_str);
            for child in &element.children {
                result.push_str(&self.generate_jsx_for_element(child, indent_level + 1));
            }
            result.push_str(&format!("{}</{}>\n", indent, tag_name));
            result
        }
    }

    /// Convert SVG attributes to React-compatible attributes
    fn attributes_to_string(&self, attributes: &[(String, String)]) -> String {
        let mut result = String::new();
        for (name, value) in attributes {
            // Handle React-specific attribute naming conversions
            let react_attr = match name.as_str() {
                "class" => "className",
                "stroke-width" => "strokeWidth",
                "stroke-linecap" => "strokeLinecap",
                "stroke-linejoin" => "strokeLinejoin",
                "fill-rule" => "fillRule",
                "clip-rule" => "clipRule",
                "stroke-dasharray" => "strokeDasharray",
                "stroke-dashoffset" => "strokeDashoffset",
                "xlink:href" => "xlinkHref",
                _ => name,
            };
            
            result.push_str(&format!(" {}=\"{}\"", react_attr, value));
        }
        result
    }
}