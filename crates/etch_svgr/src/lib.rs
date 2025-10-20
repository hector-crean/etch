use std::rc::Rc;

use swc_common::{FileName, SourceMap};
use swc_ecma_ast::*;
use swc_xml::parser::parse_file_as_document;

mod error;
mod hast_to_swc_ast;

pub use error::SvgrError;

/// Transform SVG into JSX elements.
///
/// It takes SVG source code and returns a JSX element that can be used in React components.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// use etch_svgr::parse_svg_to_jsx;
///
/// let result = parse_svg_to_jsx(r#"<svg></svg>"#);
/// ```
pub fn parse_svg_to_jsx(svg_content: &str) -> Result<JSXElement, SvgrError> {
    let cm = Rc::<SourceMap>::default();
    let fm = cm.new_source_file(FileName::Anon.into(), svg_content.to_string());

    let mut errors = vec![];
    let document = parse_file_as_document(fm.as_ref(), Default::default(), &mut errors)
        .map_err(|e| SvgrError::Parse(e.message().to_string()))?;

    let jsx_element = hast_to_swc_ast::to_swc_ast(document);
    if jsx_element.is_none() {
        return Err(SvgrError::InvalidSvg);
    }
    let jsx_element = jsx_element.unwrap();

    Ok(jsx_element)
}

/// Clean up SVG content by removing unnecessary attributes and elements
pub fn clean_svg_content(svg_content: &str) -> String {
    // Remove XML declaration if present
    let cleaned = if svg_content.trim_start().starts_with("<?xml") {
        if let Some(end) = svg_content.find("?>") {
            svg_content[end + 2..].trim_start()
        } else {
            svg_content
        }
    } else {
        svg_content
    };

    // Remove DOCTYPE if present
    let cleaned = if cleaned.trim_start().starts_with("<!DOCTYPE") {
        if let Some(end) = cleaned.find('>') {
            cleaned[end + 1..].trim_start()
        } else {
            cleaned
        }
    } else {
        cleaned
    };

    cleaned.to_string()
}
