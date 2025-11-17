//! JSX hierarchy building utilities
//!
//! This module provides functionality for building hierarchical JSX structures
//! from the flat map of elements produced by the visitor.

use std::collections::HashMap;
use swc_ecma_ast::{JSXElement, JSXElementChild};

/// Build hierarchical JSX elements from a flat structure
pub struct JsxHierarchyBuilder<'a> {
    /// All JSX elements by ID
    jsx_elements: &'a HashMap<String, JSXElement>,
    
    /// Parent-child relationships
    parent_child_map: &'a HashMap<String, Vec<String>>,
}

impl<'a> JsxHierarchyBuilder<'a> {
    /// Create a new JSX hierarchy builder
    pub fn new(
        jsx_elements: &'a HashMap<String, JSXElement>,
        parent_child_map: &'a HashMap<String, Vec<String>>,
    ) -> Self {
        Self {
            jsx_elements,
            parent_child_map,
        }
    }

    /// Build a hierarchical JSX element starting from a root node
    ///
    /// This recursively builds the JSX tree by:
    /// 1. Getting the element for this node
    /// 2. Finding all its children
    /// 3. Recursively building children
    /// 4. Inserting children into the parent element
    pub fn build_hierarchy(&self, node_id: &str) -> Option<JSXElement> {
        // Get the base element for this node
        let mut element = self.jsx_elements.get(node_id)?.clone();

        // Get children for this node
        if let Some(child_ids) = self.parent_child_map.get(node_id) {
            // Build child elements recursively
            let children: Vec<JSXElementChild> = child_ids
                .iter()
                .filter_map(|child_id| {
                    self.build_hierarchy(child_id)
                        .map(|child_element| JSXElementChild::JSXElement(Box::new(child_element)))
                })
                .collect();

            // Insert children into the parent element
            element.children = children;
        }

        Some(element)
    }

    /// Build all root elements with full hierarchy
    pub fn build_all_roots(&self, root_ids: &[String]) -> HashMap<String, JSXElement> {
        let mut result = HashMap::new();

        for root_id in root_ids {
            if let Some(element) = self.build_hierarchy(root_id) {
                result.insert(root_id.clone(), element);
            }
        }

        result
    }

    /// Build root elements with their semantic names
    pub fn build_named_roots(
        &self,
        root_ids: &[String],
        node_names: &HashMap<String, String>,
    ) -> HashMap<String, (String, JSXElement)> {
        let mut result = HashMap::new();

        for root_id in root_ids {
            if let Some(element) = self.build_hierarchy(root_id) {
                let name = node_names
                    .get(root_id)
                    .cloned()
                    .unwrap_or_else(|| format!("Component_{}", root_id));
                result.insert(name, (root_id.clone(), element));
            }
        }

        result
    }

    /// Build only exportable elements
    pub fn build_exportable_roots(
        &self,
        root_ids: &[String],
        node_names: &HashMap<String, String>,
        exportable_nodes: &HashMap<String, bool>,
    ) -> HashMap<String, (String, JSXElement)> {
        let mut result = HashMap::new();

        for root_id in root_ids {
            // Check if this is an exportable root
            let is_exportable = exportable_nodes.get(root_id).copied().unwrap_or(false);
            
            if is_exportable {
                if let Some(element) = self.build_hierarchy(root_id) {
                    let name = node_names
                        .get(root_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Component_{}", root_id));
                    result.insert(name, (root_id.clone(), element));
                }
            }
        }

        result
    }
}

