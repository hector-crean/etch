use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};

use crate::MarkdownTransformer;


pub struct TailwindTransformer;

impl MarkdownTransformer for TailwindTransformer {
    fn transform_event<'a>(&mut self, event: Event<'a>) -> Event<'a> {
        match event {
            Event::Start(Tag::Paragraph) => {
                Event::Html("<p class=\"text-gray-700 leading-relaxed mb-4\">".into())
            },
            Event::End(TagEnd::Paragraph) => {
                Event::Html("</p>".into())
            },
            Event::Start(Tag::Heading { level, .. }) => {
                let class = match level {
                    HeadingLevel::H1 => "text-4xl font-bold mb-6",
                    HeadingLevel::H2 => "text-3xl font-semibold mb-4",
                    HeadingLevel::H3 => "text-2xl font-medium mb-3",
                    _ => "text-xl font-medium mb-2"
                };
                Event::Html(format!("<{} class=\"{}\">", level, class).into())
            },
            Event::End(TagEnd::Heading(level)) => {
                Event::Html(format!("</{}>", level).into())
            },
            _ => event,
        }
    }
}