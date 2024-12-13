use std::path::Path;
use crate::{rc_dom::RcDom, visitor::NodeVisitor};


pub fn process_html_file<P: AsRef<Path>, V: NodeVisitor>(
    file_path: P,
    mut visitor: V,
) -> Result<(String, V), std::io::Error> {
    // Read the HTML file
    let dom = RcDom::from_file(file_path)?;

    // Run the visitor
    let (node_handle, _) = visitor.traverse(dom.document);
    let updated_html = node_handle.to_html_string();

    Ok((updated_html, visitor))
}


pub fn process_html_str<V: NodeVisitor>(
    html: &str,
    mut visitor: V,
) -> Result<(String, V), std::io::Error> {
    // Read the HTML file
    let dom = RcDom::from_str(html);

    // Run the visitor
    let (node_handle, _) = visitor.traverse(dom.document);
    let updated_html = node_handle.to_html_string();

    Ok((updated_html, visitor))
}