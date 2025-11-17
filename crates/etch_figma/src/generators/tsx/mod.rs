//! TSX (React/TypeScript) code generation
//!
//! This module generates React components with TypeScript from Figma designs.
//! It produces clean, production-ready TSX with Tailwind CSS styling.
//!
//! ## Module Organization
//!
//! - `generator` - Main TSX generator that emits code
//! - `visitor` - Visitor that walks Figma tree and converts to JSX
//! - `builder` - Builder pattern for configuring generators
//! - `state` - State management for visitor
//! - `jsx_builder` - Utilities for building JSX hierarchies
//! - `svg_handler` - SVG conversion and optimization
//! - `style_generator` - Style and positioning strategy generation

pub mod builder;
pub mod generator;
pub mod jsx_builder;
pub mod state;
pub mod style_generator;
pub mod svg_handler;
pub mod visitor;

// Re-export main types
pub use builder::TsxGeneratorBuilder;
pub use generator::TsxGenerator;
pub use jsx_builder::JsxHierarchyBuilder;
pub use state::{StateStats, VisitorState};
pub use style_generator::{PositioningStrategy, StyleGenerator};
pub use svg_handler::SvgHandler;
pub use visitor::TsxVisitor;
