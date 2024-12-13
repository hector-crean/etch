use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;

use crate::MarkdownTransformer;

pub struct RustTransformer {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    in_rust_block: bool,
}

impl Default for RustTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl RustTransformer {
    pub fn new() -> Self {
        RustTransformer {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            in_rust_block: false,
        }
    }
}

impl MarkdownTransformer for RustTransformer {
    fn transform_event<'a>(&mut self, event: Event<'a>) -> Event<'a> {
        match &event {
            Event::Start(Tag::CodeBlock(code_block_kind)) => {
                match code_block_kind {
                    CodeBlockKind::Fenced(lang) => {
                        self.in_rust_block = matches!(lang.as_ref(), "rust");
                    }
                    CodeBlockKind::Indented => {
                        self.in_rust_block = true;
                    }
                }
                event
            },
            Event::Text(code) if self.in_rust_block => {
                let syntax = self.syntax_set.find_syntax_by_extension("rs").unwrap();
                let theme = &self.theme_set.themes["base16-ocean.dark"];
                let highlighted = highlighted_html_for_string(
                    code,
                    &self.syntax_set,
                    syntax,
                    theme
                ).unwrap();
                Event::Html(highlighted.into())
            },
            Event::End(TagEnd::CodeBlock) => {
                self.in_rust_block = false;
                event
            },
            _ => event,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Event;

    #[test]
    fn test_rust_code_highlighting() {
        // Create transformer instance
        let mut transformer = RustTransformer::new();

        // Sample Rust code
        let rust_code = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let input_event = Event::Code(rust_code.into());

        // Transform the event
        let output_event = transformer.transform_event(input_event);

        // Verify the output is HTML and contains syntax highlighting classes
        match output_event {
            Event::Html(html) => {
                let html_str = html.to_string();
                assert!(html_str.contains("<span")); // Should contain syntax highlighting spans
                assert!(html_str.contains("class=")); // Should contain CSS classes
                assert!(html_str.contains("println!")); // Should contain the original code
            },
            _ => panic!("Expected HTML output"),
        }
    }
}