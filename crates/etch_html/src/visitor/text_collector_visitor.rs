use crate::rc_dom::Handle;
use html5ever::{Attribute, QualName};
use std::cell::RefCell;
use std::collections::HashMap;

use tendril::StrTendril;

use super::NodeVisitor;

/// A visitor that collects text content associated with HTML elements that have ID attributes.
///
/// This visitor traverses the DOM tree and maintains a mapping between element IDs and their
/// contained text content.
///
/// # Example
/// ```
/// let visitor = TextCollectorVisitor::new();
/// // ... traverse DOM ...
/// let text_map = visitor.text_map();
/// // Access text content by element ID
/// if let Some(content) = text_map.get("my-element-id") {
///     println!("Text content: {}", content);
/// }
/// ```
pub struct TextCollectorVisitor {
    /// Maps element IDs to their accumulated text content
    text_map: HashMap<String, String>,
    /// Tracks the ID of the current element being visited
    current_id: Option<String>,
}

impl Default for TextCollectorVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl TextCollectorVisitor {
    /// Creates a new TextCollectorVisitor with empty text mappings
    pub fn new() -> Self {
        TextCollectorVisitor {
            text_map: HashMap::new(),
            current_id: None,
        }
    }

    /// Returns a reference to the collected text mappings
    ///
    /// # Returns
    /// A HashMap where keys are element IDs and values are the accumulated text content
    pub fn text_map(&self) -> &HashMap<String, String> {
        &self.text_map
    }
}

impl NodeVisitor for TextCollectorVisitor {
    /// Processes an element node in the DOM tree
    ///
    /// # Arguments
    /// * `name` - The qualified name of the element
    /// * `attrs` - The element's attributes
    /// * `template_contents` - Contents if this is a template element
    /// * `mathml_annotation_xml_integration_point` - MathML integration flag
    /// * `handle` - Reference to the DOM node
    ///
    /// # Returns
    /// A tuple containing an optional handle and a boolean indicating whether to continue traversal
    fn visit_element(
        &mut self,
        name: &QualName,
        attrs: &RefCell<Vec<Attribute>>,
        template_contents: &RefCell<Option<Handle>>,
        mathml_annotation_xml_integration_point: bool,
        handle: &Handle,
    ) -> (Option<Handle>, bool) {
        // Check if the element has an ID attribute
        let id = attrs
            .borrow()
            .iter()
            .find(|attr| attr.name.local.to_string() == "id")
            .map(|attr| attr.value.to_string());

        // Update the current_id
        self.current_id = id;

        (None, true)
    }

    /// Processes a text node in the DOM tree
    ///
    /// If the text node is within an element that has an ID, its content is accumulated
    /// in the text_map under that ID.
    ///
    /// # Arguments
    /// * `contents` - The text contents of the node
    /// * `handle` - Reference to the DOM node
    ///
    /// # Returns
    /// A tuple containing an optional handle and a boolean indicating whether to continue traversal
    fn visit_text(
        &mut self,
        contents: &RefCell<StrTendril>,
        handle: &Handle,
    ) -> (Option<Handle>, bool) {
        if let Some(id) = &self.current_id {
            let text = contents.borrow().to_string();
            self.text_map
                .entry(id.clone())
                .and_modify(|existing| *existing += &text)
                .or_insert(text);
        }
        (None, true)
    }
}
