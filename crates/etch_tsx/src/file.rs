use std::path::Path;
use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_codegen;
use swc_ecma_parser::{lexer::Lexer, Parser as SwcParser, StringInput, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TsxError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub fn parse_tsx_file<P: AsRef<Path>>(
    file_path: P,
) -> Result<(Lrc<SourceMap>, swc_ecma_ast::Module), TsxError> {
    // Set up the parser
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.load_file(file_path.as_ref())?;

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

    // Parse the module
    let module = parser
        .parse_module()
        .map_err(|e| TsxError::ParseError(format!("{:?}", e)))?;

    Ok((cm, module))
}

pub fn visit_tsx_file<P: AsRef<Path>, V: Visit>(
    file_path: P,
    mut visitor: V,
) -> Result<(String, V), TsxError> {
    let (cm, module) = parse_tsx_file(file_path)?;

    // Run the visitor
    module.visit_with(&mut visitor);

    // Get the original source code
    let source_code = cm.lookup_source_file(module.span.lo).src.to_string();

    Ok((source_code, visitor))
}

pub fn visit_tsx_file_mut<P: AsRef<Path>, V: VisitMut>(
    file_path: P,
    mut visitor: V,
) -> Result<(String, V), TsxError> {
    let (_, mut module) = parse_tsx_file(file_path)?;

    // Run the mutable visitor
    module.visit_mut_with(&mut visitor);

    // Generate the updated source code from the modified AST
    let updated_source = swc_ecma_codegen::to_code(&module);

    Ok((updated_source, visitor))
}
