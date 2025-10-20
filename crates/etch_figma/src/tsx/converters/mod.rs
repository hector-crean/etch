use swc_ecma_ast::JSXElement;

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

/// JSX-specific conversion traits
pub trait ToJsx {
    /// Convert to JSX element in HTML context (default)
    fn to_jsx(&self) -> JSXElement {
        self.to_jsx_with_context(RenderContext::Html)
    }

    /// Convert to JSX element with specific rendering context
    fn to_jsx_with_context(&self, context: RenderContext) -> JSXElement;
}

pub mod frame;
pub mod group;
pub mod shapes;
pub mod text;
pub mod vector;

// Re-export the ToJsx trait for convenience
// pub use ToJsx; // This causes a conflict since we define the trait here
