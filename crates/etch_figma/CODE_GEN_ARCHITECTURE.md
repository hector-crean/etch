# Code Generation Architecture

This document describes the new generic code generation system that replaces the tightly-coupled TSX-specific `CodeGenExt` trait.

## Overview

The new system is built around three core concepts:

1. **Visitors** - Traverse Figma nodes and collect data
2. **Generators** - Convert visitor data into specific output formats
3. **CodeGenSystem** - Orchestrates the generation and filesystem writing

## Architecture

### Core Traits

#### `CodeGenerator<V: NodeVisitor>`
Generic trait that defines how to generate code from any visitor:

```rust
pub trait CodeGenerator<V: NodeVisitor> {
    fn generate(&self, visitor: &V) -> Result<CodeGenResult, Box<dyn std::error::Error>>;
    fn file_extension(&self) -> &str;
    fn output_directory(&self) -> &str;
}
```

#### `CodeGenResult`
Contains generated files and metadata:

```rust
pub struct CodeGenResult {
    pub files: HashMap<PathBuf, String>,
    pub metadata: HashMap<String, String>,
}
```

### Configuration

#### `CodeGenConfig`
Controls generation behavior:

```rust
pub struct CodeGenConfig {
    pub output_dir: PathBuf,
    pub separate_files: bool,
    pub exportable_only: bool,
    pub file_naming: FileNamingStrategy,
}
```

#### `FileNamingStrategy`
Controls how files are named:

```rust
pub enum FileNamingStrategy {
    ComponentName,
    NodeId,
    Custom(fn(&str, &str) -> String),
}
```

## Usage Examples

### Basic Usage

```rust
use etch_figma::{TsxVisitor, TsxGenerator, CodeGenConfig, CodeGenSystem, walker::Walker};

// 1. Create visitor and generator
let visitor = TsxVisitor::new();
let generator = TsxGenerator::new()
    .with_react_imports(true)
    .with_separate_files(true);

// 2. Configure output
let config = CodeGenConfig {
    output_dir: PathBuf::from("generated/tsx"),
    separate_files: true,
    exportable_only: false,
    file_naming: FileNamingStrategy::ComponentName,
};

// 3. Create system
let system = CodeGenSystem::new(visitor, generator, config);

// 4. Walk the Figma tree and generate
let visitor = Walker::new(visitor).walk_canvas(&canvas);
let result = system.generate_and_write()?;
```

### Multiple Formats

```rust
// Generate both TSX and HTML
let tsx_visitor = TsxVisitor::new();
let html_visitor = HtmlVisitor::new();

let tsx_generator = TsxGenerator::new();
let html_generator = HtmlGenerator::new();

// Generate TSX
let tsx_system = CodeGenSystem::new(tsx_visitor, tsx_generator, tsx_config);
let tsx_result = tsx_system.generate_and_write()?;

// Generate HTML
let html_system = CodeGenSystem::new(html_visitor, html_generator, html_config);
let html_result = html_system.generate_and_write()?;
```

## Available Generators

### TSX Generator (`TsxGenerator`)
- Generates React components
- Supports separate files or single module
- Configurable React imports
- Exportable-only filtering

### HTML Generator (`HtmlGenerator`)
- Generates HTML elements
- Supports DOCTYPE inclusion
- Configurable templates
- Exportable-only filtering

## Benefits

1. **Decoupled**: Generators are independent of visitors
2. **Extensible**: Easy to add new output formats
3. **Configurable**: Flexible generation options
4. **Filesystem-oriented**: Built-in file writing capabilities
5. **Type-safe**: Generic system with compile-time guarantees

## Migration from Old System

The old `CodeGenExt` trait methods are replaced by:

- `to_tsx_module()` → `TsxGenerator::generate()`
- `to_separate_tsx_files()` → `TsxGenerator` with `separate_files: true`
- `to_exportable_tsx_files()` → `TsxGenerator` with `exportable_only: true`

## Future Extensions

The system is designed to easily support:

- CSS generators
- JSON generators
- Markdown generators
- Custom format generators

Simply implement the `CodeGenerator<V>` trait for your visitor type.
