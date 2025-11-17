
## Current Pain Points

1. **Mixed concerns in `tsx/visitor.rs`** (1912 lines!) - Does too many things
2. **Unclear module boundaries** - `svg/`, `tsx/svgr/`, and `analyzers/` overlap
3. **Hidden complexity** - SVG processing pipeline is fragmented
4. **No clear entry point** - Multiple ways to use the library
5. **Confusing naming** - "svgr" module isn't self-explanatory

## Proposed Restructure

### 1. **Clearer Module Hierarchy**

```
crates/etch_figma/
├── src/
│   ├── lib.rs                          # Clear API surface
│   │
│   ├── core/                           # Core abstractions
│   │   ├── mod.rs
│   │   ├── walker.rs                   # Tree traversal
│   │   ├── visitor.rs                  # Visitor trait
│   │   ├── context.rs                  # NodeContext
│   │   └── config.rs                   # CodeGenConfig
│   │
│   ├── analysis/                       # Analysis phase (read-only)
│   │   ├── mod.rs
│   │   ├── layout.rs                   # Layout analysis
│   │   ├── text.rs                     # Text analysis
│   │   ├── svg_container.rs            # SVG decisions
│   │   └── responsive.rs               # Responsive analysis
│   │
│   ├── conversion/                     # Conversion phase (Figma → intermediate)
│   │   ├── mod.rs
│   │   ├── styles.rs                   # Tailwind generation
│   │   ├── jsx.rs                      # JSX building
│   │   └── svg/                        # SVG-specific conversion
│   │       ├── mod.rs
│   │       ├── inline.rs               # Inline SVG generation
│   │       ├── exporter.rs             # API export
│   │       ├── grouping.rs             # SVG optimization
│   │       └── path_registry.rs        # Path extraction
│   │
│   ├── generators/                     # Code generation (intermediate → output)
│   │   ├── mod.rs
│   │   ├── tsx/
│   │   │   ├── mod.rs
│   │   │   ├── generator.rs
│   │   │   └── visitor.rs              # TSX-specific visitor
│   │   └── html/
│   │       ├── mod.rs
│   │       ├── generator.rs
│   │       └── visitor.rs
│   │
│   ├── pipeline/                       # Orchestration
│   │   ├── mod.rs
│   │   ├── unified.rs                  # UnifiedPipeline
│   │   └── builder.rs                  # Builder pattern
│   │
│   └── extensions/                     # Optional extensions
│       ├── mod.rs
│       ├── tailwind.rs                 # Tailwind helpers
│       └── figma_api.rs                # API integration
```

### 2. **Split the Giant TsxVisitor**

Currently `tsx/visitor.rs` is 1912 lines. Split into:

```rust
// generators/tsx/visitor.rs - Core visitor (~400 lines)
pub struct TsxVisitor {
    state: VisitorState,
    converter: JsxConverter,
    svg_handler: SvgHandler,
    style_generator: StyleGenerator,
}

// generators/tsx/state.rs - State management (~300 lines)
pub struct VisitorState {
    jsx_elements: HashMap<String, JSXElement>,
    parent_child_map: HashMap<String, Vec<String>>,
    root_nodes: Vec<String>,
    // ... etc
}

// generators/tsx/jsx_converter.rs - JSX conversion (~400 lines)
pub struct JsxConverter {
    // Handles Frame → div, Text → p, etc.
}

// generators/tsx/svg_handler.rs - SVG coordination (~300 lines)
pub struct SvgHandler {
    exporter: FigmaSvgExporter,
    grouping_manager: SvgGroupingManager,
    pending_vectors: HashMap<String, VectorNode>,
}

// generators/tsx/style_generator.rs - Style generation (~500 lines)
pub struct StyleGenerator {
    // Tailwind class generation
}
```

### 3. **Clearer API Entry Points**

```rust
// lib.rs - Simple, clear public API
pub mod core;
pub mod analysis;
pub mod conversion;
pub mod generators;
pub mod pipeline;

// Primary API - Simple cases
pub use pipeline::{Pipeline, PipelineBuilder};

// Advanced API - Custom workflows
pub use core::{Walker, NodeVisitor, NodeContext};
pub use generators::tsx::{TsxGenerator, TsxVisitor};
pub use generators::html::{HtmlGenerator, HtmlVisitor};

// Quick start function
pub fn figma_to_tsx(canvas: &CanvasNode, config: Config) -> Result<CodeGenResult> {
    PipelineBuilder::tsx()
        .with_config(config)
        .build()
        .process(canvas)
}
```

### 4. **Better Documentation Structure**

```
docs/
├── README.md                    # Overview + quick start
├── ARCHITECTURE.md              # The doc I just wrote
├── GUIDE_BASIC.md              # Basic usage guide
├── GUIDE_ADVANCED.md           # Custom visitors, generators
├── GUIDE_SVG.md                # SVG handling deep-dive
└── CONTRIBUTING.md             # Development guide
```

### 5. **Clearer Phase Separation**

```rust
// Make the phases explicit in code
pub struct ConversionPipeline {
    // Phase 1: Analysis
    analysis_phase: AnalysisPhase,
    
    // Phase 2: Conversion
    conversion_phase: ConversionPhase,
    
    // Phase 3: Generation
    generation_phase: GenerationPhase,
    
    // Phase 4: Post-processing
    postprocess_phase: PostProcessPhase,
}

impl ConversionPipeline {
    pub async fn process(&mut self, canvas: &CanvasNode) -> Result<Output> {
        let analysis = self.analysis_phase.analyze(canvas)?;
        let intermediate = self.conversion_phase.convert(canvas, analysis)?;
        let code = self.generation_phase.generate(intermediate)?;
        let final_code = self.postprocess_phase.process(code)?;
        Ok(final_code)
    }
}
```

### 6. **Rename Confusing Modules**

- `tsx/svgr/` → `conversion/svg/` (svgr is not intuitive)
- `codegen_ext.rs` → `core/codegen.rs` (ext suggests extension)
- `tailwind_ext.rs` → `conversion/styles.rs` (more descriptive)

### 7. **Add Comprehensive Examples**

```rust
// examples/basic.rs
// Simplest possible usage

// examples/custom_visitor.rs
// How to create your own visitor

// examples/svg_strategies.rs
// Different SVG handling approaches

// examples/post_processing.rs
// Adding custom post-processors

// examples/multiple_formats.rs
// Generating TSX + HTML simultaneously
```

### 8. **Better Type Names**

```rust
// Current - unclear
pub struct AsyncSvgResult { ... }
pub struct ProcessingStats { ... }

// Better - self-documenting
pub struct SvgExportResult { ... }
pub struct SvgExportStatistics { ... }

// Current - confusing
pub struct GroupedSvgElement { ... }

// Better
pub struct OptimizedSvgGroup { ... }
```

### 9. **Consistent Builder Pattern**

```rust
// Make all builders consistent
pub struct TsxGeneratorBuilder { ... }
pub struct HtmlGeneratorBuilder { ... }
pub struct PipelineBuilder { ... }

// All follow same pattern
let generator = TsxGeneratorBuilder::new()
    .with_react_imports(true)
    .with_separate_files(true)
    .build();
```

### 10. **Add Feature Flags**

```toml
[features]
default = ["tsx", "tailwind"]
tsx = []
html = []
svg-api = ["tokio", "reqwest"]  # Async SVG export
svg-inline = []                  # Inline SVG only
tailwind = []
analyzers = []
full = ["tsx", "html", "svg-api", "svg-inline", "tailwind", "analyzers"]
```

## Implementation Plan

If you want to proceed with this restructure, here's a safe migration path:

### Phase 1: Documentation (No code changes)
1. Write the improved docs
2. Add inline code documentation
3. Create examples

### Phase 2: Module reorganization (Safe refactor)
1. Move files to new structure
2. Update imports
3. Keep old module paths with deprecation warnings

### Phase 3: Split large files (Breaking but clear)
1. Split `TsxVisitor` into multiple files
2. Extract concerns into separate modules
3. Update public API

### Phase 4: API cleanup (Breaking changes)
1. Simplify public API
2. Add builder patterns
3. Remove confusing names

### Phase 5: Polish
1. Add feature flags
2. Improve error messages
3. Add integration tests

