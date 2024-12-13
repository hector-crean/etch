use crate::rc_dom::{Handle, NodeData};
use html5ever::{Attribute, QualName};
use std::cell::RefCell;
use std::collections::HashMap;

use super::NodeVisitor;

pub struct DocumentIdVisitor {
    document_id_cursor: usize,
    ref_map: HashMap<String, usize>,
}

impl Default for DocumentIdVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentIdVisitor {
    pub fn new() -> Self {
        Self {
            document_id_cursor: 1,
            ref_map: HashMap::new(),
        }
    }
}

impl NodeVisitor for DocumentIdVisitor {
    fn visit_element(
        &mut self,
        name: &QualName,
        attrs: &RefCell<Vec<Attribute>>,
        _template_contents: &RefCell<Option<Handle>>,
        _mathml_annotation_xml_integration_point: bool,
        handle: &Handle,
    ) -> (Option<Handle>, bool) {
        let tag_name = name.local.as_ref();
        if tag_name == "a" {
            let mut ref_uuid = None;
            let mut should_update = false;

            for attr in attrs.borrow().iter() {
                if attr.name.local.as_ref() == "data-ref" {
                    ref_uuid = Some(attr.value.to_string());
                    should_update = true;
                    break;
                }
            }

            if should_update {
                if let Some(uuid) = ref_uuid {
                    let document_id = self.ref_map.entry(uuid).or_insert_with(|| {
                        let id = self.document_id_cursor;
                        self.document_id_cursor += 1;
                        id
                    });

                    // Update the text content of the <a> element
                    if let Some(first_child) = handle.children.borrow().first() {
                        if let NodeData::Text { ref contents } = first_child.data {
                            *contents.borrow_mut() = document_id.to_string().into();
                        }
                    }
                }
            }
        }
        (None, true)
    }

    // No need to implement visit_text for this visitor
}
