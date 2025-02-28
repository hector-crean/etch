use super::NodeVisitor;
use crate::rc_dom::{Handle, NodeData};
use html5ever::{Attribute, QualName};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use tokio::runtime::Runtime;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    pub text: String,
}

pub struct LinkVisitor {
    links: HashMap<Uuid, Link>,
}

impl Default for LinkVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkVisitor {
    pub fn new() -> Self {
        LinkVisitor {
            links: HashMap::new(),
        }
    }

    pub fn links(&self) -> &HashMap<Uuid, Link> {
        &self.links
    }
}

impl NodeVisitor for LinkVisitor {
    fn visit_element(
        &mut self,
        name: &QualName,
        attrs: &RefCell<Vec<Attribute>>,
        template_contents: &RefCell<Option<Handle>>,
        mathml_annotation_xml_integration_point: bool,
        handle: &Handle,
    ) -> (Option<Handle>, bool) {
        if name.local.as_ref() == "a" {
            if let Some(url) = attrs.borrow().iter()
                .find(|attr| attr.name.local.as_ref() == "href")
                .map(|attr| attr.value.to_string()) 
            {
                let text = handle.children.borrow()
                    .iter()
                    .filter_map(|child| {
                        match child.data {
                            NodeData::Text { ref contents } => Some(contents.borrow().to_string()),
                            _ => None
                        }
                    })
                    .collect::<String>();

                let link = Link { url, text };
                self.links.insert(Uuid::new_v4(), link);
            }
        }
        (None, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rc_dom::RcDom;

    #[tokio::test]
    async fn test_link_visitor() {
        let html = r###"
            <div>
                <a href="https://www.rust-lang.org">Rust</a>
                <a href="https://this-is-definitely-not-a-real-website.com">Invalid</a>
                <a href="#local-anchor">Local</a>
                <a href="mailto:test@example.com">Email</a>
            </div>
        "###;

        let mut visitor = LinkVisitor::new();
        let (_, visitor) = crate::file::process_html_str(html, visitor).unwrap();

        let urls = visitor.links();
        
        // Print results for inspection
        for url in urls {
            // println!("URL: {}\nStatus: {:?}\n", url);
        }
    }
}
