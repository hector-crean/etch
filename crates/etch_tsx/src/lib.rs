use color_eyre::eyre;
use log::info;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_ast::{Expr, JSXAttrName, JSXAttrValue, JSXElement, JSXElementName, Lit};
use swc_ecma_parser::{lexer::Lexer, Parser as SwcParser, StringInput, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitWith};
use walkdir::WalkDir;

pub struct TransVisitor {
    translations: HashMap<String, String>,
}

impl TransVisitor {
    fn new() -> Self {
        TransVisitor {
            translations: HashMap::new(),
        }
    }
    fn extract_text_content(&mut self, children: &[swc_ecma_ast::JSXElementChild]) -> String {
        children
            .iter()
            .map(|child| match child {
                swc_ecma_ast::JSXElementChild::JSXText(text) => text.value.to_string(),
                swc_ecma_ast::JSXElementChild::JSXExprContainer(container) => {
                    match &container.expr {
                        swc_ecma_ast::JSXExpr::Expr(expr) => {
                            if let Expr::Lit(Lit::Str(str)) = &**expr {
                                str.value.to_string()
                            } else {
                                String::new()
                            }
                        }
                        _ => String::new(),
                    }
                }
                _ => String::new(),
            })
            .collect::<Vec<String>>()
            .join("")
            .split_whitespace()
            // Split by whitespace
            .collect::<Vec<&str>>()
            // Collect into a vector of string slices
            .join(" ") // Join with a single space
            .trim() // Trim any leading or trailing whitespace
            .to_string()
    }
}

impl Visit for TransVisitor {
    fn visit_jsx_element(&mut self, jsx_element: &JSXElement) {
        if let JSXElementName::Ident(ident) = &jsx_element.opening.name {
            if ident.sym == *"Trans" {
                let mut i18n_key = String::new();
                for attr in &jsx_element.opening.attrs {
                    if let swc_ecma_ast::JSXAttrOrSpread::JSXAttr(attr) = attr {
                        if let JSXAttrName::Ident(ident) = &attr.name {
                            if ident.sym == *"i18nKey" {
                                match &attr.value {
                                    Some(JSXAttrValue::Lit(Lit::Str(str))) => {
                                        i18n_key = str.value.to_string();
                                        break;
                                    }
                                    Some(JSXAttrValue::JSXExprContainer(container)) => {
                                        if let swc_ecma_ast::JSXExpr::Expr(expr) = &container.expr {
                                            if let Expr::Lit(Lit::Str(str)) = &**expr {
                                                i18n_key = str.value.to_string();
                                                break;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                if !i18n_key.is_empty() {
                    let content = self.extract_text_content(&jsx_element.children);
                    self.translations.insert(i18n_key, content);
                }
            }
        }
        jsx_element.visit_children_with(self);
    }
}




pub fn parse_tsx_and_export_translations(
    input_dir: &PathBuf,
    output_file: &PathBuf,
) -> eyre::Result<()> {
    let mut all_translations = HashMap::new();
    let walker = WalkDir::new(input_dir).into_iter();
    for entry in walker {
        let entry = entry?;
        info!("Processing file: {:?}", entry.path());
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .map_or(false, |ext| ext == "tsx" || ext == "jsx")
        {
            let cm: Lrc<SourceMap> = Default::default();
            let fm = cm.load_file(path)?;
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
                .map_err(|e| eyre::eyre!("Failed to parse module: {:?}", e))?;
            let mut visitor = TransVisitor::new();
            module.visit_with(&mut visitor);
            all_translations.extend(visitor.translations);
        }
    }
    let json_output = json!(all_translations);
    // Create the directory if it doesn't exist
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }
    // Write to the file, creating it if it doesn't exist
    std::fs::write(output_file, serde_json::to_string_pretty(&json_output)?)?;
    Ok(())
}
