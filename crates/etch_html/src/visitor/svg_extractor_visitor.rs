use super::NodeVisitor;
use crate::rc_dom::{Handle, Node, NodeData};
use html5ever::{Attribute, LocalName, QualName};
use uuid::Uuid;
use std::{cell::RefCell, collections::HashMap, path::PathBuf};
use markup5ever::{namespace_url, ns};
use strum::{Display, EnumString};

//Embedding the svg

#[derive(Clone, Copy, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum SvgImportType {
    /// Renders SVG using <object> tag - best for interactive SVGs
    Object,
    /// Renders SVG using <img> tag - simplest approach, treats SVG as image
    Img,
    /// Renders SVG using <embed> tag - similar to object, but older
    Embed,
}

pub struct SvgExtractVisitor {
    svgs: HashMap<Uuid, String>,
    import_type: SvgImportType,
    asset_dir: Option<PathBuf>
}

impl SvgExtractVisitor {
    pub fn new(import_type: SvgImportType, asset_dir: Option<PathBuf>,) -> Self {
        SvgExtractVisitor {
            svgs: HashMap::new(),
            import_type,
            asset_dir
        }
    }
    pub fn svgs(&self) -> &HashMap<Uuid, String> {
        &self.svgs
    }

    fn create_replacement_element(&self, uuid: Uuid, asset_dir: Option<PathBuf>) -> Handle {
        let path = asset_dir
            .unwrap_or_else(|| PathBuf::new())
            .join(format!("{}.svg", uuid))
            .to_string_lossy()
            .into_owned();

        match self.import_type {
            SvgImportType::Object => {
                let attrs = vec![
                    Attribute {
                        name: QualName::new(None, ns!(), LocalName::from("data")),
                        value:path.into(),
                    },
                    Attribute {
                        name: QualName::new(None, ns!(), LocalName::from("type")),
                        value: "image/svg+xml".into(),
                    },
                    Attribute {
                        name: QualName::new(None, ns!(), LocalName::from("id")),
                        value: uuid.to_string().into(),
                    }
                ];
                create_element("object", attrs, vec![])
            },
            SvgImportType::Img => {
                let attrs = vec![
                    Attribute {
                        name: QualName::new(None, ns!(), LocalName::from("src")),
                        value: format!("{}.svg", uuid).into(),
                    },
                    Attribute {
                        name: QualName::new(None, ns!(), LocalName::from("id")),
                        value: uuid.to_string().into(),
                    }
                ];
                create_element("img", attrs, vec![])
            },
            SvgImportType::Embed => {
                let attrs = vec![
                    Attribute {
                        name: QualName::new(None, ns!(), LocalName::from("src")),
                        value: format!("{}.svg", uuid).into(),
                    },
                    Attribute {
                        name: QualName::new(None, ns!(), LocalName::from("type")),
                        value: "image/svg+xml".into(),
                    },
                    Attribute {
                        name: QualName::new(None, ns!(), LocalName::from("id")),
                        value: uuid.to_string().into(),
                    }
                ];
                create_element("embed", attrs, vec![])
            },
        }
    }
}

impl NodeVisitor for SvgExtractVisitor {
    
  
    fn visit_element(
        &mut self,
        element_name: &QualName,
        element_attrs: &RefCell<Vec<Attribute>>,
        template_contents: &RefCell<Option<Handle>>,
        mathml_annotation_xml_integration_point: bool,
        handle: &Handle,
    ) -> (Option<Handle>, bool) {
        let element_name = element_name.local.as_ref();
        match element_name {
            "svg" => {
                log::info!("Visiting svg element");
                
                // Generate unique ID for the SVG
                let uuid = Uuid::new_v4();
    
                // Start with opening svg tag
                let mut svg_content = String::from("<svg");
                
                // Add all original attributes
                for attr in element_attrs.borrow().iter() {
                    svg_content.push_str(&format!(" {}=\"{}\"", 
                        attr.name.local, 
                        attr.value
                    ));
                }
                
                // Close opening tag
                svg_content.push('>');
                
                // Add inner content
                svg_content.push_str(&handle.to_html_string());
                
                // Add closing tag
                svg_content.push_str("</svg>");

                self.svgs.insert(uuid, svg_content);
             
              
                
                let new_node = self.create_replacement_element(uuid, self.asset_dir.clone());
                (Some(new_node), true)
            },
            _ => (None, true)
        }
    }
}



/// Creates a new DOM element with the specified name, attributes, and children.
/// 
/// # Arguments
/// * `name` - The HTML tag name for the element
/// * `attrs` - A vector of attributes to apply to the element
/// * `children` - A vector of child nodes to attach to this element
fn create_element(name: &str, attrs: Vec<Attribute>, children: Vec<Handle>) -> Handle {
    let element = NodeData::Element {
        name: html5ever::QualName::new(
            None,
            html5ever::ns!(),
            html5ever::LocalName::from(name),
        ),
        attrs: RefCell::new(attrs),
        template_contents: RefCell::new(None),
        mathml_annotation_xml_integration_point: false,
    };
    let handle = Node::new(element);

    *handle.children.borrow_mut() = children;
    handle
}