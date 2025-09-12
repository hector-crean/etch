use swc_ecma_visit::{VisitMut, VisitMutWith};
use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};
use swc_common::{DUMMY_SP, SyntaxContext};
use swc_ecma_ast::{
    Expr, Ident, ImportDecl, ImportDefaultSpecifier, ImportNamedSpecifier, ImportSpecifier, 
    JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXElement, JSXExpr, JSXExprContainer, Lit, 
    Module, ModuleDecl, ModuleItem, Str,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base64Image {
    pub id: Uuid,
    pub mime_type: String,
    pub data: String,
    pub variable_name: String,
    pub file_extension: String,
}

pub struct XlinkBase64Extractor {
    pub extracted_images: HashMap<Uuid, Base64Image>,
    pub output_dir: Option<PathBuf>,
    pub generate_files: bool,
    pub constants_import_path: String,
    pub use_file_imports: bool,
    pub import_prefix: String,
}

impl XlinkBase64Extractor {
    pub fn new(output_dir: Option<PathBuf>, generate_files: bool) -> Self {
        Self::new_with_import_path(output_dir, generate_files, "./base64_images".to_string())
    }

    pub fn new_with_import_path(output_dir: Option<PathBuf>, generate_files: bool, constants_import_path: String) -> Self {
        Self {
            extracted_images: HashMap::new(),
            output_dir,
            generate_files,
            constants_import_path,
            use_file_imports: false,
            import_prefix: "@/app/".to_string(),
        }
    }

    /// Create a new extractor that generates file imports instead of constants
    pub fn new_with_file_imports(output_dir: Option<PathBuf>, import_prefix: String) -> Self {
        Self {
            extracted_images: HashMap::new(),
            output_dir,
            generate_files: true, // Always generate files when using file imports
            constants_import_path: String::new(), // Not used for file imports
            use_file_imports: true,
            import_prefix,
        }
    }

    pub fn images(&self) -> &HashMap<Uuid, Base64Image> {
        &self.extracted_images
    }

    /// Extract base64 data from a data URI
    fn extract_base64_data(&self, data_uri: &str) -> Option<(String, String, String)> {
        if !data_uri.starts_with("data:") {
            return None;
        }

        // Parse data URI format: data:[<mediatype>][;base64],<data>
        let parts: Vec<&str> = data_uri.splitn(2, ',').collect();
        if parts.len() != 2 {
            return None;
        }

        let header = parts[0];
        let data = parts[1];

        if !header.contains("base64") {
            return None;
        }

        // Extract MIME type
        let mime_type = if let Some(semicolon_pos) = header.find(';') {
            header[5..semicolon_pos].to_string() // Skip "data:"
        } else {
            "application/octet-stream".to_string()
        };

        // Determine file extension from MIME type
        let file_extension = match mime_type.as_str() {
            "image/png" => "png",
            "image/jpeg" | "image/jpg" => "jpg",
            "image/gif" => "gif",
            "image/svg+xml" => "svg",
            "image/webp" => "webp",
            _ => "bin", // fallback for unknown types
        };

        Some((mime_type, data.to_string(), file_extension.to_string()))
    }

    /// Generate a TypeScript variable name from UUID
    fn generate_variable_name(uuid: &Uuid) -> String {
        format!("base64Image_{}", uuid.simple())
    }

    /// Write base64 data to file if generate_files is true
    fn write_base64_to_file(&self, base64_data: &str, file_path: &PathBuf) -> Result<(), String> {
        if !self.generate_files {
            return Ok(());
        }

        // Create directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Decode base64 and write to file
        let decoded = general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| format!("Failed to decode base64: {}", e))?;

        fs::write(file_path, decoded)
            .map_err(|e| format!("Failed to write file {}: {}", file_path.display(), e))?;

        Ok(())
    }

    /// Process a data URI and return the replacement variable name
    fn process_data_uri(&mut self, data_uri: &str) -> Option<String> {
        if let Some((mime_type, base64_data, file_extension)) = self.extract_base64_data(data_uri) {
            let uuid = Uuid::new_v4();
            let variable_name = Self::generate_variable_name(&uuid);

            // Write to file if requested
            if let Some(output_dir) = &self.output_dir {
                let file_path = output_dir.join(format!("{}.{}", uuid.simple(), file_extension));
                if let Err(e) = self.write_base64_to_file(&base64_data, &file_path) {
                    log::warn!("Failed to write base64 image file: {}", e);
                }
            }

            let image = Base64Image {
                id: uuid,
                mime_type,
                data: base64_data,
                variable_name: variable_name.clone(),
                file_extension,
            };

            self.extracted_images.insert(uuid, image);
            Some(variable_name)
        } else {
            None
        }
    }

    /// Generate TypeScript variable declarations for all extracted images
    pub fn generate_typescript_declarations(&self) -> String {
        let mut declarations = String::new();
        declarations.push_str("// Auto-generated base64 image constants\n");
        declarations.push_str("// These constants replace inline base64 data for better readability\n\n");

        for image in self.extracted_images.values() {
            declarations.push_str(&format!(
                "export const {} = \"data:{};base64,{}\";\n",
                image.variable_name, image.mime_type, image.data
            ));
        }

        declarations
    }

    /// Add import statements for base64 constants or file imports to the top of the module
    pub fn add_import_statement(&self, module: &mut Module) {
        if self.extracted_images.is_empty() {
            return;
        }

        if self.use_file_imports {
            // Generate individual file imports for each image
            let mut import_decls = Vec::new();
            let mut sorted_images: Vec<_> = self.extracted_images.values().collect();
            sorted_images.sort_by(|a, b| a.variable_name.cmp(&b.variable_name));

            for image in sorted_images {
                let import_path = format!("{}{}.{}", 
                    self.import_prefix, 
                    image.id.simple(), 
                    image.file_extension
                );

                let import_decl = ImportDecl {
                    span: DUMMY_SP,
                    specifiers: vec![ImportSpecifier::Default(ImportDefaultSpecifier {
                        span: DUMMY_SP,
                        local: Ident {
                            span: DUMMY_SP,
                            sym: image.variable_name.clone().into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        },
                    })],
                    src: Box::new(Str {
                        span: DUMMY_SP,
                        value: import_path.into(),
                        raw: None,
                    }),
                    type_only: false,
                    with: None,
                    phase: Default::default(),
                };

                import_decls.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)));
            }

            // Insert all imports at the beginning of the module
            for (i, import_decl) in import_decls.into_iter().enumerate() {
                module.body.insert(i, import_decl);
            }
        } else {
            // Original behavior: generate named imports from constants file
            let mut variable_names: Vec<String> = self.extracted_images
                .values()
                .map(|img| img.variable_name.clone())
                .collect();
            variable_names.sort(); // Sort for consistent output

            // Create import specifiers
            let import_specifiers: Vec<ImportSpecifier> = variable_names
                .into_iter()
                .map(|name| {
                    ImportSpecifier::Named(ImportNamedSpecifier {
                        span: DUMMY_SP,
                        local: Ident {
                            span: DUMMY_SP,
                            sym: name.into(),
                            optional: false,
                            ctxt: SyntaxContext::empty(),
                        },
                        imported: None,
                        is_type_only: false,
                    })
                })
                .collect();

            // Create the import declaration
            let import_decl = ImportDecl {
                span: DUMMY_SP,
                specifiers: import_specifiers,
                src: Box::new(Str {
                    span: DUMMY_SP,
                    value: self.constants_import_path.clone().into(),
                    raw: None,
                }),
                type_only: false,
                with: None,
                phase: Default::default(),
            };

            // Insert at the beginning of the module
            module.body.insert(
                0,
                ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)),
            );
        }
    }
}

impl VisitMut for XlinkBase64Extractor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        // First, visit all elements to extract base64 data
        module.visit_mut_children_with(self);
        
        // After processing, add import statement if we found any base64 images
        if !self.extracted_images.is_empty() {
            self.add_import_statement(module);
        }
    }

    fn visit_mut_jsx_element(&mut self, element: &mut JSXElement) {
        // Visit children first
        element.visit_mut_children_with(self);
        // Check if this is an SVG element or child that might have xlink:href
        for attr in &mut element.opening.attrs {
            if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                // Check for xlinkHref or href attributes
                let attr_name = match &jsx_attr.name {
                    JSXAttrName::Ident(ident) => ident.sym.as_str(),
                    JSXAttrName::JSXNamespacedName(namespaced) => {
                        if namespaced.ns.sym.as_str() == "xlink"
                            && namespaced.name.sym.as_str() == "href"
                        {
                            "xlinkHref"
                        } else {
                            continue;
                        }
                    }
                };

                if attr_name == "xlinkHref" || attr_name == "href" {
                    if let Some(JSXAttrValue::Lit(Lit::Str(str_lit))) = &jsx_attr.value {
                        let value = str_lit.value.as_str();
                        if value.starts_with("data:") && value.contains("base64") {
                            // Extract the base64 data and replace with variable reference
                            if let Some(variable_name) = self.process_data_uri(value) {
                                // Replace the string literal with a JSX expression containing the variable
                                jsx_attr.value = Some(JSXAttrValue::JSXExprContainer(
                                    JSXExprContainer {
                                        span: DUMMY_SP,
                                        expr: JSXExpr::Expr(Box::new(Expr::Ident(Ident {
                                            span: DUMMY_SP,
                                            sym: variable_name.clone().into(),
                                            optional: false,
                                            ctxt: SyntaxContext::empty(),
                                        }))),
                                    },
                                ));
                                log::info!("Replaced base64 data URI with variable reference: {}", variable_name);
                            }
                        }
                    }
                }
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base64_data() {
        let extractor = XlinkBase64Extractor::new(None, false);
        
        // Test PNG data URI
        let png_uri = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        let result = extractor.extract_base64_data(png_uri);
        
        assert!(result.is_some());
        let (mime_type, data, extension) = result.unwrap();
        assert_eq!(mime_type, "image/png");
        assert_eq!(extension, "png");
        assert!(!data.is_empty());
    }

    #[test]
    fn test_generate_variable_name() {
        let uuid = Uuid::new_v4();
        let var_name = XlinkBase64Extractor::generate_variable_name(&uuid);
        assert!(var_name.starts_with("base64Image_"));
        assert_eq!(var_name.len(), "base64Image_".len() + 32); // UUID simple format is 32 chars
    }

    #[test]
    fn test_non_base64_uri_ignored() {
        let extractor = XlinkBase64Extractor::new(None, false);
        let regular_uri = "https://example.com/image.png";
        let result = extractor.extract_base64_data(regular_uri);
        assert!(result.is_none());
    }
}
