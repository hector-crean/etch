use std::path::Path;
use swc_common::SourceMap;
use swc_common::sync::Lrc;
use swc_ecma_codegen;
use swc_ecma_parser::{Parser as SwcParser, StringInput, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TsxError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("FromUtf8Error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}
