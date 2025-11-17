//! Error types for the etch_figma crate

use std::fmt;
use std::io;

/// Result type alias for etch_figma operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for all etch_figma operations
#[derive(Debug)]
pub enum Error {
    /// I/O error when reading or writing files
    Io(io::Error),

    /// Error parsing SVG content
    SvgParse {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Error during code generation
    CodeGen {
        message: String,
        context: Option<String>,
    },

    /// Error from the Figma API
    FigmaApi {
        message: String,
        node_id: Option<String>,
    },

    /// Invalid configuration
    Config { field: String, message: String },

    /// Missing required data
    MissingData { what: String, context: String },

    /// Error during visitor traversal
    Visitor {
        message: String,
        node_id: Option<String>,
    },

    /// UTF-8 conversion error
    Utf8(std::string::FromUtf8Error),

    /// Generic error for compatibility
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::SvgParse { message, .. } => write!(f, "SVG parse error: {}", message),
            Error::CodeGen { message, context } => {
                if let Some(ctx) = context {
                    write!(f, "Code generation error: {} (context: {})", message, ctx)
                } else {
                    write!(f, "Code generation error: {}", message)
                }
            }
            Error::FigmaApi { message, node_id } => {
                if let Some(id) = node_id {
                    write!(f, "Figma API error for node {}: {}", id, message)
                } else {
                    write!(f, "Figma API error: {}", message)
                }
            }
            Error::Config { field, message } => {
                write!(f, "Configuration error in field '{}': {}", field, message)
            }
            Error::MissingData { what, context } => {
                write!(f, "Missing {}: {}", what, context)
            }
            Error::Visitor { message, node_id } => {
                if let Some(id) = node_id {
                    write!(f, "Visitor error at node {}: {}", id, message)
                } else {
                    write!(f, "Visitor error: {}", message)
                }
            }
            Error::Utf8(err) => write!(f, "UTF-8 conversion error: {}", err),
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Utf8(err) => Some(err),
            Error::SvgParse { source, .. } => source
                .as_ref()
                .map(|s| s.as_ref() as &(dyn std::error::Error + 'static)),
            _ => None,
        }
    }
}

// Conversions from common error types
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::Utf8(err)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Other(msg)
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::Other(msg.to_string())
    }
}

// Forward compatibility for external error types
// These allow seamless conversion from other error types in the crate
impl From<crate::conversion::svg::async_bridge::SvgBridgeError> for Error {
    fn from(err: crate::conversion::svg::async_bridge::SvgBridgeError) -> Self {
        Error::FigmaApi {
            message: err.to_string(),
            node_id: None,
        }
    }
}

// Helper constructors for common error patterns
impl Error {
    /// Create a code generation error
    pub fn codegen(message: impl Into<String>) -> Self {
        Error::CodeGen {
            message: message.into(),
            context: None,
        }
    }

    /// Create a code generation error with context
    pub fn codegen_with_context(message: impl Into<String>, context: impl Into<String>) -> Self {
        Error::CodeGen {
            message: message.into(),
            context: Some(context.into()),
        }
    }

    /// Create a Figma API error
    pub fn figma_api(message: impl Into<String>) -> Self {
        Error::FigmaApi {
            message: message.into(),
            node_id: None,
        }
    }

    /// Create a Figma API error for a specific node
    pub fn figma_api_node(message: impl Into<String>, node_id: impl Into<String>) -> Self {
        Error::FigmaApi {
            message: message.into(),
            node_id: Some(node_id.into()),
        }
    }

    /// Create a configuration error
    pub fn config(field: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Config {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a missing data error
    pub fn missing(what: impl Into<String>, context: impl Into<String>) -> Self {
        Error::MissingData {
            what: what.into(),
            context: context.into(),
        }
    }

    /// Create an SVG parse error
    pub fn svg_parse(message: impl Into<String>) -> Self {
        Error::SvgParse {
            message: message.into(),
            source: None,
        }
    }
}
