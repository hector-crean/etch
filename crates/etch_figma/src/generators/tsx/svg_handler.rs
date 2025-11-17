//! SVG handling and coordination for TSX generation
//!
//! This module handles all SVG-related operations during TSX generation:
//! - SVG export coordination
//! - Grouping optimization
//! - SVG-to-JSX conversion

use figma_api::models::VectorNode;
use std::collections::HashMap;
use swc_atoms::Atom;
use swc_common::DUMMY_SP;
use swc_ecma_ast::{
    IdentName, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXClosingElement, JSXElement,
    JSXElementChild, JSXElementName, JSXOpeningElement, JSXText, Lit, Str,
};

use crate::conversion::svg::{
    FigmaSvgExporter, GroupedSvgElement, SvgConfig, SvgGroupingManager, VectorExportConfig,
};

/// Handler for SVG-related operations during TSX conversion
pub struct SvgHandler {
    /// SVG exporter for vector content
    pub svg_exporter: FigmaSvgExporter,

    /// SVG grouping manager for optimizing SVG containers
    pub svg_grouping_manager: SvgGroupingManager,

    /// Current Figma file key for SVG exports
    pub file_key: Option<String>,

    /// Vector export configuration
    pub vector_export_config: Option<VectorExportConfig>,
}

impl SvgHandler {
    /// Create a new SVG handler with default configuration
    pub fn new() -> Self {
        Self {
            svg_exporter: FigmaSvgExporter::new(SvgConfig::default()),
            svg_grouping_manager: SvgGroupingManager::new(),
            file_key: None,
            vector_export_config: None,
        }
    }

    /// Create a new SVG handler with custom SVG configuration
    pub fn with_svg_config(config: SvgConfig) -> Self {
        Self {
            svg_exporter: FigmaSvgExporter::new(config),
            svg_grouping_manager: SvgGroupingManager::new(),
            file_key: None,
            vector_export_config: None,
        }
    }

    /// Set the Figma file key for SVG exports
    pub fn set_file_key(&mut self, file_key: String) {
        self.file_key = Some(file_key);
    }

    /// Get a mutable reference to the SVG exporter
    pub fn get_svg_exporter_mut(&mut self) -> &mut FigmaSvgExporter {
        &mut self.svg_exporter
    }

    /// Set a custom SVG exporter
    pub fn set_svg_exporter(&mut self, svg_exporter: FigmaSvgExporter) {
        self.svg_exporter = svg_exporter;
    }

    /// Get the SVG grouping manager
    pub fn get_grouping_manager(&self) -> &SvgGroupingManager {
        &self.svg_grouping_manager
    }

    /// Process SVG groups to create optimized grouped elements
    ///
    /// Note: This is a placeholder. The actual grouping logic needs to be implemented
    /// based on the SvgGroupingManager's API.
    pub fn process_svg_groups(
        &mut self,
        _pending_vectors: &HashMap<String, VectorNode>,
        _jsx_elements: &HashMap<String, JSXElement>,
    ) -> Vec<GroupedSvgElement> {
        // TODO: Implement proper SVG grouping
        Vec::new()
    }

    /// Create JSX element from SVG content string
    ///
    /// Parses SVG markup and converts it to a JSX element with proper structure.
    pub fn create_jsx_from_svg_content(&self, svg_content: &str) -> JSXElement {
        // Parse the SVG content to extract attributes and children
        // For now, create a basic SVG wrapper element
        // TODO: Implement proper SVG parsing to extract all attributes and children

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                name: JSXElementName::Ident(
                    IdentName {
                        span: DUMMY_SP,
                        sym: Atom::from("svg"),
                    }
                    .into(),
                ),
                span: DUMMY_SP,
                attrs: vec![],
                self_closing: false,
                type_args: None,
            },
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    IdentName {
                        span: DUMMY_SP,
                        sym: Atom::from("svg"),
                    }
                    .into(),
                ),
            }),
            children: vec![
                // Include the raw SVG content as a child
                JSXElementChild::JSXText(JSXText {
                    span: DUMMY_SP,
                    value: Atom::from(svg_content),
                    raw: Atom::from(svg_content),
                }),
            ],
        }
    }

    /// Create a placeholder JSX element for a vector that's awaiting SVG data
    pub fn create_vector_placeholder(&self, vector: &VectorNode) -> JSXElement {
        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                name: JSXElementName::Ident(
                    IdentName {
                        span: DUMMY_SP,
                        sym: Atom::from("div"),
                    }
                    .into(),
                ),
                span: DUMMY_SP,
                attrs: vec![
                    // Add a data attribute to mark it as a placeholder
                    JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(IdentName {
                            span: DUMMY_SP,
                            sym: Atom::from("data-vector-id"),
                        }),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: Atom::from(vector.id.as_str()),
                            raw: None,
                        }))),
                    }),
                ],
                self_closing: true,
                type_args: None,
            },
            closing: None,
            children: vec![],
        }
    }

    /// Check if a vector should be exported via API
    pub fn should_export_vector(&self, _vector: &VectorNode) -> bool {
        // Determine if this vector is complex enough to need API export
        // vs. being rendered as inline SVG
        // For now, default to true (use API export)
        true
    }
}

impl Default for SvgHandler {
    fn default() -> Self {
        Self::new()
    }
}
