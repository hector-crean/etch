pub mod transformers;

use std::path::Path;

use pulldown_cmark::{Event, Options, Parser};
use transformers::footnote::FootnoteTransformer;
use transformers::tailwind::TailwindTransformer;

use std::io;


pub trait MarkdownTransformer {
    fn transform_event<'a>(&mut self, event: Event<'a>) -> Event<'a>;
}

pub struct Etcher {
    md: String,
    options: Options,
}

impl Default for Etcher {
    fn default() -> Self {
        Self {
            md: String::new(),
            options: Options::all(),
        }
    }
}

impl Etcher {
    pub fn new<S: AsRef<str>>(md: S) -> Self {
        Self { 
            md: md.as_ref().to_string(),
            ..Default::default()
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let md = std::fs::read_to_string(path)?;
        Ok(Self::new(md))
    }

    pub fn with_option(mut self, option: Options) -> Self {
        self.options.insert(option);
        self
    }

    pub fn parse(&self, transformer: &mut impl MarkdownTransformer) -> String {
        let mut footnote_transformer = FootnoteTransformer::default();
        let parser = Parser::new_ext(&self.md, self.options)
            .map(|event| transformer.transform_event(footnote_transformer.transform_event(event)));

        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);
        html_output.push_str(&footnote_transformer.get_footnotes_html());
        html_output
    }

    pub fn parse_with_tailwind(&self) -> String {
        self.parse(&mut TailwindTransformer)
    }

    pub fn parse_with_pipeline(&self, pipeline: &mut TransformerPipeline) -> String {
        self.parse(pipeline)
    }
}

pub struct TransformerPipeline {
    transformers: Vec<Box<dyn MarkdownTransformer>>,
}

impl Default for TransformerPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformerPipeline {
    pub fn new() -> Self {
        Self {
            transformers: Vec::new(),
        }
    }

    pub fn add<T: MarkdownTransformer + 'static>(mut self, transformer: T) -> Self {
        self.transformers.push(Box::new(transformer));
        self
    }
}

impl MarkdownTransformer for TransformerPipeline {
    fn transform_event<'a>(&mut self, event: Event<'a>) -> Event<'a> {
        self.transformers.iter_mut().fold(event, |acc, transformer| {
            transformer.transform_event(acc)
        })
    }
}

