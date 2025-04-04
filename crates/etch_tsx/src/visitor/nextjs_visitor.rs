use swc_common::DUMMY_SP;
use swc_ecma_ast::{Expr, ExprStmt, Lit, Module, ModuleItem, Stmt, Str};
use swc_ecma_visit::VisitMut;

pub enum Runtime {
    Server,
    Client,
}

pub struct NextjsVisitor {
    runtime: Runtime,
}

impl NextjsVisitor {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl VisitMut for NextjsVisitor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        // Create the directive based on the runtime
        let directive_text = match self.runtime {
            Runtime::Server => "use server",
            Runtime::Client => "use client",
        };

        // Check if the directive already exists
        let has_directive = module.body.iter().any(|item| {
            if let ModuleItem::Stmt(Stmt::Expr(ExprStmt { expr, .. })) = item {
                if let Expr::Lit(Lit::Str(Str { value, .. })) = &**expr {
                    return value.to_string() == directive_text;
                }
            }
            false
        });

        // Add the directive if it doesn't exist
        if !has_directive {
            let directive = ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                span: DUMMY_SP,
                expr: Box::new(Expr::Lit(Lit::Str(Str {
                    span: DUMMY_SP,
                    value: directive_text.into(),
                    raw: None,
                }))),
            }));

            // Insert the directive at the beginning of the module
            module.body.insert(0, directive);
        }

        // Continue visiting the rest of the module
        // swc_ecma_visit::visit_mut_module(self, module);
    }
}
