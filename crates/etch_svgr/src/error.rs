use thiserror::Error;

#[derive(Error, Debug)]
pub enum SvgrError {
    #[error("Failed to parse SVG: {0}")]
    Parse(String),
    #[error("Invalid SVG content")]
    InvalidSvg,
    #[error("Empty SVG content provided")]
    EmptyContent,
}
