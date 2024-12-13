pub mod document_id_visitor;
pub mod text_collector_visitor;
pub mod rich_text_transform_visitor;
pub mod svg_extractor_visitor;

use crate::rc_dom::{Handle, NodeData};
use html5ever::{Attribute, QualName};
use std::cell::RefCell;
use tendril::StrTendril;

pub use document_id_visitor::DocumentIdVisitor;
pub use text_collector_visitor::TextCollectorVisitor;
pub use rich_text_transform_visitor::RichTextTransformVisitor;
pub use svg_extractor_visitor::SvgExtractVisitor;

pub type ReplaceNode = Option<Handle>;
pub type ContinueToChildren = bool;

/// A trait for visiting and potentially modifying nodes in an HTML DOM tree.
///
/// Implementors of this trait can define custom behavior for different types of nodes
/// in the DOM tree. The visitor pattern allows for operations to be performed on an
/// object structure without changing the structure itself.
pub trait NodeVisitor {
    /// Visit a document node.
    ///
    /// # Returns
    /// A tuple `(ReplaceNode, ContinueToChildren)` where:
    /// - The `Option<Handle>` is `Some` if the node should be replaced, or `None` if unchanged.
    /// - The `bool` indicates whether to continue visiting this node's children (true) or not (false).
    fn visit_document(&mut self, handle: &Handle) -> (ReplaceNode, ContinueToChildren) {
        (None, true)
    }

    /// Visit a doctype node.
    ///
    /// # Returns
    /// A tuple `(ReplaceNode, ContinueToChildren)` where:
    /// - The `Option<Handle>` is `Some` if the node should be replaced, or `None` if unchanged.
    /// - The `bool` indicates whether to continue visiting this node's children (true) or not (false).
    fn visit_doctype(
        &mut self,
        name: &StrTendril,
        public_id: &StrTendril,
        system_id: &StrTendril,
        handle: &Handle,
    ) -> (ReplaceNode, ContinueToChildren) {
        (None, true)
    }

    /// Visit a text node.
    ///
    /// # Returns
    /// A tuple `(ReplaceNode, ContinueToChildren)` where:
    /// - The `Option<Handle>` is `Some` if the node should be replaced, or `None` if unchanged.
    /// - The `bool` indicates whether to continue visiting this node's children (true) or not (false).
    fn visit_text(
        &mut self,
        contents: &RefCell<StrTendril>,
        handle: &Handle,
    ) -> (ReplaceNode, ContinueToChildren) {
        (None, true)
    }

    /// Visit a comment node.
    ///
    /// # Returns
    /// A tuple `(ReplaceNode, ContinueToChildren)` where:
    /// - The `Option<Handle>` is `Some` if the node should be replaced, or `None` if unchanged.
    /// - The `bool` indicates whether to continue visiting this node's children (true) or not (false).
    fn visit_comment(&mut self, contents: &StrTendril, handle: &Handle) -> (ReplaceNode, ContinueToChildren) {
        (None, true)
    }

    /// Visit an element node.
    ///
    /// # Returns
    /// A tuple `(ReplaceNode, ContinueToChildren)` where:
    /// - The `Option<Handle>` is `Some` if the node should be replaced, or `None` if unchanged.
    /// - The `bool` indicates whether to continue visiting this node's children (true) or not (false).
    fn visit_element(
        &mut self,
        name: &QualName,
        attrs: &RefCell<Vec<Attribute>>,
        template_contents: &RefCell<Option<Handle>>,
        mathml_annotation_xml_integration_point: bool,
        handle: &Handle,
    ) -> (ReplaceNode, ContinueToChildren) {
        (None, true)
    }

    /// Visit a processing instruction node.
    ///
    /// # Returns
    /// A tuple `(ReplaceNode, ContinueToChildren)` where:
    /// - The `Option<Handle>` is `Some` if the node should be replaced, or `None` if unchanged.
    /// - The `bool` indicates whether to continue visiting this node's children (true) or not (false).
    fn visit_processing_instruction(
        &mut self,
        target: &StrTendril,
        contents: &StrTendril,
        handle: &Handle,
    ) -> (ReplaceNode, ContinueToChildren) {
        (None, true)
    }

    /// Traverses the DOM tree starting from the given node, applying the visitor to each node.
    ///
    /// This method implements the depth-first traversal logic, calling the appropriate
    /// `visit_*` method for each node type. It also handles the replacement of nodes
    /// and continuing to child nodes based on the return values of the `visit_*` methods.
    ///
    /// # Arguments
    /// * `handle` - The root node to start traversing from.
    ///
    /// # Returns
    /// A tuple `(Handle, bool)` where:
    /// - The `Handle` is the possibly modified node (or a replacement node).
    /// - The `bool` is always true for the root call (unused, but kept for consistency
    ///   with other methods).
    fn traverse(&mut self, handle: Handle) -> (Handle, bool) {
        let (new_node, continue_children) = match handle.data {
            NodeData::Document => self.visit_document(&handle),
            NodeData::Doctype {
                ref name,
                ref public_id,
                ref system_id,
            } => self.visit_doctype(name, public_id, system_id, &handle),
            NodeData::Text { ref contents } => self.visit_text(contents, &handle),
            NodeData::Comment { ref contents } => self.visit_comment(contents, &handle),
            NodeData::Element {
                ref name,
                ref attrs,
                ref template_contents,
                mathml_annotation_xml_integration_point,
            } => self.visit_element(
                name,
                attrs,
                template_contents,
                mathml_annotation_xml_integration_point,
                &handle,
            ),
            NodeData::ProcessingInstruction {
                ref target,
                ref contents,
            } => self.visit_processing_instruction(target, contents, &handle),
        };

        let node = new_node.unwrap_or(handle);

        if continue_children {
            let mut new_children = Vec::new();
            for child in node.children.borrow().iter() {
                let (new_child, _) = self.traverse(child.clone());
                new_children.push(new_child);
            }
            *node.children.borrow_mut() = new_children;
        }

        (node, true)
    }
}
