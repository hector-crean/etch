use serde::{Deserialize, Serialize};
use swc_common::{sync::Lrc, FileName, SourceMap, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser as SwcParser, StringInput, Syntax, TsSyntax};
use thiserror::Error;
use ts_rs::TS;

/// A wrapper around raw HTML content that can be parsed into JSX elements.
/// This is useful for handling HTML content that needs to be processed as JSX.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash, TS)]
#[ts(export)]
pub struct RawHtml(pub String);

/// Errors that can occur when parsing HTML into JSX elements
#[derive(Debug, thiserror::Error)]
pub enum RawHtmlError {
    #[error("Failed to parse JSX: {0}")]
    ParseError(String),
    #[error("No JSX element found in parsed content")]
    NoJsxElement,
    #[error("Invalid JSX fragment structure")]
    InvalidFragment,
    #[error("Empty HTML content provided")]
    EmptyContent,
}

impl RawHtml {
    /// Creates a new RawHtml instance from a string or string-like value
    pub fn new(html: impl Into<String>) -> Self {
        Self(html.into())
    }

    /// Returns a reference to the inner HTML string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the inner HTML string
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Checks if the HTML content is empty
    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }

    /// Parses the HTML content into a JSX element
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The content is empty
    /// - The content cannot be parsed as JSX
    /// - No valid JSX element is found
    pub fn parse(&self) -> Result<JSXElement, RawHtmlError> {
        if self.is_empty() {
            return Err(RawHtmlError::EmptyContent);
        }

        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(
            FileName::Anon.into(),
            // Wrap the HTML in a JSX expression for parsing
            format!("<>{}</>;", self.0),
        );

        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsx: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = SwcParser::new_from(lexer);
        let module = parser
            .parse_module()
            .map_err(|e| RawHtmlError::ParseError(format!("{:?}", e)))?;

        // Extract the JSX element from the module
        if let Some(ModuleItem::Stmt(Stmt::Expr(expr_stmt))) = module.body.first() {
            if let Expr::JSXFragment(fragment) = &*expr_stmt.expr {
                if let Some(child) = fragment.children.first() {
                    if let JSXElementChild::JSXElement(jsx) = child {
                        return Ok(*jsx.clone());
                    }
                }
                return Err(RawHtmlError::InvalidFragment);
            }
        }

        Err(RawHtmlError::NoJsxElement)
    }

    /// Attempts to parse the HTML content and returns the first child element
    /// if successful
    pub fn parse_first_child(&self) -> Result<JSXElement, RawHtmlError> {
        let element = self.parse()?;
        if let Some(child) = element.children.first() {
            if let JSXElementChild::JSXElement(jsx) = child {
                return Ok(*jsx.clone());
            }
        }
        Err(RawHtmlError::NoJsxElement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let html = r#"<div>Hello World</div>"#;
        let raw = RawHtml::new(html);
        let result = raw.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_nested_html() {
        let html = r#"<div><span>Nested</span></div>"#;
        let raw = RawHtml::new(html);
        let result = raw.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_html() {
        let raw = RawHtml::new("");
        let result = raw.parse();
        assert!(matches!(result, Err(RawHtmlError::EmptyContent)));
    }

    #[test]
    fn test_invalid_html() {
        let html = r#"<div>Unclosed"#;
        let raw = RawHtml::new(html);
        let result = raw.parse();
        assert!(matches!(result, Err(RawHtmlError::ParseError(_))));
    }

  
}
