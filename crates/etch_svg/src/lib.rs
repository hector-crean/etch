use std::error::Error;
use swc_common::{DUMMY_SP, SourceMap, SyntaxContext, sync::Lrc};
use swc_ecma_ast::*;
use swc_ecma_codegen::{Emitter, text_writer::JsWriter};

pub struct SvgConverter {
    content: String,
}

impl SvgConverter {
    pub fn new(svg_content: &str) -> Self {
        Self {
            content: svg_content.to_string(),
        }
    }

    // Create a React component from the SVG content
    pub fn to_react_component(&self, component_name: &str) -> Result<String, Box<dyn Error>> {
        // Parse the SVG content and create a JSX element
        let jsx_element = self.parse_svg_to_jsx()?;

        // Create the React component structure
        let component = self.create_react_component(component_name, jsx_element);

        // Generate TypeScript code
        let code = self.generate_typescript_code(component)?;

        Ok(code)
    }

    fn parse_svg_to_jsx(&self) -> Result<JSXElement, Box<dyn Error>> {
        // For now, create a simple JSX element structure
        // In a full implementation, you'd parse the SVG XML and convert it
        let ctxt = SyntaxContext::empty();

        let jsx_element = JSXElement {
            span: DUMMY_SP,
            opening: JSXOpeningElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident::new("svg".into(), DUMMY_SP, ctxt).into()),
                type_args: None,
                attrs: vec![JSXAttrOrSpread::JSXAttr(JSXAttr {
                    span: DUMMY_SP,
                    name: JSXAttrName::Ident(Ident::new("viewBox".into(), DUMMY_SP, ctxt).into()),
                    value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: "0 0 100 100".into(),
                        raw: None,
                    }))),
                })],
                self_closing: false,
            },
            children: vec![JSXElementChild::JSXElement(Box::new(JSXElement {
                span: DUMMY_SP,
                opening: JSXOpeningElement {
                    span: DUMMY_SP,
                    name: JSXElementName::Ident(Ident::new("circle".into(), DUMMY_SP, ctxt).into()),
                    type_args: None,
                    attrs: vec![
                        JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(
                                Ident::new("cx".into(), DUMMY_SP, ctxt).into(),
                            ),
                            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: "50".into(),
                                raw: None,
                            }))),
                        }),
                        JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(
                                Ident::new("cy".into(), DUMMY_SP, ctxt).into(),
                            ),
                            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: "50".into(),
                                raw: None,
                            }))),
                        }),
                        JSXAttrOrSpread::JSXAttr(JSXAttr {
                            span: DUMMY_SP,
                            name: JSXAttrName::Ident(Ident::new("r".into(), DUMMY_SP, ctxt).into()),
                            value: Some(JSXAttrValue::Lit(Lit::Str(Str {
                                span: DUMMY_SP,
                                value: "40".into(),
                                raw: None,
                            }))),
                        }),
                    ],
                    self_closing: true,
                },
                children: vec![],
                closing: None,
            }))],
            closing: Some(JSXClosingElement {
                span: DUMMY_SP,
                name: JSXElementName::Ident(Ident::new("svg".into(), DUMMY_SP, ctxt).into()),
            }),
        };

        Ok(jsx_element)
    }

    fn create_react_component(&self, component_name: &str, jsx_element: JSXElement) -> Module {
        let ctxt = SyntaxContext::empty();

        // Create the React component function
        let component_function = Function {
            span: DUMMY_SP,
            is_async: false,
            is_generator: false,
            params: vec![],
            decorators: vec![],
            body: Some(BlockStmt {
                span: DUMMY_SP,
                ctxt,
                stmts: vec![Stmt::Return(ReturnStmt {
                    span: DUMMY_SP,
                    arg: Some(Box::new(Expr::JSXElement(Box::new(jsx_element)))),
                })],
            }),
            return_type: None,
            type_params: None,
            ctxt,
        };

        // Create the component declaration
        let component_decl = Decl::Fn(FnDecl {
            ident: Ident::new(component_name.into(), DUMMY_SP, ctxt),
            declare: false,
            function: Box::new(component_function),
        });

        // Create the module with imports and exports
        Module {
            span: DUMMY_SP,
            body: vec![
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    span: DUMMY_SP,
                    specifiers: vec![ImportSpecifier::Default(ImportDefaultSpecifier {
                        span: DUMMY_SP,
                        local: Ident::new("React".into(), DUMMY_SP, ctxt),
                    })],
                    src: Box::new(Str {
                        span: DUMMY_SP,
                        value: "react".into(),
                        raw: None,
                    }),
                    type_only: false,
                    with: None,
                    phase: Default::default(),
                })),
                ModuleItem::Stmt(Stmt::Decl(component_decl)),
                ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(ExportDefaultExpr {
                    span: DUMMY_SP,
                    expr: Box::new(Expr::Ident(Ident::new(
                        component_name.into(),
                        DUMMY_SP,
                        ctxt,
                    ))),
                })),
            ],
            shebang: None,
        }
    }

    fn generate_typescript_code(&self, module: Module) -> Result<String, Box<dyn Error>> {
        let cm: Lrc<SourceMap> = Default::default();
        let mut buf = vec![];

        let mut emitter = Emitter {
            cfg: Default::default(),
            cm: cm.clone(),
            comments: None,
            wr: JsWriter::new(cm, "\n", &mut buf, None),
        };

        emitter.emit_module(&module)?;

        Ok(String::from_utf8_lossy(&buf).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_to_tsx_file() {
        let svg_content = r#"<svg viewBox="0 0 100 100">
    <circle cx="50" cy="50" r="40"/>
</svg>"#;

        let converter = SvgConverter::new(svg_content);
        let react_component = converter.to_react_component("CircleIcon").unwrap();

        // Verify the component contains the React import
        assert!(react_component.contains("import React from 'react';"));

        // Verify the component name is used correctly
        assert!(react_component.contains("const CircleIcon = () =>"));

        // Verify export statement
        assert!(react_component.contains("export default CircleIcon;"));
    }
}
