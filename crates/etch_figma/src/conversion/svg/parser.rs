use etch_svgr::{
    clean_svg_content as svgr_clean_svg_content, parse_svg_to_jsx as svgr_parse_svg_to_jsx,
};
use swc_ecma_ast::*;

/// Errors that can occur when parsing SVG into JSX elements
#[derive(Debug, thiserror::Error)]
pub enum SvgParseError {
    #[error("Failed to parse SVG: {0}")]
    ParseError(String),
    #[error("No JSX element found in parsed SVG content")]
    NoJsxElement,
    #[error("Empty SVG content provided")]
    EmptyContent,
    #[error("SVG content is not valid XML")]
    InvalidXml,
}

/// Parses SVG content and converts it to JSX elements using svgr-rs
pub struct SvgParser;

impl SvgParser {
    /// Parse SVG content and return a JSX element using etch_svgr
    pub fn parse_svg_to_jsx(svg_content: &str) -> Result<JSXElement, SvgParseError> {
        if svg_content.trim().is_empty() {
            return Err(SvgParseError::EmptyContent);
        }

        // Clean the SVG content first
        let cleaned_svg = Self::clean_svg_content(svg_content);

        // Use our updated svgr implementation
        svgr_parse_svg_to_jsx(&cleaned_svg).map_err(|e| SvgParseError::ParseError(e.to_string()))
    }

    /// Clean up SVG content by removing unnecessary attributes and elements
    pub fn clean_svg_content(svg_content: &str) -> String {
        svgr_clean_svg_content(svg_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_svg() {
        let svg_content = r#"
            <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
                <circle cx="50" cy="50" r="40" fill="red"/>
                <text x="50" y="50" text-anchor="middle">Hello</text>
            </svg>
        "#;

        let result = SvgParser::parse_svg_to_jsx(svg_content);
        assert!(result.is_ok());

        let jsx_element = result.unwrap();

        // Check that it's an SVG element
        if let JSXElementName::Ident(ident) = &jsx_element.opening.name {
            assert_eq!(ident.sym.as_ref(), "svg");
        }

        // Check that it has children
        assert!(!jsx_element.children.is_empty());
    }

    #[test]
    fn test_clean_svg_content() {
        let svg_with_xml_decl = r#"<?xml version="1.0" encoding="UTF-8"?>
            <svg viewBox="0 0 100 100">
                <circle cx="50" cy="50" r="40"/>
            </svg>
        "#;

        let cleaned = SvgParser::clean_svg_content(svg_with_xml_decl);
        assert!(!cleaned.contains("<?xml"));
        assert!(cleaned.contains("<svg"));
    }

    #[test]
    fn test_parse_complex_svg() {
        let svg_content = r#"
            <svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
                <defs>
                    <linearGradient id="grad1" x1="0%" y1="0%" x2="100%" y2="0%">
                        <stop offset="0%" style="stop-color:rgb(255,255,0);stop-opacity:1" />
                        <stop offset="100%" style="stop-color:rgb(255,0,0);stop-opacity:1" />
                    </linearGradient>
                </defs>
                <path d="M 10 10 L 100 10 L 100 100 L 10 100 Z" fill="url(#grad1)"/>
            </svg>
        "#;

        let result = SvgParser::parse_svg_to_jsx(svg_content);
        assert!(result.is_ok());

        let jsx_element = result.unwrap();

        // Check that it's an SVG element
        if let JSXElementName::Ident(ident) = &jsx_element.opening.name {
            assert_eq!(ident.sym.as_ref(), "svg");
        }

        // Check that it has children (defs and path)
        assert!(jsx_element.children.len() >= 2);
    }
}
