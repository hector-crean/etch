use super::NodeVisitor;
use crate::rc_dom::{Handle, Node, NodeData};
use html5ever::{Attribute, LocalName, QualName};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::{cell::RefCell, collections::HashMap};
use markup5ever::{namespace_url, ns};

const RICH_TEXT_TAG: &str = "rich-text";

/// A visitor that transforms text nodes and certain HTML elements into rich-text elements
/// with unique identifiers, maintaining a mapping between IDs and their HTML content.
/// 
/// This visitor specifically handles:
/// - Text nodes: Converts them into <rich-text> elements with sanitized content
/// - Lists (ul/ol): Wraps their content in <rich-text> elements
/// - List items (li): Preserves structure while sanitizing content
/// - Paragraphs (p): Wraps their content in <rich-text> elements
#[derive(Debug, Serialize, Deserialize)]
pub struct RichTextTransformVisitor {
    /// Maps unique identifiers to their corresponding HTML content
    rich_text_mappings: HashMap<Uuid, String>,
}

impl Default for RichTextTransformVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl RichTextTransformVisitor {
    pub fn new() -> Self {
        RichTextTransformVisitor {
            rich_text_mappings: HashMap::new(),
        }
    }

    /// Returns a reference to the mapping of rich text IDs to their HTML content
    pub fn rich_text_mappings(&self) -> &HashMap<Uuid, String> {
        &self.rich_text_mappings
    }
}

impl NodeVisitor for RichTextTransformVisitor {
    fn visit_text(
        &mut self,
        text_contents: &RefCell<tendril::StrTendril>,
        handle: &Handle,
    ) -> (Option<Handle>, bool) {
        let text = text_contents.borrow();
        if !text.trim().is_empty() {
            log::info!("Visiting non-empty text node: {:?}", text);

            let sanitized_text = sanitize_text(&text);
            let text_node = create_text_node(&sanitized_text);
            let uuid = uuid::Uuid::new_v4();
            let new_attrs: Vec<Attribute> = vec![Attribute {
                name: QualName::new(None, ns!(), LocalName::from("id")),
                value: uuid.to_string().into(),
            }];
            let rich_text_node = create_element(RICH_TEXT_TAG, new_attrs, vec![text_node]);
            
            // Insert the UUID and inner HTML into the rich_text_mappings
            self.rich_text_mappings.insert(uuid, sanitized_text);
            
            (Some(rich_text_node), false)
        } else {
            log::debug!("Skipping empty or whitespace-only text node");
            (None, true)
        }
    }
    fn visit_element(
        &mut self,
        element_name: &QualName,
        element_attrs: &RefCell<Vec<Attribute>>,
        template_contents: &RefCell<Option<Handle>>,
        mathml_annotation_xml_integration_point: bool,
        handle: &Handle,
    ) -> (Option<Handle>, bool) {
        let element_name = element_name.local.as_ref();
        match element_name {
            lists @ ("ul" | "ol") => {
                log::info!("Visiting list element");
              
                let sanitized_children = sanitize_children(&handle.children.borrow());
                let uuid = uuid::Uuid::new_v4();
                let rich_text_attrs: Vec<Attribute> = vec![Attribute {
                    name: QualName::new(None, ns!(), LocalName::from("id")),
                    value: uuid.to_string().into(),
                }];
                let rich_text_node = create_element(RICH_TEXT_TAG, rich_text_attrs, sanitized_children.clone());
                
                // Insert the UUID and inner HTML into the rich_text_mappings
                let inner_html = rich_text_node.to_html_string();
                self.rich_text_mappings.insert(uuid, inner_html);
                
                let new_node = create_element(lists, element_attrs.borrow().clone(), vec![rich_text_node]);
                (Some(new_node), true)
            },
            li @ "li" => {
                log::info!("Visiting list item element");       
                let sanitized_children = sanitize_children(&handle.children.borrow());
                let new_node = create_element(li, element_attrs.borrow().clone(), sanitized_children);
                (Some(new_node), false)
            },
            paragraph @ "p" => {
                log::info!("Visiting paragraph element");
                let sanitized_children = sanitize_children(&handle.children.borrow());
                let uuid = uuid::Uuid::new_v4();
                let rich_text_attrs: Vec<Attribute> = vec![Attribute {
                    name: QualName::new(None, ns!(), LocalName::from("id")),
                    value: uuid.to_string().into(),
                }];
                let rich_text_node = create_element(RICH_TEXT_TAG, rich_text_attrs, sanitized_children.clone());
               
                // Insert the UUID and inner HTML into the rich_text_mappings
                let inner_html = rich_text_node.to_html_string();
                self.rich_text_mappings.insert(uuid, inner_html);
               
                let new_node = create_element(paragraph, element_attrs.borrow().clone(), vec![rich_text_node]);
                (Some(new_node), false)
            },
            _ => (None, true)
        }
    }
}

/// Processes a vector of child nodes, sanitizing any text content while preserving structure.
/// 
/// Returns a new vector of processed child nodes.
fn sanitize_children(children: &Vec<Handle>) -> Vec<Handle> {
    children.iter().map(|child| {
        match child.data {
            NodeData::Text { ref contents } => {
                let sanitized_text = sanitize_text(&contents.borrow());
                create_text_node(&sanitized_text)
            },
            _ => child.clone(),
        }
    }).collect()
}

/// Creates a new DOM element with the specified name, attributes, and children.
/// 
/// # Arguments
/// * `name` - The HTML tag name for the element
/// * `attrs` - A vector of attributes to apply to the element
/// * `children` - A vector of child nodes to attach to this element
fn create_element(name: &str, attrs: Vec<Attribute>, children: Vec<Handle>) -> Handle {
    let element = NodeData::Element {
        name: html5ever::QualName::new(
            None,
            html5ever::ns!(),
            html5ever::LocalName::from(name),
        ),
        attrs: RefCell::new(attrs),
        template_contents: RefCell::new(None),
        mathml_annotation_xml_integration_point: false,
    };
    let handle = Node::new(element);

    *handle.children.borrow_mut() = children;
    handle
}

/// Creates a new text node with the specified content.
/// 
/// # Arguments
/// * `content` - The text content for the node
fn create_text_node(content: &str) -> Handle {
    Node::new(NodeData::Text {
        contents: RefCell::new(content.into()),
    })
}

/// Sanitizes text content by:
/// - Removing leading/trailing whitespace
/// - Removing empty lines
/// - Normalizing spaces (converting non-breaking spaces and multiple spaces)
/// 
/// # Arguments
/// * `text` - The text content to sanitize
fn sanitize_text(text: &str) -> String {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
        .replace('\u{00A0}', " ") // Replace non-breaking spaces with regular spaces
        .replace("  ", " ") // Replace double spaces with single spaces
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    

    fn format_html(html: &str, indent: usize) -> String {
        let mut formatted = String::new();
        let mut depth: usize = 0;
        let mut chars = html.chars().peekable();
        let indent_str = " ".repeat(indent);

        while let Some(c) = chars.next() {
            match c {
                '<' => {
                    if chars.peek() == Some(&'/') {
                        depth = depth.saturating_sub(1);
                        formatted.push('\n');
                        formatted.push_str(&indent_str.repeat(depth));
                    } else {
                        formatted.push('\n');
                        formatted.push_str(&indent_str.repeat(depth));
                        depth += 1;
                    }
                    formatted.push('<');
                }
                '>' => {
                    formatted.push('>');
                    if chars.peek().is_some_and(|&next| next != '<') {
                        formatted.push('\n');
                        formatted.push_str(&indent_str.repeat(depth));
                    }
                }
                _ => formatted.push(c),
            }
        }
        formatted
    }

    #[test]
    fn test_rich_text_transformation() -> Result<(), std::io::Error> {
        // Create a sample HTML string with mixed content
        let html = r#"
            <div>
                <p>Hello, world!</p>
                <ul>
                    <li>First item</li>
                    <li>Second item</li>
                </ul>
                <p>Another paragraph with <strong>bold text</strong></p>
            </div>
        "#;

 
        // Create and apply the visitor
        let visitor = RichTextTransformVisitor::new();
        let (updated_dom, visitor) = crate::file::process_html_str(html, visitor)?;



        // Get the mappings
        let mappings = visitor.rich_text_mappings();

      

        // Print the transformed HTML and mappings for inspection
        println!("Transformed HTML:");
        println!("{}", format_html(&updated_dom.to_string(), 2));

        println!("\nRich Text Mappings:");
        for (id, content) in mappings {
            println!("ID: {}\nContent: {}\n", id, content);
        }

        Ok(())
    }
}
