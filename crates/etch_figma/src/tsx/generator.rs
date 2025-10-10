use std::collections::HashMap;
use std::path::PathBuf;
use swc_ecma_ast::JSXElement;
use crate::codegen_ext::{CodeGenerator, CodeGenResult};
use super::visitor::TsxVisitor;

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
    fn generate_separate_files(&self, visitor: &TsxVisitor, result: &mut CodeGenResult) -> Result<(), Box<dyn std::error::Error>> {
        let root_elements = if self.exportable_only {
            visitor.exportable_root_jsx_elements_by_name()
        } else {
            visitor.root_jsx_elements_by_name()
        };
        
        for (component_name, (node_id, jsx_element)) in root_elements {
            let file_name = format!("{}.tsx", component_name);
            let file_path = PathBuf::from(file_name);
            
            let content = self.generate_component_file(&component_name, &jsx_element)?;
            result.add_file(file_path, content);
        }
        
        Ok(())
    }
    
    /// Generate a single TSX file with all components
    fn generate_single_file(&self, visitor: &TsxVisitor, result: &mut CodeGenResult) -> Result<(), Box<dyn std::error::Error>> {
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
    fn generate_component_file(&self, component_name: &str, jsx_element: &JSXElement) -> Result<String, Box<dyn std::error::Error>> {
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
    fn generate_module_file(&self, root_elements: &HashMap<String, (String, JSXElement)>) -> Result<String, Box<dyn std::error::Error>> {
        let mut content = String::new();
        
        // Add React imports if needed
        if self.include_react_imports {
            content.push_str("import React from 'react';\n\n");
        }
        
        // Generate all component functions
        for (component_name, (_, jsx_element)) in root_elements {
            let component_code = self.generate_component_function(component_name, jsx_element)?;
            content.push_str(&component_code);
            content.push_str("\n\n");
        }
        
        // Generate exports
        content.push_str("// Exports\n");
        for component_name in root_elements.keys() {
            content.push_str(&format!("export {{ {} }};\n", component_name));
        }
        
        Ok(content)
    }
    
    /// Generate a React component function
    fn generate_component_function(&self, component_name: &str, jsx_element: &JSXElement) -> Result<String, Box<dyn std::error::Error>> {
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
    
    /// Convert JSX element to string representation
    fn jsx_element_to_string(&self, jsx_element: &JSXElement) -> Result<String, Box<dyn std::error::Error>> {
        // This is a simplified implementation
        // In a real implementation, you'd use a proper JSX code generator
        let mut content = String::new();
        
        // Get the tag name
        let tag_name = match &jsx_element.opening.name {
            swc_ecma_ast::JSXElementName::Ident(ident) => &ident.sym,
            swc_ecma_ast::JSXElementName::JSXMemberExpr(_) => "div", // fallback
            swc_ecma_ast::JSXElementName::JSXNamespacedName(_) => "div", // fallback
        };
        
        content.push_str(&format!("    <{}", tag_name));
        
        // Add attributes
        for attr in &jsx_element.opening.attrs {
            if let swc_ecma_ast::JSXAttrOrSpread::JSXAttr(attr) = attr {
                if let swc_ecma_ast::JSXAttrName::Ident(name) = &attr.name {
                    if let Some(swc_ecma_ast::JSXAttrValue::Lit(lit)) = &attr.value {
                        if let swc_ecma_ast::Lit::Str(str_lit) = lit {
                            content.push_str(&format!(" {}=\"{}\"", name.sym, str_lit.value));
                        }
                    }
                }
            }
        }
        
        if jsx_element.opening.self_closing {
            content.push_str(" />");
        } else {
            content.push_str(">");
            
            // Add children
            for child in &jsx_element.children {
                match child {
                    swc_ecma_ast::JSXElementChild::JSXElement(child_jsx) => {
                        let child_content = self.jsx_element_to_string(child_jsx)?;
                        content.push_str(&child_content);
                    }
                    swc_ecma_ast::JSXElementChild::JSXText(text) => {
                        content.push_str(&text.value);
                    }
                    _ => {} // Handle other child types as needed
                }
            }
            
            content.push_str(&format!("</{}>", tag_name));
        }
        
        Ok(content)
    }
}
