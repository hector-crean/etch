use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "props")]
pub enum Formatting {
    Bold,
    Italic,
    Underline,
    StrikeThrough,
    FontSize(u32),
    FontColor(u8, u8, u8), // RGB
    FontFamily(String),
    BackgroundColor(u8, u8, u8),
    Superscript,
    Subscript,
    Alignment(TextAlignment),
    Link { url: String },
    Citation { text: String },
    Code { language: Option<String> },
    Quote { source: Option<String> },
    Footnote { note: String },
    Comment { text: String },
    Highlight,
    CustomClass(String),
    CustomAttribute { key: String, value: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

pub struct FormattedText {
    text: Vec<(String, Vec<Formatting>)>,
}
