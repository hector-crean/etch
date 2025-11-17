use crate::conversion::svg::path_registry::PathRegistry;
use figma_api::models::VectorNode;
use std::collections::HashMap;
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;

/// Utility functions for percentage-based positioning
pub struct PositioningUtils;

impl PositioningUtils {
    /// Calculate percentage-based inset values from Figma coordinates
    /// Returns Tailwind CSS inset class like "inset-[12.5%_20.83%_20.83%_12.5%]"
    pub fn calculate_inset_percentages(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        parent_width: f64,
        parent_height: f64,
    ) -> String {
        let top_percent = (y / parent_height) * 100.0;
        let right_percent = ((parent_width - (x + width)) / parent_width) * 100.0;
        let bottom_percent = ((parent_height - (y + height)) / parent_height) * 100.0;
        let left_percent = (x / parent_width) * 100.0;

        format!(
            "inset-[{:.2}%_{:.2}%_{:.2}%_{:.2}%]",
            top_percent, right_percent, bottom_percent, left_percent
        )
    }

    /// Calculate inner inset for nested SVG elements (like Figma MCP's inner div)
    /// This creates the inner positioning div with negative inset for overflow
    pub fn calculate_inner_inset(
        width: f64,
        height: f64,
        parent_width: f64,
        parent_height: f64,
    ) -> String {
        // Calculate how much the element overflows its container
        let width_overflow = ((width - parent_width) / parent_width) * 100.0;
        let height_overflow = ((height - parent_height) / parent_height) * 100.0;

        // Use negative inset to allow overflow (like Figma MCP)
        let inset_value = if width_overflow > 0.0 || height_overflow > 0.0 {
            format!("inset-[{:.2}%]", -width_overflow.max(height_overflow))
        } else {
            "inset-0".to_string()
        };

        inset_value
    }

    /// Determine if an element should use percentage-based positioning
    pub fn should_use_percentage_positioning(
        parent_has_auto_layout: bool,
        element_size: (f64, f64),
        parent_size: (f64, f64),
    ) -> bool {
        // Use percentage positioning for non-auto-layout elements
        // or when element is significantly smaller than parent (scaling scenario)
        !parent_has_auto_layout
            || (element_size.0 < parent_size.0 * 0.8 && element_size.1 < parent_size.1 * 0.8)
    }
}

/// Represents a grouped SVG element with positioning information
#[derive(Debug, Clone)]
pub struct GroupedSvgElement {
    pub node_id: String,
    pub vector_node: VectorNode,
    pub svg_content: String,
    pub position: (f64, f64),
    pub size: (f64, f64),
}

/// Groups SVG elements by their parent container in the Figma hierarchy
pub struct SvgGrouper {
    /// SVG elements that need to be grouped (all siblings under the same parent)
    svg_elements: Vec<GroupedSvgElement>,
    /// Parent container ID in the Figma hierarchy
    parent_id: String,
    /// Registry for managing SVG path data extraction
    path_registry: PathRegistry,
}

impl SvgGrouper {
    pub fn new(parent_id: String) -> Self {
        Self {
            svg_elements: Vec::new(),
            parent_id,
            path_registry: PathRegistry::new(),
        }
    }

    /// Add an SVG element to be grouped
    pub fn add_svg_element(&mut self, element: GroupedSvgElement) {
        self.svg_elements.push(element);
    }

    /// Create positioned div wrappers for all SVG elements
    pub fn create_positioned_svg_elements(&mut self) -> Vec<JSXElement> {
        if self.svg_elements.is_empty() {
            return Vec::new();
        }

        // For now, create individual positioned wrappers for each element
        // TODO: Implement proper grouping with path registry integration
        self.svg_elements
            .iter()
            .map(|element| self.create_positioned_svg_wrapper(element))
            .collect()
    }

    /// Group SVG elements by their semantic meaning (icons, backgrounds, etc.)
    fn group_svg_elements_by_semantics(&self) -> Vec<Vec<&GroupedSvgElement>> {
        let mut groups = Vec::new();
        let mut remaining = self.svg_elements.iter().collect::<Vec<_>>();

        while !remaining.is_empty() {
            let mut current_group = Vec::new();
            let first_element = remaining.remove(0);
            current_group.push(first_element);

            // Group elements that are likely part of the same semantic unit
            // (e.g., multiple parts of the same icon, or related SVG elements)
            remaining.retain(|element| {
                if self.are_semantically_related(first_element, element) {
                    current_group.push(*element);
                    false
                } else {
                    true
                }
            });

            groups.push(current_group);
        }

        groups
    }

    /// Check if two SVG elements are semantically related (part of the same component)
    fn are_semantically_related(
        &self,
        elem1: &GroupedSvgElement,
        elem2: &GroupedSvgElement,
    ) -> bool {
        // Check if they're close in position (likely part of the same icon/component)
        let (x1, y1) = elem1.position;
        let (x2, y2) = elem2.position;
        let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();

        // If they're within 100px of each other, they're likely related
        distance < 100.0
    }

    /// Create a semantic SVG group (like an icon component)
    fn create_semantic_svg_group(&mut self, elements: Vec<&GroupedSvgElement>) -> JSXElement {
        if elements.len() == 1 {
            // Single element - create a simple positioned wrapper with percentage-based positioning
            return self.create_positioned_svg_wrapper(elements[0]);
        }

        // For multiple elements, create a single SVG with <g> elements
        // This is more efficient and follows SVG best practices
        let svg_element = self.create_grouped_svg_with_g_elements(elements);

        // Wrap the SVG in a positioned container div for uniformity
        self.create_svg_container_wrapper(svg_element)
    }

    /// Create a container wrapper for SVG elements
    fn create_svg_container_wrapper(&self, svg_element: JSXElement) -> JSXElement {
        let mut container_attrs = Vec::new();
        container_attrs.push(self.create_attr("className", "relative"));
        container_attrs.push(self.create_attr("data-name", "svg-container"));
        container_attrs.push(self.create_attr("data-node-id", &self.parent_id));

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
                type_args: None,
                attrs: container_attrs,
                self_closing: false,
            },
            children: vec![JSXElementChild::JSXElement(Box::new(svg_element))],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
            }),
        }
    }

    /// Create a flexbox layout for linear SVG arrangements
    fn create_flex_layout(&self, elements: &[&GroupedSvgElement]) -> JSXElement {
        // Calculate container bounds
        let (min_x, min_y, max_x, max_y) = elements.iter().fold(
            (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ),
            |(min_x, min_y, max_x, max_y), element| {
                let (x, y) = element.position;
                let (width, height) = element.size;
                (
                    min_x.min(x),
                    min_y.min(y),
                    max_x.max(x + width),
                    max_y.max(y + height),
                )
            },
        );

        let container_width = max_x - min_x;
        let container_height = max_y - min_y;

        // Create flex container
        let mut container_attrs = Vec::new();
        container_attrs
            .push(self.create_attr("className", "content-stretch flex items-center relative"));
        container_attrs.push(self.create_attr("data-name", "svg-flex-container"));
        container_attrs.push(self.create_attr(
            "style",
            &format!(
                "left: {}px; top: {}px; width: {}px; height: {}px;",
                min_x, min_y, container_width, container_height
            ),
        ));

        // Create flex items
        let mut children = Vec::new();
        for element in elements {
            let flex_item = self.create_flex_item(element, min_x, min_y);
            children.push(JSXElementChild::JSXElement(Box::new(flex_item)));
        }

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
                type_args: None,
                attrs: container_attrs,
                self_closing: false,
            },
            children,
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
            }),
        }
    }

    /// Create a flex item for flexbox layouts
    fn create_flex_item(
        &self,
        element: &GroupedSvgElement,
        container_x: f64,
        container_y: f64,
    ) -> JSXElement {
        let relative_x = element.position.0 - container_x;
        let relative_y = element.position.1 - container_y;
        let (width, height) = element.size;

        // Parse SVG content
        let svg_element = match self.parse_svg_content(&element.svg_content) {
            Some(svg) => svg,
            None => {
                return self.create_placeholder_div(
                    &element.node_id,
                    relative_x,
                    relative_y,
                    width,
                    height,
                );
            }
        };

        // Create flex item
        let mut item_attrs = Vec::new();
        item_attrs.push(self.create_attr("className", "relative shrink-0"));
        item_attrs.push(self.create_attr(
            "style",
            &format!(
                "left: {}px; top: {}px; width: {}px; height: {}px;",
                relative_x, relative_y, width, height
            ),
        ));
        item_attrs.push(self.create_attr("data-name", "Vector"));
        item_attrs.push(self.create_attr("data-node-id", &element.node_id));

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
                type_args: None,
                attrs: item_attrs,
                self_closing: false,
            },
            children: vec![JSXElementChild::JSXElement(Box::new(svg_element))],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
            }),
        }
    }

    /// Create a grouped SVG component (like IconZoomIn from Figma export)
    fn create_grouped_svg_component(&self, elements: Vec<&GroupedSvgElement>) -> JSXElement {
        // Calculate the bounding box for the group
        let (min_x, min_y, max_x, max_y) = elements.iter().fold(
            (
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ),
            |(min_x, min_y, max_x, max_y), element| {
                let (x, y) = element.position;
                let (width, height) = element.size;
                (
                    min_x.min(x),
                    min_y.min(y),
                    max_x.max(x + width),
                    max_y.max(y + height),
                )
            },
        );

        let group_width = max_x - min_x;
        let group_height = max_y - min_y;

        // Create the group container
        let mut container_attrs = Vec::new();
        container_attrs.push(self.create_attr("className", "relative"));
        container_attrs.push(self.create_attr("data-name", "svg-group"));
        container_attrs.push(self.create_attr(
            "style",
            &format!(
                "left: {}px; top: {}px; width: {}px; height: {}px;",
                min_x, min_y, group_width, group_height
            ),
        ));

        // Create positioned children relative to the group
        let mut children = Vec::new();
        for element in elements {
            let relative_x = element.position.0 - min_x;
            let relative_y = element.position.1 - min_y;

            let child_element = self.create_relative_svg_element(element, relative_x, relative_y);
            children.push(JSXElementChild::JSXElement(Box::new(child_element)));
        }

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
                type_args: None,
                attrs: container_attrs,
                self_closing: false,
            },
            children,
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
            }),
        }
    }

    /// Create a relative SVG element within a group
    fn create_relative_svg_element(
        &self,
        element: &GroupedSvgElement,
        relative_x: f64,
        relative_y: f64,
    ) -> JSXElement {
        let (width, height) = element.size;

        // Parse the SVG content
        let svg_element = match self.parse_svg_content(&element.svg_content) {
            Some(svg) => svg,
            None => {
                return self.create_placeholder_div(
                    &element.node_id,
                    relative_x,
                    relative_y,
                    width,
                    height,
                );
            }
        };

        // Create relative positioned wrapper
        let mut div_attrs = Vec::new();
        div_attrs.push(self.create_attr("className", "absolute"));
        div_attrs.push(self.create_attr(
            "style",
            &format!(
                "left: {}px; top: {}px; width: {}px; height: {}px;",
                relative_x, relative_y, width, height
            ),
        ));
        div_attrs.push(self.create_attr("data-name", "Vector"));
        div_attrs.push(self.create_attr("data-node-id", &element.node_id));

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
                type_args: None,
                attrs: div_attrs,
                self_closing: false,
            },
            children: vec![JSXElementChild::JSXElement(Box::new(svg_element))],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
            }),
        }
    }

    /// Create a positioned div wrapper for a single SVG element (following Figma MCP patterns)
    fn create_positioned_svg_wrapper(&self, element: &GroupedSvgElement) -> JSXElement {
        let (x, y) = element.position;
        let (width, height) = element.size;

        // Parse the SVG content to get the actual SVG element
        let svg_element = match self.parse_svg_content(&element.svg_content) {
            Some(svg) => svg,
            None => {
                // Fallback: create a placeholder div
                return self.create_placeholder_div(&element.node_id, x, y, width, height);
            }
        };

        // For single elements, we still use the nested wrapper pattern like Figma MCP
        // but with percentage-based positioning
        self.create_nested_svg_wrapper(element, svg_element)
    }

    /// Create a nested SVG wrapper following Figma MCP's pattern with percentage-based positioning
    fn create_nested_svg_wrapper(
        &self,
        element: &GroupedSvgElement,
        svg_element: JSXElement,
    ) -> JSXElement {
        let (x, y) = element.position;
        let (width, height) = element.size;

        // Calculate parent dimensions (we'll use the element's size as the parent for now)
        // In a real implementation, this would come from the actual parent container
        let parent_width = width * 1.2; // Add some padding
        let parent_height = height * 1.2;

        // Calculate percentage-based inset
        let inset_class = PositioningUtils::calculate_inset_percentages(
            x,
            y,
            width,
            height,
            parent_width,
            parent_height,
        );

        // Create outer positioned div with percentage-based inset
        let mut outer_div_attrs = Vec::new();
        outer_div_attrs.push(self.create_attr("className", &format!("absolute {}", inset_class)));
        outer_div_attrs.push(self.create_attr("data-name", "Vector"));
        outer_div_attrs.push(self.create_attr("data-node-id", &element.node_id));

        // Create inner div with negative inset for overflow (like Figma MCP)
        let inner_inset =
            PositioningUtils::calculate_inner_inset(width, height, parent_width, parent_height);
        let mut inner_div_attrs = Vec::new();
        inner_div_attrs.push(self.create_attr("className", &format!("absolute {}", inner_inset)));

        // Create the nested structure
        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
                type_args: None,
                attrs: outer_div_attrs,
                self_closing: false,
            },
            children: vec![JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(
                        Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                    ),
                    type_args: None,
                    attrs: inner_div_attrs,
                    self_closing: false,
                },
                children: vec![JSXElementChild::JSXElement(Box::new(JSXElement {
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(
                            Ident::new("svg".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                        ),
                        type_args: None,
                        attrs: vec![
                            self.create_attr("className", "block max-w-none size-full"),
                            self.create_attr("viewBox", &format!("0 0 {} {}", width, height)),
                            self.create_attr("width", &width.to_string()),
                            self.create_attr("height", &height.to_string()),
                        ],
                        self_closing: false,
                    },
                    children: self.extract_svg_children(&svg_element),
                    closing: Some(JSXClosingElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(
                            Ident::new("svg".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                        ),
                    }),
                }))],
                closing: Some(JSXClosingElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(
                        Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                    ),
                }),
            }))],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
            }),
        }
    }

    /// Extract children from a parsed SVG element (removes the outer <svg> wrapper)
    fn extract_svg_children(&self, svg_element: &JSXElement) -> Vec<JSXElementChild> {
        // If the parsed element is an SVG, extract its children
        // Otherwise, return the element itself wrapped in a child
        if let JSXElementChild::JSXElement(inner_svg) = &svg_element.children[0] {
            inner_svg.children.clone()
        } else {
            vec![JSXElementChild::JSXElement(Box::new(svg_element.clone()))]
        }
    }

    /// Create a single SVG container with multiple <g> elements for grouped SVG elements
    fn create_grouped_svg_with_g_elements(
        &mut self,
        elements: Vec<&GroupedSvgElement>,
    ) -> JSXElement {
        if elements.is_empty() {
            return self.create_empty_svg_container();
        }

        // Calculate the bounding box for all elements
        let (min_x, min_y, max_x, max_y) = self.calculate_bounding_box(&elements);
        let width = max_x - min_x;
        let height = max_y - min_y;

        // Create the main SVG container
        let mut svg_attrs = Vec::new();
        svg_attrs.push(self.create_attr(
            "viewBox",
            &format!("{} {} {} {}", min_x, min_y, width, height),
        ));
        svg_attrs.push(self.create_attr("width", &width.to_string()));
        svg_attrs.push(self.create_attr("height", &height.to_string()));
        svg_attrs.push(self.create_attr("className", "block max-w-none size-full"));
        svg_attrs.push(self.create_attr("data-name", "GroupedSVG"));
        svg_attrs.push(self.create_attr("data-node-id", &self.parent_id));

        // Create <g> elements for each SVG element with proper transforms
        let mut g_elements = Vec::new();
        for element in elements {
            let g_element = self.create_svg_group_element(element, min_x, min_y);
            g_elements.push(JSXElementChild::JSXElement(Box::new(g_element)));
        }

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("svg".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
                type_args: None,
                attrs: svg_attrs,
                self_closing: false,
            },
            children: g_elements,
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("svg".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
            }),
        }
    }

    /// Create a <g> element for a single SVG element with proper transform
    fn create_svg_group_element(
        &mut self,
        element: &GroupedSvgElement,
        offset_x: f64,
        offset_y: f64,
    ) -> JSXElement {
        let (x, y) = element.position;

        // Calculate transform to position the element relative to the SVG container
        let transform_x = x - offset_x;
        let transform_y = y - offset_y;

        // Extract and register path data from SVG content
        let path_hashes = self.extract_and_register_paths(&element.svg_content);

        // Parse the SVG content to get the actual SVG element
        let svg_content = match self.parse_svg_content(&element.svg_content) {
            Some(svg) => svg,
            None => {
                // Fallback: create a placeholder rect
                return self.create_placeholder_g_element(
                    &element.node_id,
                    transform_x,
                    transform_y,
                );
            }
        };

        // Create <g> element with transform
        let mut g_attrs = Vec::new();
        g_attrs.push(self.create_attr(
            "transform",
            &format!("translate({}, {})", transform_x, transform_y),
        ));
        g_attrs.push(self.create_attr("data-name", "Vector"));
        g_attrs.push(self.create_attr("data-node-id", &element.node_id));

        // Extract the children from the parsed SVG element and replace path data with references
        let children = self.extract_svg_children_with_path_refs(&svg_content, &path_hashes);

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("g".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
                type_args: None,
                attrs: g_attrs,
                self_closing: false,
            },
            children,
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("g".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
            }),
        }
    }

    /// Extract children from SVG element and replace path data with references
    fn extract_svg_children_with_path_refs(
        &self,
        svg_element: &JSXElement,
        path_hashes: &[String],
    ) -> Vec<JSXElementChild> {
        let mut children = self.extract_svg_children(svg_element);

        // Replace path data with references to svgPaths
        for child in &mut children {
            if let JSXElementChild::JSXElement(jsx_elem) = child {
                self.replace_path_data_with_refs(jsx_elem, path_hashes);
            }
        }

        children
    }

    /// Replace path data attributes with svgPaths references
    fn replace_path_data_with_refs(&self, jsx_element: &mut JSXElement, path_hashes: &[String]) {
        // Find and replace 'd' attributes with svgPaths references
        for attr in &mut jsx_element.opening.attrs {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                if let JSXAttrName::Ident(ident_name) = &jsx_attr.name {
                    if ident_name.sym == "d" {
                        // Replace the path data with a reference
                        if let Some(path_hash) = path_hashes.first() {
                            jsx_attr.value =
                                Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                    span: DUMMY_SP,
                                    expr: JSXExpr::Expr(Box::new(Expr::Member(MemberExpr {
                                        span: DUMMY_SP,
                                        obj: Box::new(Expr::Ident(Ident::new(
                                            "svgPaths".into(),
                                            DUMMY_SP,
                                            SyntaxContext::empty(),
                                        ))),
                                        prop: MemberProp::Ident(IdentName::new(
                                            path_hash.as_str().into(),
                                            DUMMY_SP,
                                        )),
                                    }))),
                                }));
                        }
                    }
                }
            }
        }

        // Recursively process children
        for child in &mut jsx_element.children {
            if let JSXElementChild::JSXElement(child_elem) = child {
                self.replace_path_data_with_refs(child_elem, path_hashes);
            }
        }
    }

    /// Create a placeholder <g> element when SVG parsing fails
    fn create_placeholder_g_element(&self, node_id: &str, x: f64, y: f64) -> JSXElement {
        let mut g_attrs = Vec::new();
        g_attrs.push(self.create_attr("transform", &format!("translate({}, {})", x, y)));
        g_attrs.push(self.create_attr("data-svg-placeholder", "true"));
        g_attrs.push(self.create_attr("data-node-id", node_id));

        // Create a placeholder rect
        let rect_attrs = vec![
            self.create_attr("width", "100"),
            self.create_attr("height", "100"),
            self.create_attr("fill", "none"),
            self.create_attr("stroke", "#ccc"),
            self.create_attr("stroke-width", "1"),
        ];

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("g".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
                type_args: None,
                attrs: g_attrs,
                self_closing: false,
            },
            children: vec![JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(
                        Ident::new("rect".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                    ),
                    type_args: None,
                    attrs: rect_attrs,
                    self_closing: true,
                },
                children: vec![],
                closing: None,
            }))],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("g".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
            }),
        }
    }

    /// Calculate the bounding box for all elements
    fn calculate_bounding_box(&self, elements: &[&GroupedSvgElement]) -> (f64, f64, f64, f64) {
        if elements.is_empty() {
            return (0.0, 0.0, 100.0, 100.0);
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for element in elements {
            let (x, y) = element.position;
            let (width, height) = element.size;

            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x + width);
            max_y = max_y.max(y + height);
        }

        // Add some padding
        let padding = 10.0;
        (
            min_x - padding,
            min_y - padding,
            max_x + padding,
            max_y + padding,
        )
    }

    /// Create an empty SVG container
    fn create_empty_svg_container(&self) -> JSXElement {
        let svg_attrs = vec![
            self.create_attr("viewBox", "0 0 100 100"),
            self.create_attr("width", "100"),
            self.create_attr("height", "100"),
            self.create_attr("className", "block max-w-none size-full"),
            self.create_attr("data-name", "EmptySVG"),
        ];

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("svg".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
                type_args: None,
                attrs: svg_attrs,
                self_closing: false,
            },
            children: vec![],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("svg".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
            }),
        }
    }

    /// Parse SVG content and return the JSX element
    fn parse_svg_content(&self, svg_content: &str) -> Option<JSXElement> {
        use super::parser::SvgParser;

        match SvgParser::parse_svg_to_jsx(svg_content) {
            Ok(jsx_element) => Some(jsx_element),
            Err(e) => {
                log::warn!("Failed to parse SVG content for grouping: {}", e);
                None
            }
        }
    }

    /// Create a placeholder div when SVG parsing fails
    fn create_placeholder_div(
        &self,
        node_id: &str,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> JSXElement {
        let mut div_attrs = Vec::new();
        div_attrs
            .push(self.create_attr("className", "absolute bg-gray-200 border border-gray-300"));
        div_attrs.push(self.create_attr(
            "style",
            &format!(
                "left: {}px; top: {}px; width: {}px; height: {}px;",
                x, y, width, height
            ),
        ));
        div_attrs.push(self.create_attr("data-svg-placeholder", "true"));
        div_attrs.push(self.create_attr("data-node-id", node_id));

        JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
                type_args: None,
                attrs: div_attrs,
                self_closing: false,
            },
            children: vec![JSXElementChild::JSXText(JSXText {
                span: DUMMY_SP,
                value: format!("SVG: {}", node_id).into(),
                raw: swc_atoms::Atom::new(""),
            })],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(
                    Ident::new("div".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                ),
            }),
        }
    }

    /// Helper to create JSX attributes
    fn create_attr(&self, name: &str, value: &str) -> JSXAttrOrSpread {
        JSXAttrOrSpread::JSXAttr(JSXAttr {
            span: DUMMY_SP,
            name: JSXAttrName::Ident(
                Ident::new(name.into(), DUMMY_SP, SyntaxContext::empty()).into(),
            ),
            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: value.into(),
                raw: None,
            }))),
        })
    }

    /// Get the number of SVG elements in this group
    pub fn len(&self) -> usize {
        self.svg_elements.len()
    }

    /// Check if the grouper is empty
    pub fn is_empty(&self) -> bool {
        self.svg_elements.is_empty()
    }

    /// Get the path registry for this grouper
    pub fn get_path_registry(&self) -> &PathRegistry {
        &self.path_registry
    }

    /// Get a mutable reference to the path registry
    pub fn get_path_registry_mut(&mut self) -> &mut PathRegistry {
        &mut self.path_registry
    }

    /// Extract path data from SVG content and register it
    fn extract_and_register_paths(&mut self, svg_content: &str) -> Vec<String> {
        let mut path_hashes = Vec::new();

        // Extract all path data from the SVG content
        let path_data_matches = self.find_path_data_in_svg(svg_content);

        for path_data in path_data_matches {
            if self.path_registry.should_extract_path(&path_data) {
                let hash = self.path_registry.register_path(&path_data);
                path_hashes.push(hash);
            }
        }

        path_hashes
    }

    /// Find all path data strings in SVG content
    fn find_path_data_in_svg(&self, svg_content: &str) -> Vec<String> {
        let mut paths = Vec::new();
        let mut content = svg_content;

        while let Some(start) = content.find("d=\"") {
            if let Some(end) = content[start + 3..].find("\"") {
                let path_data = content[start + 3..start + 3 + end].to_string();
                paths.push(path_data);
                content = &content[start + 3 + end + 1..];
            } else {
                break;
            }
        }

        paths
    }
}

/// Groups SVG elements by their parent containers
pub struct SvgGroupingManager {
    /// Groups of SVG elements by parent container ID
    groups: HashMap<String, SvgGrouper>,
}

impl SvgGroupingManager {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    /// Add an SVG element to the appropriate group based on parent hierarchy
    pub fn add_svg_element(&mut self, parent_id: &str, element: GroupedSvgElement) {
        self.groups
            .entry(parent_id.to_string())
            .or_insert_with(|| SvgGrouper::new(parent_id.to_string()))
            .add_svg_element(element);
    }

    /// Get all groups that have SVG elements
    pub fn get_groups(&self) -> &HashMap<String, SvgGrouper> {
        &self.groups
    }

    /// Create positioned SVG elements for all groups
    pub fn create_all_positioned_svgs(&mut self) -> HashMap<String, Vec<JSXElement>> {
        let mut result = HashMap::new();

        for (parent_id, grouper) in &mut self.groups {
            let positioned_svgs = grouper.create_positioned_svg_elements();
            if !positioned_svgs.is_empty() {
                result.insert(parent_id.clone(), positioned_svgs);
            }
        }

        result
    }

    /// Check if there are any SVG groups
    pub fn has_groups(&self) -> bool {
        !self.groups.is_empty()
    }

    /// Get all path registries from all groups
    pub fn get_all_path_registries(&self) -> Vec<&PathRegistry> {
        self.groups
            .values()
            .map(|grouper| grouper.get_path_registry())
            .collect()
    }

    /// Generate a combined TypeScript path file from all registries
    pub fn generate_combined_path_file(&self) -> String {
        let mut combined_paths = HashMap::new();

        // Collect all paths from all registries
        for registry in self.get_all_path_registries() {
            for (hash, path_data) in registry.paths() {
                combined_paths.insert(hash.clone(), path_data.clone());
            }
        }

        // Generate the TypeScript file content
        let mut content = String::from("// Auto-generated SVG path data\n");
        content.push_str("// Do not edit manually\n\n");
        content.push_str("export default {\n");

        let mut sorted_paths: Vec<_> = combined_paths.iter().collect();
        sorted_paths.sort_by_key(|(hash, _)| *hash);

        for (hash, path_data) in sorted_paths {
            // Escape quotes in path data
            let escaped = path_data.replace('\\', "\\\\").replace('"', "\\\"");
            content.push_str(&format!("  {}: \"{}\",\n", hash, escaped));
        }

        content.push_str("};\n");
        content
    }
}
