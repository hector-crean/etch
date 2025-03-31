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

    // Create a React component from the parsed SVG
    pub fn to_react_component(&self, component_name: &str) -> Result<String, Box<dyn Error>> {
        let svg_element = self.parse()?;
        let jsx_content = svg_element.to_react_jsx();
        
        Ok(format!(
            "import React from 'react';\n\n\
            const {} = () => (\n\
              {}\n\
            );\n\n\
            export default {};",
            component_name, jsx_content, component_name
        ))
    }
}

impl SvgElement {
    // Convert SVG element to React JSX string
    pub fn to_react_jsx(&self) -> String {
        let attrs = self.format_react_attributes();
        
        if self.children.is_empty() {
            format!("<{}{} />", self.tag, attrs)
        } else {
            let children = self.children
                .iter()
                .map(|child| child.to_react_jsx())
                .collect::<Vec<_>>()
                .join("\n");
            
            format!("<{}{}>\n{}\n</{}>", self.tag, attrs, children, self.tag)
        }
    }
    
    // Format attributes for React (handling special cases)
    fn format_react_attributes(&self) -> String {
        self.attributes
            .iter()
            .map(|(name, value)| {
                // Convert attribute names to React style (camelCase, className, etc.)
                let react_attr = match name.as_str() {
                    "class" => "className".to_string(),
                    "stroke-width" => "strokeWidth".to_string(),
                    "fill-opacity" => "fillOpacity".to_string(),
                    "stroke-opacity" => "strokeOpacity".to_string(),
                    "stroke-linecap" => "strokeLinecap".to_string(),
                    "stroke-linejoin" => "strokeLinejoin".to_string(),
                    // Add more mappings as needed
                    _ => name.clone(),
                };
                
                format!(" {}=\"{}\"", react_attr, value)
            })
            .collect::<String>()
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
        
        let svg = result.unwrap();
        assert_eq!(svg.tag, "svg");
        assert_eq!(svg.children.len(), 1);
        assert_eq!(svg.children[0].tag, "circle");
    }
    #[test]
    fn test_tsx_file() {
        let svg_content = r#"<svg viewBox="0 0 100 100">
    <circle cx="50" cy="50" r="40"/>
</svg>"#;

        let parser = SvgParser::new(svg_content);
        let react_component = parser.to_react_component("CircleIcon").unwrap();
        
        // Verify the component contains the React import
        assert!(react_component.contains("import React from 'react';"));
        
        // Verify the component name is used correctly
        assert!(react_component.contains("const CircleIcon = () =>"));
        
        // Verify the SVG attributes are included
        assert!(react_component.contains("<svg viewBox=\"0 0 100 100\">"));
        
        // Verify the circle element is included with correct attributes
        assert!(react_component.contains("<circle cx=\"50\" cy=\"50\" r=\"40\" />"));
        
        // Verify export statement
        assert!(react_component.contains("export default CircleIcon;"));
    }
}



