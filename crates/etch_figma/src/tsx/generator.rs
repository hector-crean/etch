use super::visitor::TsxVisitor;
use crate::codegen_ext::{CodeGenResult, CodeGenerator};
use heck::ToPascalCase;
use std::collections::HashMap;
use std::path::PathBuf;
use swc_common::{SourceMap, sync::Lrc};
use swc_ecma_ast::JSXElement;
use swc_ecma_codegen::{Config, Emitter, text_writer::JsWriter};

/// Sanitize a Figma node name to a valid React component name
/// Converts names like "flow/burden/page" to "FlowBurdenPage"
fn sanitize_component_name(name: &str) -> String {
    // Replace slashes and other separators with spaces so heck can handle them
    let normalized = name.replace(['/', '.'], " ");

    // Use heck to convert to PascalCase
    let pascal = normalized.to_pascal_case();

    // Handle edge cases
    if pascal.is_empty() {
        return "Component".to_string();
    }

    // Ensure it doesn't start with a number (React component names can't)
    if pascal.chars().next().unwrap().is_numeric() {
        format!("Component{}", pascal)
    } else {
        pascal
    }
}

/// TSX code generator that creates React components from TsxVisitor results
pub struct TsxGenerator {
    /// Whether to include React imports
    include_react_imports: bool,
    /// Whether to create separate files for each component
    separate_files: bool,
    /// Whether to only generate exportable components
    exportable_only: bool,
}

impl TsxGenerator {
    pub fn new() -> Self {
        Self {
            include_react_imports: true,
            separate_files: true,
            exportable_only: false,
        }
    }

    pub fn with_react_imports(mut self, include: bool) -> Self {
        self.include_react_imports = include;
        self
    }

    pub fn with_separate_files(mut self, separate: bool) -> Self {
        self.separate_files = separate;
        self
    }

    pub fn with_exportable_only(mut self, exportable_only: bool) -> Self {
        self.exportable_only = exportable_only;
        self
    }
}

impl CodeGenerator<TsxVisitor> for TsxGenerator {
    fn generate(&self, visitor: &TsxVisitor) -> Result<CodeGenResult, Box<dyn std::error::Error>> {
        let mut result = CodeGenResult::new();

        if self.separate_files {
            self.generate_separate_files(visitor, &mut result)?;
        } else {
            self.generate_single_file(visitor, &mut result)?;
        }

        Ok(result)
    }

    fn file_extension(&self) -> &str {
        "tsx"
    }

    fn output_directory(&self) -> &str {
        "tsx"
    }
}

impl TsxGenerator {
    /// Generate separate TSX files for each component
    fn generate_separate_files(
        &self,
        visitor: &TsxVisitor,
        result: &mut CodeGenResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root_elements = if self.exportable_only {
            visitor.exportable_root_jsx_elements_by_name()
        } else {
            visitor.root_jsx_elements_by_name()
        };

        for (component_name, (_node_id, jsx_element)) in root_elements {
            // Keep original component name for file path
            let file_name = format!("{}.tsx", component_name);
            let file_path = PathBuf::from(file_name);

            // Sanitize the component name only for the React component identifier
            let sanitized_name = sanitize_component_name(&component_name);
            let content = self.generate_component_file(&sanitized_name, &jsx_element)?;
            result.add_file(file_path, content);
        }

        Ok(())
    }

    /// Generate a single TSX file with all components
    fn generate_single_file(
        &self,
        visitor: &TsxVisitor,
        result: &mut CodeGenResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root_elements = if self.exportable_only {
            visitor.exportable_root_jsx_elements_by_name()
        } else {
            visitor.root_jsx_elements_by_name()
        };

        let file_path = PathBuf::from("components.tsx");
        let content = self.generate_module_file(&root_elements)?;
        result.add_file(file_path, content);

        Ok(())
    }

    /// Generate a single component file
    fn generate_component_file(
        &self,
        component_name: &str,
        jsx_element: &JSXElement,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut content = String::new();

        // Add React imports if needed
        if self.include_react_imports {
            content.push_str("import React from 'react';\n\n");
        }

        // Generate the component function
        let component_code = self.generate_component_function(component_name, jsx_element)?;
        content.push_str(&component_code);

        // Add export
        content.push_str(&format!("\nexport default {};\n", component_name));

        Ok(content)
    }

    /// Generate a module file with all components
    fn generate_module_file(
        &self,
        root_elements: &HashMap<String, (String, JSXElement)>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut content = String::new();

        // Add React imports if needed
        if self.include_react_imports {
            content.push_str("import React from 'react';\n\n");
        }

        // Generate all component functions
        let mut sanitized_names = Vec::new();
        for (component_name, (_, jsx_element)) in root_elements {
            let sanitized_name = sanitize_component_name(component_name);
            let component_code = self.generate_component_function(&sanitized_name, jsx_element)?;
            content.push_str(&component_code);
            content.push_str("\n\n");
            sanitized_names.push(sanitized_name);
        }

        // Generate exports
        content.push_str("// Exports\n");
        for sanitized_name in sanitized_names {
            content.push_str(&format!("export {{ {} }};\n", sanitized_name));
        }

        Ok(content)
    }

    /// Generate a React component function
    fn generate_component_function(
        &self,
        component_name: &str,
        jsx_element: &JSXElement,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut content = String::new();

        // Function signature
        content.push_str(&format!("export function {}() {{\n", component_name));
        content.push_str("  return (\n");

        // Generate JSX content
        let jsx_content = self.jsx_element_to_string(jsx_element)?;
        content.push_str(&jsx_content);

        content.push_str("\n  );\n");
        content.push_str("}\n");

        Ok(content)
    }

    /// Convert JSX element to string using SWC's proper codegen
    fn jsx_element_to_string(
        &self,
        jsx_element: &JSXElement,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use swc_common::DUMMY_SP;
        use swc_ecma_ast::{Expr, ExprStmt, Module, ModuleItem, Stmt};

        // Create a minimal expression with just the JSX for codegen
        let jsx_expr = Expr::JSXElement(Box::new(jsx_element.clone()));

        // Wrap in a module with an expression statement for codegen
        let module = Module {
            span: DUMMY_SP,
            body: vec![ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                span: DUMMY_SP,
                expr: Box::new(jsx_expr),
            }))],
            shebang: None,
        };

        // Use SWC's codegen to emit the module
        let source_map = Lrc::new(SourceMap::default());
        let mut buf = vec![];

        {
            let writer = JsWriter::new(source_map.clone(), "\n", &mut buf, None);
            let mut emitter = Emitter {
                cfg: Config::default().with_minify(false),
                cm: source_map.clone(),
                comments: None,
                wr: writer,
            };

            // Emit the entire module
            emitter
                .emit_module(&module)
                .map_err(|e| format!("Failed to emit JSX: {:?}", e))?;
        }

        // Extract just the JSX part (remove the wrapping expression statement semicolon)
        let mut code = String::from_utf8(buf)?;
        // Remove trailing semicolon and whitespace
        code = code.trim().trim_end_matches(';').to_string();
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_component_name() {
        // Test with slashes
        assert_eq!(
            sanitize_component_name("flow/burden/burden_of_bronchiectasis/comorbidities/page"),
            "FlowBurdenBurdenOfBronchiectasisComorbiditiesPage"
        );

        // Test with hyphens
        assert_eq!(
            sanitize_component_name("my-component-name"),
            "MyComponentName"
        );

        // Test with spaces
        assert_eq!(
            sanitize_component_name("My Component Name"),
            "MyComponentName"
        );

        // Test with underscores
        assert_eq!(
            sanitize_component_name("my_component_name"),
            "MyComponentName"
        );

        // Test with mixed separators
        assert_eq!(
            sanitize_component_name("my-component/sub_component.final"),
            "MyComponentSubComponentFinal"
        );

        // Test with numbers
        assert_eq!(
            sanitize_component_name("component-123-test"),
            "Component123Test"
        );

        // Test starting with number
        assert_eq!(
            sanitize_component_name("123-component"),
            "Component123Component"
        );

        // Test with special characters (they become word separators)
        assert_eq!(
            sanitize_component_name("my@component#name!"),
            "MyComponentName"
        );

        // Test empty string
        assert_eq!(sanitize_component_name(""), "Component");

        // Test single word
        assert_eq!(sanitize_component_name("component"), "Component");

        // Test with acronyms (heck converts to proper PascalCase)
        assert_eq!(sanitize_component_name("API-Component"), "ApiComponent");
    }
}
