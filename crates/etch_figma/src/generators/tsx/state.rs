//! State management for TSX visitor
//!
//! This module contains the state tracking structures used during
//! Figma-to-TSX conversion.

use figma_api::models::VectorNode;
use std::collections::{HashMap, HashSet};
use swc_ecma_ast::JSXElement;

use crate::analysis::svg_container::ContainerAnalysis;
use crate::conversion::svg::RenderingStrategy;
use crate::extensions::tailwind::TailwindStyles;

/// State container for TSX visitor
///
/// Tracks all the mutable state during Figma tree traversal and conversion.
/// This is separated from the visitor logic to make state management clearer.
#[derive(Debug)]
pub struct VisitorState {
    /// Generated JSX elements by node ID for all node types
    pub jsx_elements: HashMap<String, JSXElement>,
    
    /// Generated styles by node ID (for debugging/inspection)
    pub node_styles: HashMap<String, TailwindStyles>,
    
    /// Track node types for debugging
    pub node_types: HashMap<String, String>,
    
    /// Track parent-child relationships
    pub parent_child_map: HashMap<String, Vec<String>>,
    
    /// Track which nodes are root nodes (no parent) - stores node IDs
    pub root_nodes: Vec<String>,
    
    /// Stack to track current parent during traversal
    pub parent_stack: Vec<String>,
    
    /// Map node IDs to their semantic names
    pub node_names: HashMap<String, String>,
    
    /// Track which nodes are marked for export (have export settings)
    pub exportable_nodes: HashMap<String, bool>,
    
    /// Track rendering strategies for nodes
    pub rendering_strategies: HashMap<String, RenderingStrategy>,
    
    /// Container analyses for SVG wrapping decisions
    pub container_analyses: HashMap<String, ContainerAnalysis>,
    
    /// Track vectors awaiting SVG data
    pub pending_vectors: HashMap<String, VectorNode>,
    
    /// Store temporary placeholder JSX elements
    pub placeholder_jsx: HashMap<String, JSXElement>,
    
    /// Track nodes that are part of SVG groups (should be skipped during individual processing)
    pub grouped_svg_nodes: HashSet<String>,
    
    /// Counter for generating unique SVG group IDs
    pub svg_group_counter: usize,
}

impl Default for VisitorState {
    fn default() -> Self {
        Self {
            jsx_elements: HashMap::new(),
            node_styles: HashMap::new(),
            node_types: HashMap::new(),
            parent_child_map: HashMap::new(),
            root_nodes: Vec::new(),
            parent_stack: Vec::new(),
            node_names: HashMap::new(),
            exportable_nodes: HashMap::new(),
            rendering_strategies: HashMap::new(),
            container_analyses: HashMap::new(),
            pending_vectors: HashMap::new(),
            placeholder_jsx: HashMap::new(),
            grouped_svg_nodes: HashSet::new(),
            svg_group_counter: 0,
        }
    }
}

impl VisitorState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self::default()
    }

    /// Store a JSX element with metadata
    pub fn store_element(
        &mut self,
        id: String,
        name: String,
        element: JSXElement,
        node_type: &str,
        styles: Option<TailwindStyles>,
    ) {
        self.jsx_elements.insert(id.clone(), element);
        self.node_names.insert(id.clone(), name);
        self.node_types.insert(id.clone(), node_type.to_string());

        if let Some(styles) = styles {
            self.node_styles.insert(id, styles);
        }
    }

    /// Get the current parent node ID
    pub fn current_parent(&self) -> Option<&String> {
        self.parent_stack.last()
    }

    /// Push a parent onto the stack
    pub fn push_parent(&mut self, parent_id: String) {
        self.parent_stack.push(parent_id);
    }

    /// Pop a parent from the stack
    pub fn pop_parent(&mut self) -> Option<String> {
        self.parent_stack.pop()
    }

    /// Add a parent-child relationship
    pub fn add_child_relationship(&mut self, parent_id: String, child_id: String) {
        self.parent_child_map
            .entry(parent_id)
            .or_insert_with(Vec::new)
            .push(child_id);
    }

    /// Mark a node as a root node
    pub fn mark_as_root(&mut self, node_id: String) {
        if !self.root_nodes.contains(&node_id) {
            self.root_nodes.push(node_id);
        }
    }

    /// Get children of a node
    pub fn get_children(&self, parent_id: &str) -> Option<&Vec<String>> {
        self.parent_child_map.get(parent_id)
    }

    /// Check if a node is exportable
    pub fn is_exportable(&self, node_id: &str) -> bool {
        self.exportable_nodes.get(node_id).copied().unwrap_or(false)
    }

    /// Mark a node as exportable
    pub fn mark_exportable(&mut self, node_id: String, exportable: bool) {
        self.exportable_nodes.insert(node_id, exportable);
    }

    /// Check if a node is part of an SVG group
    pub fn is_grouped(&self, node_id: &str) -> bool {
        self.grouped_svg_nodes.contains(node_id)
    }

    /// Mark nodes as grouped
    pub fn mark_as_grouped<I>(&mut self, node_ids: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.grouped_svg_nodes.extend(node_ids);
    }

    /// Generate a unique SVG group ID
    pub fn next_svg_group_id(&mut self) -> String {
        let id = format!("svg_group_{}", self.svg_group_counter);
        self.svg_group_counter += 1;
        id
    }

    /// Get statistics about the current state
    pub fn stats(&self) -> StateStats {
        StateStats {
            total_elements: self.jsx_elements.len(),
            root_elements: self.root_nodes.len(),
            exportable_elements: self.exportable_nodes.values().filter(|&&v| v).count(),
            pending_vectors: self.pending_vectors.len(),
            grouped_nodes: self.grouped_svg_nodes.len(),
        }
    }
}

/// Statistics about visitor state
#[derive(Debug, Clone)]
pub struct StateStats {
    pub total_elements: usize,
    pub root_elements: usize,
    pub exportable_elements: usize,
    pub pending_vectors: usize,
    pub grouped_nodes: usize,
}

