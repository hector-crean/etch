# Etch Figma: Architecture & Design Document

## Overview

`etch_figma` is a Rust-based system that converts Figma design files into React TSX components. It uses a **visitor pattern** to traverse the Figma scene hierarchy and generates production-ready React code with Tailwind CSS styling.

## Core Architecture

The system is built around three fundamental concepts:

1. **Walker** - Generic tree traversal using the visitor pattern
2. **Visitors** - Collect and transform data from Figma nodes
3. **Generators** - Convert visitor data into output files (TSX, HTML, etc.)

### High-Level Flow

```
Figma JSON → Walker + Visitor → Generator → TSX Files
                ↓
        UnifiedPipeline (orchestration + post-processing)
```

---

## 1. The Walker (`core/walker.rs`)

The **Walker** is a generic tree traversal engine that implements the classic visitor pattern for Figma's hierarchical node structure.

### Key Components

#### `NodeVisitor` Trait
```rust
pub trait NodeVisitor {
    fn visit_frame(&mut self, frame: &FrameNode, context: &NodeContext) {}
    fn visit_text(&mut self, text: &TextNode, context: &NodeContext) {}
    fn visit_vector(&mut self, vector: &VectorNode, context: &NodeContext) {}
    // ... methods for all 25+ Figma node types
    
    fn enter_container(&mut self, node: &SubcanvasNode, context: &NodeContext) {}
    fn exit_container(&mut self, node: &SubcanvasNode, context: &NodeContext) {}
    fn should_traverse_children(&self, node: &SubcanvasNode) -> bool { true }
}
```

All methods have **default no-op implementations**, so you only override what you need.

#### `NodeContext`
Provides rich contextual information during traversal:
- **path**: Hierarchical path from root (e.g., `["Canvas", "Frame", "Group"]`)
- **depth**: Current nesting level
- **node_id**: ID of the current node

#### `Walker<V: NodeVisitor>`
The walker orchestrates the traversal:
```rust
let visitor = TsxVisitor::new();
let walker = Walker::new(visitor);
let completed_visitor = walker.walk_canvas(&canvas);
// Now use completed_visitor to generate code
```

**Traversal order**: Depth-first, pre-order (parent visited before children)

---

## 2. The TSX Visitor (`generators/tsx/visitor.rs`)

**TsxVisitor** is the heart of the TSX generation system. It implements `NodeVisitor` and converts Figma nodes into JSX elements (using SWC's AST).

### Internal Components

The visitor is composed of several specialized components:

1. **VisitorState** - Manages all the maps and tracking data
2. **JsxConverter** - Converts Figma nodes to JSX elements
3. **SvgHandler** - Coordinates SVG export and optimization
4. **StyleGenerator** - Generates Tailwind CSS classes

### Key Responsibilities

#### 1. **Node-to-JSX Conversion**
Each Figma node type is converted to appropriate JSX:
- **Frame** → `<div>` with layout classes
- **Text** → `<div>` or `<p>` with typography
- **Vector/Shape** → SVG elements or API-exported SVG
- **Group** → wrapper `<div>` or SVG `<g>`

#### 2. **Hierarchy Building**
The visitor maintains parent-child relationships:
- `enter_container()` - Push node to parent stack
- `visit_*()` - Create JSX for current node
- `exit_container()` - Pop stack, insert children into parent JSX

#### 3. **Style Generation**
Uses **Tailwind CSS** as the primary styling system:
- Converts Figma properties to Tailwind classes
- Falls back to inline styles for unsupported properties

#### 4. **SVG Strategy**
The visitor handles vectors through multiple strategies:
- **Inline Conversion**: Simple shapes like rectangles, ellipses
- **API Export**: Complex vectors via Figma's export API
- **Hybrid Approach**: Smart selection based on complexity

---

## 3. SVG Processing Pipeline

### Components

1. **FigmaSvgExporter** (`conversion/svg/inline.rs`) - Inline SVG generation
2. **AsyncSvgProcessor** (`conversion/svg/exporter.rs`) - API-based SVG export
3. **SvgGroupingManager** (`conversion/svg/grouping.rs`) - SVG optimization
4. **PathRegistry** (`conversion/svg/path_registry.rs`) - Path extraction

### Path Extraction

Complex SVG paths are extracted to a separate TypeScript file:

**svg-paths.ts**:
```typescript
export default {
  p10978a80: "M40 65C40.5523 65 41 64.3284...",
  p11365cc0: "M38.1866 44.6791H42.4679C42.9903...",
}
```

**Usage**:
```tsx
import svgPaths from './svg-paths';
<path d={svgPaths.p10978a80} fill="#000" />
```

---

## 4. The TSX Generator (`generators/tsx/generator.rs`)

The **TsxGenerator** takes a completed `TsxVisitor` and produces React component files.

### Output Strategies

#### Separate Files (default)
One file per component with proper exports:
```tsx
import React from 'react';

export function ComponentName() {
  return <div>...</div>;
}

export default ComponentName;
```

#### Single Module
All components in one file with named exports.

---

## 5. The Unified Pipeline (`pipeline/unified.rs`)

**UnifiedPipeline** orchestrates the entire process:

### Process Flow

1. **Generate TSX from Figma**
   - Create TsxVisitor with config
   - Walk canvas with Walker
   - Process pending vectors (async API calls)
   - Run SVG grouping optimization

2. **Post-Process TSX**
   - Parse generated TSX with `etch_tsx`
   - Apply visitors (SVG fixes, UUID injection, etc.)
   - Re-emit code with modifications

3. **Return Result**
   - Files ready to write to disk

### Builder Pattern

```rust
let pipeline = UnifiedPipelineBuilder::new()
    .with_figma_config(config)
    .with_file_key("figma_file_key")
    .with_figma_svg_processing()
    .with_uuid_injection()
    .build();

let result = pipeline.process_canvas(&canvas).await?;
```

---

## 6. Analysis Phase (`analysis/`)

Before conversion, various analyzers examine the Figma data:

- **LayoutAnalyzer** - Auto-layout → Flexbox mapping
- **TextAnalyzer** - Text complexity and formatting
- **SvgContainerAnalyzer** - SVG vs HTML container decisions
- **ResponsiveAnalyzer** - Responsive design patterns

These inform conversion decisions without mixing concerns.

---

## 7. Configuration System

### CodeGenConfig

Comprehensive configuration for all aspects:

```rust
pub struct CodeGenConfig {
    // Output
    pub output_dir: PathBuf,
    pub separate_files: bool,
    pub exportable_only: bool,
    
    // SVG Strategy
    pub vector_export_strategy: VectorExportStrategy,
    pub svg_container_mode: SvgContainerMode,
    pub extract_svg_paths: bool,
    
    // Styling
    pub use_css_variables: bool,
    
    // Text
    pub text_in_svg_mode: TextInSvgMode,
    pub detect_superscripts: bool,
    
    // Responsive
    pub enable_container_queries: bool,
    pub responsive_breakpoint_px: f64,
}
```

---

## 8. Design Principles

### 1. Separation of Concerns
- **Core**: Tree traversal (walker, visitor trait)
- **Analysis**: Read-only inspection of Figma data
- **Conversion**: Transform Figma → intermediate representation
- **Generation**: Intermediate → output files
- **Pipeline**: Orchestration

### 2. Extensibility
- New output formats: Implement `CodeGenerator<V>`
- New analyses: Add analyzers
- New transformations: Add visitors

### 3. Type Safety
- Full Rust type system
- SWC AST for correct JSX
- Compile-time guarantees

### 4. Performance
- Concurrent API requests
- SVG optimization
- Efficient tree traversal

---

## 9. Usage Examples

### Basic Usage

```rust
use etch_figma::{Pipeline, Config};

let config = Config::default();
let result = Pipeline::tsx()
    .with_config(config)
    .process(&canvas)?;
```

### Advanced: Custom Visitor

```rust
use etch_figma::core::{Walker, NodeVisitor};

struct MyVisitor { /* ... */ }

impl NodeVisitor for MyVisitor {
    fn visit_frame(&mut self, frame: &FrameNode, ctx: &NodeContext) {
        // Custom logic
    }
}

let visitor = MyVisitor::new();
let walker = Walker::new(visitor);
let completed = walker.walk_canvas(&canvas);
```

### Async SVG Export

```rust
let pipeline = UnifiedPipelineBuilder::new()
    .with_file_key("figma_file_key")
    .with_figma_config(config)
    .build();

let result = pipeline.process_canvas(&canvas).await?;
```

---

## 10. Module Structure

```
etch_figma/
├── core/           # Core abstractions (walker, visitor trait, config)
├── analysis/       # Pre-conversion analysis
├── conversion/     # Figma → intermediate representation
│   └── svg/        # SVG-specific conversion
├── generators/     # Intermediate → output files
│   ├── tsx/        # TSX generation
│   └── html/       # HTML generation
├── pipeline/       # Orchestration and builder
└── extensions/     # Optional utilities
```

---

## 11. Contributing

When adding new features:

1. **New node type support**: Update visitor trait + implementations
2. **New output format**: Implement `CodeGenerator<V>`
3. **New analysis**: Add to `analysis/` module
4. **New optimization**: Extend pipeline post-processing

---

## 12. Testing Strategy

- **Unit tests**: Each module independently
- **Integration tests**: Full pipeline end-to-end
- **Snapshot tests**: Generated code output
- **Property tests**: Invariants (valid JSX, correct hierarchy)

---

## Further Reading

- [Basic Usage Guide](./docs/GUIDE_BASIC.md)
- [Advanced Guide](./docs/GUIDE_ADVANCED.md)
- [SVG Handling](./docs/GUIDE_SVG.md)
- [Contributing](./CONTRIBUTING.md)

