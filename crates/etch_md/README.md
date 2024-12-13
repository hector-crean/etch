# Etcher

A flexible Markdown transformation library built on top of pulldown-cmark, providing easy ways to customize markdown rendering with features like footnotes, Tailwind CSS styling, and syntax highlighting.

## Features

- 📝 Footnote support
- ✨ Rust code syntax highlighting
- 🔄 Extensible transformer pipeline
- 🎯 Simple API

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
etcher = "0.1.0"
```

## Basic Usage
```rust
use etcher::Etcher;
fn main() {
    // Create a new Etcher instance with some markdown
    let markdown = "# Hello World\n\nThis is a test with some bold text.";
    let etcher = Etcher::new(markdown);
    // Parse with default Tailwind styling
    let html = etcher.parse_with_tailwind();
    println!("{}", html);
}
```

## Combining Multiple Transformers

```rust
use etcher::{ 
    Etcher, TransformerPipeline,
    transformers::{
        footnote::FootnoteTransformer,
        tailwind::TailwindTransformer,
        rust::RustTransformer,
    }
};

fn main() {

    let pipleline = TransformerPipeline::new()
            .add(RustTransformer::new())
            .add(FootnoteTransformer::default())
            .add(TailwindTransformer);

    let html = etcher.parse_with_pipeline(&mut pipeline);
}

```


The transforms are often not commutative, so the order of the transformers in the pipeline is important.