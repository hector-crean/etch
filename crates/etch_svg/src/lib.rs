use roxmltree::{Document, Node};
use std::error::Error;

#[derive(Debug)]
pub struct SvgElement {
    tag: String,
    attributes: Vec<(String, String)>,
    children: Vec<SvgElement>,
}

pub struct SvgParser {
    content: String,
}

impl SvgParser {
    pub fn new(svg_content: &str) -> Self {
        Self {
            content: svg_content.to_string(),
        }
    }

    pub fn parse(&self) -> Result<SvgElement, Box<dyn Error>> {
        let doc = Document::parse(&self.content)?;
        let root = doc.root_element();
        Ok(self.parse_node(root))
    }

    fn parse_node(&self, node: Node) -> SvgElement {
        let tag = node.tag_name().name().to_string();
        let attributes = node
            .attributes()
            .map(|attr| (attr.name().to_string(), attr.value().to_string()))
            .collect();

        let children = node
            .children()
            .filter(|n| n.is_element())
            .map(|n| self.parse_node(n))
            .collect();

        SvgElement {
            tag,
            attributes,
            children,
        }
    }

    /// Generate a TSX string that imports motion and creates a component
    /// that returns the parsed SVG as a `motion` component tree.
    pub fn to_motion_tsx(&self) -> Result<String, Box<dyn Error>> {
        let svg = self.parse()?;

        let mut tsx = String::new();
        tsx.push_str("import { motion } from 'framer-motion';\n\n");
        tsx.push_str("export function PathDrawing() {\n");
        tsx.push_str("    return (\n");
        tsx.push_str(&self.generate_jsx_for_element(&svg, 2));
        tsx.push_str("    );\n");
        tsx.push_str("}\n");

        Ok(tsx)
    }

    fn generate_jsx_for_element(&self, element: &SvgElement, indent_level: usize) -> String {
        let indent = " ".repeat(indent_level * 4);
        let tag_name = format!("motion.{}", element.tag);

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

    fn attributes_to_string(&self, attributes: &[(String, String)]) -> String {
        let mut result = String::new();
        for (name, value) in attributes {
            // Simple attribute handling: assume all attributes can be passed as strings
            result.push_str(&format!(" {}=\"{}\"", name, value));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_svg() {
        let svg_content = r#"<svg viewBox="0 0 100 100">
            <circle cx="50" cy="50" r="40"/>
        </svg>"#;

        let parser = SvgParser::new(svg_content);
        let result = parser.parse();
        assert!(result.is_ok());

        let tsx = parser.to_motion_tsx().unwrap();
        println!("{}", tsx);
        // Check that tsx contains expected motion elements
        assert!(tsx.contains("<motion.svg"));
        assert!(tsx.contains("<motion.circle"));
    }
}
