use thiserror::Error;

#[derive(Error, Debug)]
pub enum TsxError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("FromUtf8Error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("RawHtmlError: {0}")]
    RawHtmlError(#[from] crate::raw_html::RawHtmlError),
}
