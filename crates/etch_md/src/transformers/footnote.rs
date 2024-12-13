use std::collections::HashMap;

use pulldown_cmark::{Event, Tag, TagEnd};

use crate::MarkdownTransformer;


#[derive(Default)]
pub struct FootnoteTransformer {
    collected_footnotes: Vec<Vec<Event<'static>>>,
    current_footnote_buffer: Vec<Vec<Event<'static>>>,
    footnote_reference_map: HashMap<String, (usize, usize)>,
}


impl MarkdownTransformer for FootnoteTransformer {
    fn transform_event<'event>(&mut self, event: Event<'event>) -> Event<'event> {
        match event {
            Event::Start(Tag::FootnoteDefinition(_)) => {
                self.current_footnote_buffer.push(vec![event.into_static()]);
                Event::Text("".into())
            }
            Event::End(TagEnd::FootnoteDefinition) => {
                let mut footnote_content = self.current_footnote_buffer.pop().unwrap();
                footnote_content.push(event.into_static());
                self.collected_footnotes.push(footnote_content);
                Event::Text("".into())
            }
            Event::FootnoteReference(name) => {
                let next_reference_number = self.footnote_reference_map.len() + 1;
                let (reference_number, occurrence_count) = self.footnote_reference_map.entry(name.to_string()).or_insert((next_reference_number, 0usize));
                *occurrence_count += 1;
                Event::Html(
                    format!(r##"<sup class="footnote-reference" id="fr-{}-{}"><a href="#fn-{}">[{}]</a></sup>"##, name, name, name, reference_number).into()
                )
            }
            _ if !self.current_footnote_buffer.is_empty() => {
                self.current_footnote_buffer.last_mut().unwrap().push(event.into_static());
                Event::Text("".into())
            }
            _ => event,
        }
    }
}

impl FootnoteTransformer {
    pub fn get_footnotes_html(&self) -> String {
        if self.collected_footnotes.is_empty() {
            return String::new();
        }

        let mut footnotes = self.collected_footnotes.clone();
        footnotes.retain(|f| match f.first() {
            Some(Event::Start(Tag::FootnoteDefinition(name))) => {
                self.footnote_reference_map.get(&name.to_string()).unwrap_or(&(0, 0)).1 != 0
            }
            _ => false,
        });
        footnotes.sort_by_cached_key(|f| match f.first() {
            Some(Event::Start(Tag::FootnoteDefinition(name))) => {
                self.footnote_reference_map.get(&name.to_string()).unwrap_or(&(0, 0)).0
            }
            _ => unreachable!(),
        });

        let mut html_output = String::from("<hr><ol class=\"footnotes-list\">\n");
        let footnote_events = footnotes.into_iter().flatten();
        pulldown_cmark::html::push_html(&mut html_output, footnote_events);
        html_output.push_str("</ol>\n");
        html_output
    }
}

