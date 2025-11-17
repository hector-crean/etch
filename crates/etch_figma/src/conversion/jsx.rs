//! JSX element conversion and building
//!
//! This module provides traits and utilities for converting Figma nodes
//! to JSX (React) elements with proper structure and attributes.

use swc_ecma_ast::JSXElement;

pub mod converters;

/// Rendering context for JSX conversion
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderContext {
    /// Rendering in HTML context (can use divs, standard HTML elements)
    Html,
    /// Rendering inside an SVG group (must use SVG primitives only, no HTML elements)
    Svg {
        /// Parent x offset for positioning (used for transforms)
        parent_x: f64,
        /// Parent y offset for positioning (used for transforms)
        parent_y: f64,
    },
}

impl Default for RenderContext {
    fn default() -> Self {
        RenderContext::Html
    }
}

impl RenderContext {
    /// Check if we're in SVG context
    pub fn is_svg(&self) -> bool {
        matches!(self, RenderContext::Svg { .. })
    }

    /// Check if we're in HTML context
    pub fn is_html(&self) -> bool {
        matches!(self, RenderContext::Html)
    }
}

/// Extract position from relative_transform matrix
/// Transform matrix format: [[a, b, tx], [c, d, ty]]
/// Returns (x, y) tuple
pub fn extract_position_from_transform(
    relative_transform: &Option<Vec<Vec<f64>>>,
) -> Option<(f64, f64)> {
    if let Some(transform) = relative_transform {
        if transform.len() >= 2 && transform[0].len() >= 3 && transform[1].len() >= 3 {
            let tx = transform[0][2];
            let ty = transform[1][2];
            return Some((tx, ty));
        }
    }
    None
}

/// JSX-specific conversion trait
///
/// Implement this trait for types that can be converted to JSX elements.
/// This is the core abstraction for converting Figma nodes to React components.
pub trait ToJsx {
    /// Convert to JSX element in HTML context (default)
    fn to_jsx(&self) -> JSXElement {
        self.to_jsx_with_context(RenderContext::Html)
    }

    /// Convert to JSX element with specific rendering context
    ///
    /// The context determines whether we're rendering in HTML or SVG mode,
    /// which affects the choice of elements and attributes.
    fn to_jsx_with_context(&self, context: RenderContext) -> JSXElement;
}
