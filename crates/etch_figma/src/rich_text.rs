use figma_api::models::{
    Paint, TextNode, TypeStyle,
    type_style::{TextAlignHorizontal, TextDecoration},
};

use swc_common::{DUMMY_SP, SourceMap, SyntaxContext, sync::Lrc};
use swc_ecma_ast::*;
use swc_ecma_codegen::{Emitter, text_writer::JsWriter};

#[derive(Debug, Clone, Copy)]
pub enum ColorStrategy {
    /// Use inline hex colors (e.g., style="color: #ff0000;")
    InlineHex,
    /// Use Tailwind text color classes (e.g., text-red-500)
    TailwindTextColors,
    /// Don't apply any color styling
    None,
}

#[derive(Debug, Clone)]
pub struct StyleClassMapping {
    // Font weight mappings
    pub font_thin: String,
    pub font_extralight: String,
    pub font_light: String,
    pub font_normal: String,
    pub font_medium: String,
    pub font_semibold: String,
    pub font_bold: String,
    pub font_extrabold: String,
    pub font_black: String,

    // Font size mappings (Tailwind text-* classes)
    pub text_xs: String,   // 12px
    pub text_sm: String,   // 14px
    pub text_base: String, // 16px
    pub text_lg: String,   // 18px
    pub text_xl: String,   // 20px
    pub text_2xl: String,  // 24px
    pub text_3xl: String,  // 30px
    pub text_4xl: String,  // 36px
    pub text_5xl: String,  // 48px
    pub text_6xl: String,  // 60px
    pub text_7xl: String,  // 72px
    pub text_8xl: String,  // 96px
    pub text_9xl: String,  // 128px

    // Line height mappings
    pub leading_none: String,    // 1
    pub leading_tight: String,   // 1.25
    pub leading_snug: String,    // 1.375
    pub leading_normal: String,  // 1.5
    pub leading_relaxed: String, // 1.625
    pub leading_loose: String,   // 2

    // Letter spacing mappings
    pub tracking_tighter: String, // -0.05em
    pub tracking_tight: String,   // -0.025em
    pub tracking_normal: String,  // 0em
    pub tracking_wide: String,    // 0.025em
    pub tracking_wider: String,   // 0.05em
    pub tracking_widest: String,  // 0.1em

    // Text decoration
    pub underline: String,
    pub line_through: String,

    // Text alignment
    pub text_left: String,
    pub text_center: String,
    pub text_right: String,
    pub text_justify: String,

    // Style mapping preferences
    pub use_tailwind_font_sizes: bool,
    pub use_tailwind_line_heights: bool,
    pub use_tailwind_letter_spacing: bool,
    pub color_strategy: ColorStrategy,

    // Fallback to inline styles when no Tailwind class matches
    pub fallback_to_inline_font_size: bool,
    pub fallback_to_inline_line_height: bool,
    pub fallback_to_inline_letter_spacing: bool,

    // Wrapper element
    pub wrapper_tag: String,

    // Superscript detection
    pub detect_superscripts: bool,
    pub superscript_size_threshold: f32, // Ratio threshold (e.g., 0.75 means 75% of base size)
}

impl Default for StyleClassMapping {
    fn default() -> Self {
        Self {
            // Font weight classes (Tailwind defaults)
            font_thin: "font-thin".to_string(),
            font_extralight: "font-extralight".to_string(),
            font_light: "font-light".to_string(),
            font_normal: "font-normal".to_string(),
            font_medium: "font-medium".to_string(),
            font_semibold: "font-semibold".to_string(),
            font_bold: "font-bold".to_string(),
            font_extrabold: "font-extrabold".to_string(),
            font_black: "font-black".to_string(),

            // Font size classes (Tailwind defaults)
            text_xs: "text-xs".to_string(),
            text_sm: "text-sm".to_string(),
            text_base: "text-base".to_string(),
            text_lg: "text-lg".to_string(),
            text_xl: "text-xl".to_string(),
            text_2xl: "text-2xl".to_string(),
            text_3xl: "text-3xl".to_string(),
            text_4xl: "text-4xl".to_string(),
            text_5xl: "text-5xl".to_string(),
            text_6xl: "text-6xl".to_string(),
            text_7xl: "text-7xl".to_string(),
            text_8xl: "text-8xl".to_string(),
            text_9xl: "text-9xl".to_string(),

            // Line height classes (Tailwind defaults)
            leading_none: "leading-none".to_string(),
            leading_tight: "leading-tight".to_string(),
            leading_snug: "leading-snug".to_string(),
            leading_normal: "leading-normal".to_string(),
            leading_relaxed: "leading-relaxed".to_string(),
            leading_loose: "leading-loose".to_string(),

            // Letter spacing classes (Tailwind defaults)
            tracking_tighter: "tracking-tighter".to_string(),
            tracking_tight: "tracking-tight".to_string(),
            tracking_normal: "tracking-normal".to_string(),
            tracking_wide: "tracking-wide".to_string(),
            tracking_wider: "tracking-wider".to_string(),
            tracking_widest: "tracking-widest".to_string(),

            // Text decoration classes
            underline: "underline".to_string(),
            line_through: "line-through".to_string(),

            // Text alignment classes
            text_left: "text-left".to_string(),
            text_center: "text-center".to_string(),
            text_right: "text-right".to_string(),
            text_justify: "text-justify".to_string(),

            // Prefer Tailwind classes over inline styles
            use_tailwind_font_sizes: true,
            use_tailwind_line_heights: true,
            use_tailwind_letter_spacing: true,
            color_strategy: ColorStrategy::TailwindTextColors,

            // Only fallback to inline styles if no Tailwind class matches
            fallback_to_inline_font_size: false,
            fallback_to_inline_line_height: false,
            fallback_to_inline_letter_spacing: false,

            wrapper_tag: "div".to_string(),

            // Superscript detection defaults
            detect_superscripts: true,
            superscript_size_threshold: 0.75, // 75% of base font size
        }
    }
}

impl StyleClassMapping {
    pub fn new() -> Self {
        Self::default()
    }

    // Builder methods for font weights
    pub fn font_thin<S: Into<String>>(mut self, class: S) -> Self {
        self.font_thin = class.into();
        self
    }

    pub fn font_extralight<S: Into<String>>(mut self, class: S) -> Self {
        self.font_extralight = class.into();
        self
    }

    pub fn font_light<S: Into<String>>(mut self, class: S) -> Self {
        self.font_light = class.into();
        self
    }

    pub fn font_normal<S: Into<String>>(mut self, class: S) -> Self {
        self.font_normal = class.into();
        self
    }

    pub fn font_medium<S: Into<String>>(mut self, class: S) -> Self {
        self.font_medium = class.into();
        self
    }

    pub fn font_semibold<S: Into<String>>(mut self, class: S) -> Self {
        self.font_semibold = class.into();
        self
    }

    pub fn font_bold<S: Into<String>>(mut self, class: S) -> Self {
        self.font_bold = class.into();
        self
    }

    pub fn font_extrabold<S: Into<String>>(mut self, class: S) -> Self {
        self.font_extrabold = class.into();
        self
    }

    pub fn font_black<S: Into<String>>(mut self, class: S) -> Self {
        self.font_black = class.into();
        self
    }

    // Builder methods for decorations
    pub fn underline<S: Into<String>>(mut self, class: S) -> Self {
        self.underline = class.into();
        self
    }

    pub fn line_through<S: Into<String>>(mut self, class: S) -> Self {
        self.line_through = class.into();
        self
    }

    // Builder methods for alignment
    pub fn text_left<S: Into<String>>(mut self, class: S) -> Self {
        self.text_left = class.into();
        self
    }

    pub fn text_center<S: Into<String>>(mut self, class: S) -> Self {
        self.text_center = class.into();
        self
    }

    pub fn text_right<S: Into<String>>(mut self, class: S) -> Self {
        self.text_right = class.into();
        self
    }

    pub fn text_justify<S: Into<String>>(mut self, class: S) -> Self {
        self.text_justify = class.into();
        self
    }

    // Builder methods for Tailwind preferences
    pub fn use_tailwind_font_sizes(mut self, enabled: bool) -> Self {
        self.use_tailwind_font_sizes = enabled;
        self
    }

    pub fn use_tailwind_line_heights(mut self, enabled: bool) -> Self {
        self.use_tailwind_line_heights = enabled;
        self
    }

    pub fn use_tailwind_letter_spacing(mut self, enabled: bool) -> Self {
        self.use_tailwind_letter_spacing = enabled;
        self
    }

    // Builder methods for inline style fallbacks
    pub fn fallback_to_inline_font_size(mut self, enabled: bool) -> Self {
        self.fallback_to_inline_font_size = enabled;
        self
    }

    pub fn fallback_to_inline_line_height(mut self, enabled: bool) -> Self {
        self.fallback_to_inline_line_height = enabled;
        self
    }

    pub fn fallback_to_inline_letter_spacing(mut self, enabled: bool) -> Self {
        self.fallback_to_inline_letter_spacing = enabled;
        self
    }

    pub fn color_strategy(mut self, strategy: ColorStrategy) -> Self {
        self.color_strategy = strategy;
        self
    }

    pub fn wrapper_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.wrapper_tag = tag.into();
        self
    }

    pub fn detect_superscripts(mut self, enabled: bool) -> Self {
        self.detect_superscripts = enabled;
        self
    }

    pub fn superscript_size_threshold(mut self, threshold: f32) -> Self {
        self.superscript_size_threshold = threshold;
        self
    }
}

fn color_to_css(paint: &Paint, strategy: ColorStrategy) -> Option<String> {
    match strategy {
        ColorStrategy::None => None,
        ColorStrategy::InlineHex => match paint {
            Paint::SolidPaint(sp) => {
                let color = &sp.color;
                let r = (color.r * 255.0).round() as i32;
                let g = (color.g * 255.0).round() as i32;
                let b = (color.b * 255.0).round() as i32;
                Some(format!("color: #{:02x}{:02x}{:02x};", r, g, b))
            }
            _ => None,
        },
        ColorStrategy::TailwindTextColors => match paint {
            Paint::SolidPaint(sp) => {
                let color = &sp.color;
                // Map common colors to Tailwind classes
                // This is a simplified mapping - you might want to expand this
                match (
                    (color.r * 255.0).round() as i32,
                    (color.g * 255.0).round() as i32,
                    (color.b * 255.0).round() as i32,
                ) {
                    (0, 0, 0) => Some("text-black".to_string()),
                    (255, 255, 255) => Some("text-white".to_string()),
                    (239, 68, 68) => Some("text-red-500".to_string()),
                    (34, 197, 94) => Some("text-green-500".to_string()),
                    (59, 130, 246) => Some("text-blue-500".to_string()),
                    (168, 85, 247) => Some("text-purple-500".to_string()),
                    (245, 158, 11) => Some("text-amber-500".to_string()),
                    (107, 114, 128) => Some("text-gray-500".to_string()),
                    // Fallback to hex for uncommon colors
                    _ => {
                        let r = (color.r * 255.0).round() as i32;
                        let g = (color.g * 255.0).round() as i32;
                        let b = (color.b * 255.0).round() as i32;
                        Some(format!("color: #{:02x}{:02x}{:02x};", r, g, b))
                    }
                }
            }
            _ => None,
        },
    }
}

fn font_size_to_tailwind(font_size: f32, mapping: &StyleClassMapping) -> Option<String> {
    let size = font_size.round() as i32;
    match size {
        12 => Some(mapping.text_xs.clone()),
        14 => Some(mapping.text_sm.clone()),
        16 => Some(mapping.text_base.clone()),
        18 => Some(mapping.text_lg.clone()),
        20 => Some(mapping.text_xl.clone()),
        24 => Some(mapping.text_2xl.clone()),
        30 => Some(mapping.text_3xl.clone()),
        36 => Some(mapping.text_4xl.clone()),
        48 => Some(mapping.text_5xl.clone()),
        60 => Some(mapping.text_6xl.clone()),
        72 => Some(mapping.text_7xl.clone()),
        96 => Some(mapping.text_8xl.clone()),
        128 => Some(mapping.text_9xl.clone()),
        _ => None,
    }
}

fn line_height_to_tailwind(
    line_height: f32,
    font_size: Option<f32>,
    mapping: &StyleClassMapping,
) -> Option<String> {
    // Convert line height to ratio if we have font size
    if let Some(fs) = font_size {
        let ratio = line_height / fs;
        match (ratio * 100.0).round() as i32 {
            100 => Some(mapping.leading_none.clone()),
            125 => Some(mapping.leading_tight.clone()),
            137..=138 => Some(mapping.leading_snug.clone()),
            150 => Some(mapping.leading_normal.clone()),
            162..=163 => Some(mapping.leading_relaxed.clone()),
            200 => Some(mapping.leading_loose.clone()),
            _ => None,
        }
    } else {
        None
    }
}

fn letter_spacing_to_tailwind(letter_spacing: f32, mapping: &StyleClassMapping) -> Option<String> {
    // Letter spacing in Figma is in pixels, convert to approximate em values
    let em_value = letter_spacing / 16.0; // Assuming 16px base font size
    match (em_value * 1000.0).round() as i32 {
        -50 => Some(mapping.tracking_tighter.clone()),
        -25 => Some(mapping.tracking_tight.clone()),
        0 => Some(mapping.tracking_normal.clone()),
        25 => Some(mapping.tracking_wide.clone()),
        50 => Some(mapping.tracking_wider.clone()),
        100 => Some(mapping.tracking_widest.clone()),
        _ => None,
    }
}

fn style_to_attrs(
    style: &TypeStyle,
    base_style: &TypeStyle,
    mapping: &StyleClassMapping,
) -> (Vec<String>, Option<String>, Option<String>) {
    let mut classes: Vec<String> = Vec::new();
    let mut inline_style: Vec<String> = Vec::new();
    let mut tag: Option<String> = None;

    // Check if this should be a superscript
    if mapping.detect_superscripts {
        if let (Some(current_size), Some(base_size)) = (style.font_size, base_style.font_size) {
            let size_ratio = (current_size / base_size) as f32;
            if size_ratio <= mapping.superscript_size_threshold {
                tag = Some("sup".to_string());
            }
        }
    }

    // Font size - prefer Tailwind classes
    if let Some(font_size) = style.font_size {
        if mapping.use_tailwind_font_sizes {
            if let Some(tailwind_class) = font_size_to_tailwind(font_size as f32, mapping) {
                if !tailwind_class.is_empty() {
                    classes.push(tailwind_class);
                }
            } else if mapping.fallback_to_inline_font_size {
                inline_style.push(format!("font-size: {}px;", font_size));
            }
        } else if mapping.fallback_to_inline_font_size {
            inline_style.push(format!("font-size: {}px;", font_size));
        }
    }

    // Font weight
    if let Some(weight) = style.font_weight {
        let class = match weight.round() as i32 {
            100 => Some(&mapping.font_thin),
            200 => Some(&mapping.font_extralight),
            300 => Some(&mapping.font_light),
            400 => Some(&mapping.font_normal),
            500 => Some(&mapping.font_medium),
            600 => Some(&mapping.font_semibold),
            700 => Some(&mapping.font_bold),
            800 => Some(&mapping.font_extrabold),
            900 => Some(&mapping.font_black),
            _ => None,
        };
        if let Some(class) = class {
            if !class.is_empty() {
                classes.push(class.clone());
            }
        }
    }

    // Line height - prefer Tailwind classes
    if let Some(line_height) = style.line_height_px {
        if mapping.use_tailwind_line_heights {
            if let Some(tailwind_class) = line_height_to_tailwind(
                line_height as f32,
                style.font_size.map(|fs| fs as f32),
                mapping,
            ) {
                if !tailwind_class.is_empty() {
                    classes.push(tailwind_class);
                }
            } else if mapping.fallback_to_inline_line_height {
                inline_style.push(format!("line-height: {}px;", line_height));
            }
        } else if mapping.fallback_to_inline_line_height {
            inline_style.push(format!("line-height: {}px;", line_height));
        }
    }

    // Letter spacing - prefer Tailwind classes
    if let Some(letter) = style.letter_spacing {
        if mapping.use_tailwind_letter_spacing {
            if let Some(tailwind_class) = letter_spacing_to_tailwind(letter as f32, mapping) {
                if !tailwind_class.is_empty() {
                    classes.push(tailwind_class);
                }
            } else if mapping.fallback_to_inline_letter_spacing {
                inline_style.push(format!("letter-spacing: {}px;", letter));
            }
        } else if mapping.fallback_to_inline_letter_spacing {
            inline_style.push(format!("letter-spacing: {}px;", letter));
        }
    }

    // Text decoration
    if let Some(ref decoration) = style.text_decoration {
        let class = match decoration {
            TextDecoration::Underline => Some(&mapping.underline),
            TextDecoration::Strikethrough => Some(&mapping.line_through),
            _ => None,
        };
        if let Some(class) = class {
            if !class.is_empty() {
                classes.push(class.clone());
            }
        }
    }

    // Text alignment
    if let Some(ref align) = style.text_align_horizontal {
        let class = match align {
            TextAlignHorizontal::Left => Some(&mapping.text_left),
            TextAlignHorizontal::Center => Some(&mapping.text_center),
            TextAlignHorizontal::Right => Some(&mapping.text_right),
            TextAlignHorizontal::Justified => Some(&mapping.text_justify),
        };
        if let Some(class) = class {
            if !class.is_empty() {
                classes.push(class.clone());
            }
        }
    }

    // Colors - handle both Tailwind classes and inline styles
    if let Some(ref fills) = style.fills {
        for paint in fills {
            if let Some(color_result) = color_to_css(paint, mapping.color_strategy) {
                match mapping.color_strategy {
                    ColorStrategy::TailwindTextColors => {
                        // color_result might be a class name or inline style
                        if color_result.starts_with("text-") {
                            classes.push(color_result);
                        } else {
                            // Fallback inline style
                            inline_style.push(color_result);
                        }
                    }
                    ColorStrategy::InlineHex => {
                        inline_style.push(color_result);
                    }
                    ColorStrategy::None => {}
                }
                break;
            }
        }
    }

    (
        classes,
        if inline_style.is_empty() {
            None
        } else {
            Some(inline_style.join(" "))
        },
        tag,
    )
}

/// Parse CSS string into JSX style object properties
fn parse_css_to_jsx_style_props(css_string: &str) -> Vec<PropOrSpread> {
    let mut props = Vec::new();

    // Split by semicolon and parse each CSS property
    for declaration in css_string.split(';') {
        let declaration = declaration.trim();
        if declaration.is_empty() {
            continue;
        }

        if let Some((property, value)) = declaration.split_once(':') {
            let property = property.trim();
            let value = value.trim();

            // Convert CSS property names to camelCase for JSX
            let jsx_property = match property {
                "font-size" => "fontSize",
                "line-height" => "lineHeight",
                "letter-spacing" => "letterSpacing",
                "font-weight" => "fontWeight",
                "text-decoration" => "textDecoration",
                "text-align" => "textAlign",
                "color" => "color",
                _ => property, // fallback for other properties
            };

            props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: jsx_property.into(),
                }),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: value.into(),
                    raw: None,
                }))),
            }))));
        }
    }

    props
}

/// Convert a Figma TextNode directly into a SWC JSXElement tree without intermediate HTML.
pub fn textnode_to_jsx_with_mapping(text: &TextNode, mapping: &StyleClassMapping) -> JSXElement {
    // Build runs from character_style_overrides and style_override_table
    let chars: Vec<char> = text.characters.chars().collect();
    let overrides = &text.character_style_overrides;

    let mut runs: Vec<(usize, usize, i64)> = Vec::new();
    if !chars.is_empty() {
        let mut i: usize = 0;
        while i < chars.len() {
            let current_idx = overrides.get(i).cloned().unwrap_or(0.0) as i64;
            let mut j = i + 1;
            while j < chars.len() {
                let idx = overrides.get(j).cloned().unwrap_or(0.0) as i64;
                if idx != current_idx {
                    break;
                }
                j += 1;
            }
            runs.push((i, j, current_idx));
            i = j;
        }
    }

    // Helper to create a text child node
    fn jsx_text_child(text: String) -> JSXElementChild {
        JSXElementChild::JSXExprContainer(JSXExprContainer {
            span: DUMMY_SP,
            expr: JSXExpr::Expr(Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: text.into(),
                raw: None,
            })))),
        })
    }

    // Build children per run; default to a single text child when no runs
    let mut children: Vec<JSXElementChild> = Vec::new();
    if runs.is_empty() {
        children.push(jsx_text_child(text.characters.clone()));
    } else {
        for (start, end, idx) in runs {
            let base_style = &text.style;
            let (mut classes, inline_style, mut tag) = style_to_attrs(base_style, base_style, mapping);
            let mut style_string = inline_style;

            if idx != 0 {
                let key = idx.to_string();
                if let Some(override_style) = text.style_override_table.get(&key) {
                    let (o_classes, o_inline, o_tag) = style_to_attrs(override_style, base_style, mapping);
                    classes.extend(o_classes);
                    // Use override tag if present
                    if o_tag.is_some() {
                        tag = o_tag;
                    }
                    // Merge inline styles by concatenation; JSX style prop as string attribute
                    let merged_inline = match (style_string.clone(), o_inline) {
                        (Some(mut a), Some(b)) => {
                            a.push(' ');
                            a.push_str(&b);
                            Some(a)
                        }
                        (Some(a), None) => Some(a),
                        (None, Some(b)) => Some(b),
                        (None, None) => None,
                    };

                    // Replace inline style with merged
                    style_string = merged_inline;
                }
            }

            let text_segment: String = text
                .characters
                .chars()
                .skip(start)
                .take(end - start)
                .collect();

            // If there are no classes or inline styles, push plain text child
            let all_classes = classes;

            // Determine the element tag to use (span, sup, etc.)
            let element_tag = tag.as_deref().unwrap_or("span");

            if all_classes.is_empty() && style_string.is_none() && element_tag == "span" {
                children.push(jsx_text_child(text_segment));
            } else {
                // Build <element_tag className="..." style="...">text</element_tag>
                let mut attrs: Vec<JSXAttrOrSpread> = Vec::new();
                if !all_classes.is_empty() {
                    attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                        span: DUMMY_SP,
                        name: JSXAttrName::Ident(IdentName {
                            span: DUMMY_SP,
                            sym: "className".into(),
                        }),
                        value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: all_classes.join(" ").into(),
                            raw: None,
                        }))),
                    }));
                }
                if let Some(style_attr) = style_string.clone() {
                    // Parse CSS string into JSX style object
                    let style_props = parse_css_to_jsx_style_props(&style_attr);

                    if !style_props.is_empty() {
                        attrs.push(JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(IdentName {
                                span: DUMMY_SP,
                                sym: "style".into(),
                            }),
                            value: Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                span: DUMMY_SP,
                                expr: JSXExpr::Expr(Box::new(Expr::Object(ObjectLit {
                                    span: DUMMY_SP,
                                    props: style_props,
                                }))),
                            })),
                        }));
                    }
                }

                let styled_element = JSXElement {
                    span: DUMMY_SP,
                    opening: JSXOpeningElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: element_tag.into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                        attrs,
                        self_closing: false,
                        type_args: None,
                    },
                    children: vec![jsx_text_child(text_segment)],
                    closing: Some(JSXClosingElement {
                        span: DUMMY_SP,
                        name: JSXElementName::Ident(Ident {
                            span: DUMMY_SP,
                            sym: element_tag.into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        }),
                    }),
                };

                children.push(JSXElementChild::JSXElement(Box::new(styled_element)));
            }
        }
    }

    // Wrap in a container to return a JSXElement root
    JSXElement {
        span: DUMMY_SP,
        opening: JSXOpeningElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: mapping.wrapper_tag.as_str().into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
            attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(IdentName {
                    span: DUMMY_SP,
                    sym: "data-rich-text".into(),
                }),
                value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: "true".into(),
                    raw: None,
                }))),
            })],
            self_closing: false,
            type_args: None,
        },
        children,
        closing: Some(JSXClosingElement {
            span: DUMMY_SP,
            name: JSXElementName::Ident(Ident {
                span: DUMMY_SP,
                sym: mapping.wrapper_tag.as_str().into(),
                optional: false,
                ctxt: SyntaxContext::empty(),
            }),
        }),
    }
}

pub fn textnode_to_jsx(text: &TextNode) -> JSXElement {
    textnode_to_jsx_with_mapping(text, &StyleClassMapping::default())
}

pub trait TextNodeExt {
    fn to_jsx(&self) -> JSXElement;
    fn to_jsx_with_mapping(&self, mapping: &StyleClassMapping) -> JSXElement;
}

impl TextNodeExt for TextNode {
    fn to_jsx(&self) -> JSXElement {
        textnode_to_jsx(self)
    }

    fn to_jsx_with_mapping(&self, mapping: &StyleClassMapping) -> JSXElement {
        textnode_to_jsx_with_mapping(self, mapping)
    }
}

/// Convert a JSXElement to a TSX/JSX string representation
pub fn jsx_element_to_string(
    jsx_element: &JSXElement,
) -> Result<String, Box<dyn std::error::Error>> {
    let cm: Lrc<SourceMap> = Default::default();

    // Create a simple module containing just the JSX element as an expression
    let module = Module {
        span: DUMMY_SP,
        body: vec![ModuleItem::Stmt(Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(Expr::JSXElement(Box::new(jsx_element.clone()))),
        }))],
        shebang: None,
    };

    let mut output = Vec::new();
    {
        let writer = JsWriter::new(cm.clone(), "\n", &mut output, None);
        let mut emitter = Emitter {
            cfg: swc_ecma_codegen::Config::default(),
            cm: cm.clone(),
            comments: None,
            wr: writer,
        };
        emitter.emit_module(&module)?;
    }

    let mut result = String::from_utf8(output)?;

    // Remove the trailing semicolon and newline that gets added to the expression statement
    if result.ends_with(";\n") {
        result.truncate(result.len() - 2);
    } else if result.ends_with(';') {
        result.truncate(result.len() - 1);
    }

    Ok(result)
}

/// Extension trait to add string conversion to JSXElement
pub trait JSXElementExt {
    fn to_string(&self) -> Result<String, Box<dyn std::error::Error>>;
}

impl JSXElementExt for JSXElement {
    fn to_string(&self) -> Result<String, Box<dyn std::error::Error>> {
        jsx_element_to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use figma_api::models::{Paint, SolidPaint, TypeStyle};

    // #[test]
    // fn test_style_prop_as_object() {
    //     // Create a simple style with inline properties
    //     let style = TypeStyle {
    //         font_size: Some(16.0),
    //         font_weight: Some(700.0),
    //         fills: Some(vec![Paint::SolidPaint(SolidPaint {
    //             color: Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
    //             opacity: Some(1.0),
    //             visible: Some(true),
    //             blend_mode: None,
    //         })]),
    //         ..Default::default()
    //     };

    //     let mapping = StyleClassMapping::new()
    //         .use_tailwind_font_sizes(false)
    //         .fallback_to_inline_font_size(true)
    //         .color_strategy(ColorStrategy::InlineHex);

    //     let (classes, inline_style, _) = style_to_attrs(&style, &mapping);

    //     // Verify that we get inline styles
    //     assert!(inline_style.is_some());
    //     let style_string = inline_style.unwrap();
    //     assert!(style_string.contains("font-size: 16px"));
    //     assert!(style_string.contains("color: #ff0000"));

    //     // Test CSS parsing
    //     let props = parse_css_to_jsx_style_props(&style_string);
    //     assert!(!props.is_empty());

    //     // The props should contain fontSize and color as camelCase properties
    //     println!("Generated style props: {:?}", props);
    // }
}
