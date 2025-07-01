use std::collections::HashMap;
use std::path::{Path, PathBuf};

use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AssetReference {
    pub original_path: String,
    pub resolved_path: PathBuf,
    pub asset_type: AssetType,
    pub reference_type: ReferenceType,
}

#[derive(Debug, Clone)]
pub enum AssetType {
    Image,
    Video,
    Audio,
    Document,
    Other(String),
}

#[derive(Debug, Clone)]
pub enum ReferenceType {
    ImportStatement,
    JsxAttribute,
    StringLiteral,
}

#[derive(Error, Debug)]
pub enum AssetVisitorError {
    #[error("Invalid asset path: {0}")]
    InvalidPath(String),
    #[error("File system error: {0}")]
    FileSystemError(#[from] std::io::Error),
}

pub struct AssetVisitor {
    /// The current file being processed
    current_file: PathBuf,
    /// Base directory for resolving relative paths
    base_dir: PathBuf,
    /// Target directory where assets should be moved
    target_dir: PathBuf,
    /// Assets found during traversal
    assets: Vec<AssetReference>,
    /// Path mapping from old to new locations
    path_mappings: HashMap<String, String>,
    /// Asset file extensions to track
    asset_extensions: Vec<String>,
}

impl AssetVisitor {
    pub fn new<P: AsRef<Path>>(
        current_file: P,
        base_dir: P,
        target_dir: P,
    ) -> Self {
        Self {
            current_file: current_file.as_ref().to_path_buf(),
            base_dir: base_dir.as_ref().to_path_buf(),
            target_dir: target_dir.as_ref().to_path_buf(),
            assets: Vec::new(),
            path_mappings: HashMap::new(),
            asset_extensions: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "svg".to_string(),
                "webp".to_string(),
                "ico".to_string(),
                "mp4".to_string(),
                "webm".to_string(),
                "ogg".to_string(),
                "mp3".to_string(),
                "wav".to_string(),
                "pdf".to_string(),
                "doc".to_string(),
                "docx".to_string(),
            ],
        }
    }

    pub fn assets(&self) -> &Vec<AssetReference> {
        &self.assets
    }

    pub fn path_mappings(&self) -> &HashMap<String, String> {
        &self.path_mappings
    }

    /// Add a path mapping for asset relocation
    pub fn add_path_mapping(&mut self, old_path: String, new_path: String) {
        self.path_mappings.insert(old_path, new_path);
    }

    /// Check if a path is an asset based on its extension
    fn is_asset_path(&self, path: &str) -> bool {
        if let Some(extension) = Path::new(path).extension() {
            if let Some(ext_str) = extension.to_str() {
                return self.asset_extensions.contains(&ext_str.to_lowercase());
            }
        }
        false
    }

    /// Resolve TypeScript path aliases (like @/ -> src/)
    fn resolve_ts_path_alias(&self, path: &str) -> Option<String> {
        // Handle common TypeScript/Vite path aliases
        if path.starts_with("@/") {
            // @/ typically maps to src/ with baseUrl: "./src"
            // Strip the @/ and return the rest
            Some(path.strip_prefix("@/").unwrap_or("").to_string())
        } else {
            None
        }
    }

    /// Resolve a path relative to the current file
    fn resolve_path(&self, path: &str) -> PathBuf {
        // First, try to resolve TypeScript path aliases
        let resolved_alias = if let Some(alias_resolved) = self.resolve_ts_path_alias(path) {
            alias_resolved
        } else {
            path.to_string()
        };

        if resolved_alias.starts_with('/') {
            // Absolute path - in web apps, this typically means public directory
            // Try multiple resolution strategies for absolute paths
            self.resolve_absolute_path(&resolved_alias)
        } else if resolved_alias.contains('/') || resolved_alias.contains('\\') {
            // Relative path with directory structure
            // For TypeScript aliases, resolve from base_dir (typically src/)
            if self.resolve_ts_path_alias(path).is_some() {
                self.base_dir.join(&resolved_alias)
            } else {
                let parent_dir = self.current_file.parent().unwrap_or(&self.base_dir);
                parent_dir.join(&resolved_alias)
            }
        } else {
            // Just a filename - try multiple resolution strategies
            self.resolve_filename(&resolved_alias)
        }
    }

    /// Resolve an absolute path (starting with /) by trying common web app patterns
    fn resolve_absolute_path(&self, path: &str) -> PathBuf {
        let path_without_slash = path.strip_prefix('/').unwrap_or(path);
        
        // Get the project root (parent of base_dir, which is typically src/)
        let project_root = self.base_dir.parent().unwrap_or(&self.base_dir);
        
        // Common locations for absolute web assets
        let potential_locations = [
            project_root.join("public").join(path_without_slash),     // public/asset.jpg
            project_root.join("assets").join(path_without_slash),     // assets/asset.jpg  
            project_root.join("static").join(path_without_slash),     // static/asset.jpg
            project_root.join(path_without_slash),                    // asset.jpg (project root)
            self.base_dir.join(path_without_slash),                   // src/asset.jpg (fallback)
        ];
        
        // Try each location and return the first that exists
        for potential_path in &potential_locations {
            if potential_path.exists() {
                return potential_path.clone();
            }
        }
        
        // If nothing found, default to public directory (most common for absolute paths)
        project_root.join("public").join(path_without_slash)
    }

    /// Resolve a bare filename by trying common asset directory patterns
    fn resolve_filename(&self, filename: &str) -> PathBuf {
        let parent_dir = self.current_file.parent().unwrap_or(&self.base_dir);
        let project_root = self.base_dir.parent().unwrap_or(&self.base_dir);
        
        // Project-level directories (always resolved from project root)
        let project_level_dirs = [
            "",                    // Project root
            "public",              // public/
            "public/assets",       // public/assets/
            "assets",              // assets/ (project root)
            "static",              // static/
        ];
        
        // Relative directories (resolved from various base directories)
        let relative_dirs = [
            "",                    // Same directory as the file
            "assets",              // ./assets/
            "assets/images",       // ./assets/images/
            "assets/videos",       // ./assets/videos/
            "assets/audio",        // ./assets/audio/
            "images",              // ./images/
            "videos",              // ./videos/
            "audio",               // ./audio/
            "media",               // ./media/
            "../assets",           // ../assets/
            "../public/assets",    // ../public/assets/
        ];
        
        // First, try project-level directories from project root
        for asset_dir in &project_level_dirs {
            let potential_path = if asset_dir.is_empty() {
                project_root.join(filename)
            } else {
                project_root.join(asset_dir).join(filename)
            };
            
            if potential_path.exists() {
                return potential_path;
            }
        }
        
        // Then try relative directories from different base directories
        let base_dirs = [
            parent_dir,            // Directory of current file
            &self.base_dir,        // Base directory (typically src/)
        ];
        
        for base_dir in &base_dirs {
            for asset_dir in &relative_dirs {
                let potential_path = if asset_dir.is_empty() {
                    base_dir.join(filename)
                } else {
                    base_dir.join(asset_dir).join(filename)
                };
                
                if potential_path.exists() {
                    return potential_path;
                }
            }
        }
        
        // If nothing found, default to public directory (common for web assets)
        project_root.join("public").join(filename)
    }

    /// Get the asset type based on file extension
    fn get_asset_type(&self, path: &str) -> AssetType {
        if let Some(extension) = Path::new(path).extension() {
            if let Some(ext_str) = extension.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" => AssetType::Image,
                    "mp4" | "webm" | "ogg" => AssetType::Video,
                    "mp3" | "wav" => AssetType::Audio,
                    "pdf" | "doc" | "docx" => AssetType::Document,
                    ext => AssetType::Other(ext.to_string()),
                }
            } else {
                AssetType::Other("unknown".to_string())
            }
        } else {
            AssetType::Other("no_extension".to_string())
        }
    }

    /// Extract asset reference and add to collection
    fn extract_asset(&mut self, path: &str, reference_type: ReferenceType) {
        if self.is_asset_path(path) {
            let resolved_path = self.resolve_path(path);
            let asset_type = self.get_asset_type(path);
            
            let asset_ref = AssetReference {
                original_path: path.to_string(),
                resolved_path,
                asset_type,
                reference_type,
            };
            
            self.assets.push(asset_ref);
        }
    }

    /// Update a string literal if it matches an asset path
    fn update_string_literal(&mut self, lit: &mut Str) {
        let path = lit.value.to_string();
        if let Some(new_path) = self.path_mappings.get(&path) {
            lit.value = new_path.clone().into();
        }
    }
}

impl VisitMut for AssetVisitor {
    /// Visit import declarations to find asset imports
    fn visit_mut_import_decl(&mut self, node: &mut ImportDecl) {
        // Visit children first
        node.visit_mut_children_with(self);

        // Check the import source
        let import_path = node.src.value.to_string();
        self.extract_asset(&import_path, ReferenceType::ImportStatement);
        
        // Update the path if we have a mapping
        if let Some(new_path) = self.path_mappings.get(&import_path) {
            node.src.value = new_path.clone().into();
        }
    }

    /// Visit JSX attributes to find asset references
    fn visit_mut_jsx_attr(&mut self, node: &mut JSXAttr) {
        // Visit children first
        node.visit_mut_children_with(self);

        // Check if this is an attribute that commonly references assets
        if let JSXAttrName::Ident(ident) = &node.name {
            let attr_name = ident.sym.as_ref();
            if matches!(attr_name, 
                "src" | "href" | "poster" | "srcset" | "content" |
                "data-bg-image" | "data-icon" | "data-image" | "data-video" | 
                "data-audio" | "data-background" | "data-poster" |
                "backgroundImage" | "background" | "icon" | "image" |
                "url" | "path" | "file" | "media"
            ) {
                if let Some(JSXAttrValue::Lit(Lit::Str(str_lit))) = &mut node.value {
                    let path = str_lit.value.to_string();
                    self.extract_asset(&path, ReferenceType::JsxAttribute);
                    self.update_string_literal(str_lit);
                }
            }
        }
    }

    /// Visit string literals to find asset references
    fn visit_mut_str(&mut self, node: &mut Str) {
        let path = node.value.to_string();
        
        // Check if it's an asset based on extension
        if self.is_asset_path(&path) {
            self.extract_asset(&path, ReferenceType::StringLiteral);
        }
        
        self.update_string_literal(node);
    }

    /// Visit template literals to find asset references
    fn visit_mut_tpl(&mut self, node: &mut Tpl) {
        // Visit children first
        node.visit_mut_children_with(self);

        // Check each quasi (string part) of the template literal
        for quasi in &mut node.quasis {
            let raw = quasi.raw.to_string();
            if self.is_asset_path(&raw) {
                self.extract_asset(&raw, ReferenceType::StringLiteral);
                
                // Update the template quasi if we have a mapping
                if let Some(new_path) = self.path_mappings.get(&raw) {
                    quasi.raw = new_path.clone().into();
                    quasi.cooked = Some(new_path.clone().into());
                }
            }
        }
    }
}

impl AssetReference {
    /// Generate a new path in the target directory
    pub fn generate_target_path(&self, target_dir: &Path) -> PathBuf {
        if let Some(filename) = self.resolved_path.file_name() {
            target_dir.join(filename)
        } else {
            target_dir.join("unknown_asset")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_structure() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();
        
        // Create directory structure
        let src_dir = project_root.join("src");
        let public_dir = project_root.join("public");
        let assets_dir = src_dir.join("assets");
        let data_dir = src_dir.join("data");
        
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&public_dir).unwrap();
        fs::create_dir_all(&assets_dir).unwrap();
        fs::create_dir_all(&data_dir).unwrap();
        
        // Create test assets
        fs::write(assets_dir.join("icon.svg"), "<svg></svg>").unwrap();
        fs::write(public_dir.join("video.mp4"), "fake video").unwrap();
        fs::write(public_dir.join("poster.jpg"), "fake image").unwrap();
        
        (temp_dir, project_root, src_dir)
    }

    #[test]
    fn test_asset_extraction_from_imports() {
        let (_temp_dir, project_root, src_dir) = create_test_structure();
        let test_file = src_dir.join("test.tsx");
        let target_dir = project_root.join("target");
        
        let mut visitor = AssetVisitor::new(&test_file, &src_dir, &target_dir);
        
        // Simulate import declaration
        visitor.extract_asset("@/assets/icon.svg", ReferenceType::ImportStatement);
        visitor.extract_asset("/video.mp4", ReferenceType::ImportStatement);
        
        let assets = visitor.assets();
        assert_eq!(assets.len(), 2);
        
        // Check TypeScript alias resolution
        let ts_asset = &assets[0];
        assert_eq!(ts_asset.original_path, "@/assets/icon.svg");
        assert!(ts_asset.resolved_path.ends_with("src/assets/icon.svg"));
        
        // Check public path resolution
        let public_asset = &assets[1];
        assert_eq!(public_asset.original_path, "/video.mp4");
        assert!(public_asset.resolved_path.ends_with("public/video.mp4"));
    }

    #[test]
    fn test_asset_extraction_from_data_files() {
        let (_temp_dir, project_root, src_dir) = create_test_structure();
        let test_file = src_dir.join("data/landing.tsx");
        let target_dir = project_root.join("target");
        
        let mut visitor = AssetVisitor::new(&test_file, &src_dir, &target_dir);
        
        // Simulate bare filename references (common in data files)
        visitor.extract_asset("poster.jpg", ReferenceType::StringLiteral);
        visitor.extract_asset("video.mp4", ReferenceType::StringLiteral);
        
        let assets = visitor.assets();
        assert_eq!(assets.len(), 2);
        
        // Both should resolve to public directory
        for asset in assets {
            assert!(asset.resolved_path.to_string_lossy().contains("public"));
        }
    }

    #[test]
    fn test_path_mapping_updates() {
        let (_temp_dir, project_root, src_dir) = create_test_structure();
        let test_file = src_dir.join("test.tsx");
        let target_dir = project_root.join("target");
        
        let mut visitor = AssetVisitor::new(&test_file, &src_dir, &target_dir);
        
        // Add path mappings
        visitor.add_path_mapping("@/assets/icon.svg".to_string(), "/locales/en/icon.svg".to_string());
        visitor.add_path_mapping("poster.jpg".to_string(), "/locales/en/poster.jpg".to_string());
        
        // Test string literal update
        let mut str_node = swc_ecma_ast::Str {
            span: swc_common::DUMMY_SP,
            value: "@/assets/icon.svg".into(),
            raw: None,
        };
        
        visitor.update_string_literal(&mut str_node);
        assert_eq!(str_node.value.to_string(), "/locales/en/icon.svg");
        
        // Test another mapping
        let mut str_node2 = swc_ecma_ast::Str {
            span: swc_common::DUMMY_SP,
            value: "poster.jpg".into(),
            raw: None,
        };
        
        visitor.update_string_literal(&mut str_node2);
        assert_eq!(str_node2.value.to_string(), "/locales/en/poster.jpg");
    }

    #[test]
    fn test_asset_type_detection() {
        let (_temp_dir, project_root, src_dir) = create_test_structure();
        let test_file = src_dir.join("test.tsx");
        let target_dir = project_root.join("target");
        
        let visitor = AssetVisitor::new(&test_file, &src_dir, &target_dir);
        
        assert!(matches!(visitor.get_asset_type("image.jpg"), AssetType::Image));
        assert!(matches!(visitor.get_asset_type("video.mp4"), AssetType::Video));
        assert!(matches!(visitor.get_asset_type("audio.mp3"), AssetType::Audio));
        assert!(matches!(visitor.get_asset_type("doc.pdf"), AssetType::Document));
        assert!(matches!(visitor.get_asset_type("unknown.xyz"), AssetType::Other(_)));
    }

    #[test]
    fn test_typescript_path_alias_resolution() {
        let (_temp_dir, project_root, src_dir) = create_test_structure();
        let test_file = src_dir.join("test.tsx");
        let target_dir = project_root.join("target");
        
        let visitor = AssetVisitor::new(&test_file, &src_dir, &target_dir);
        
        // Test @/ alias
        assert_eq!(
            visitor.resolve_ts_path_alias("@/assets/icon.svg"),
            Some("assets/icon.svg".to_string())
        );
        
        // Test non-alias path
        assert_eq!(
            visitor.resolve_ts_path_alias("./local/file.jpg"),
            None
        );
    }

    #[test]
    fn test_is_asset_path() {
        let (_temp_dir, project_root, src_dir) = create_test_structure();
        let test_file = src_dir.join("test.tsx");
        let target_dir = project_root.join("target");
        
        let visitor = AssetVisitor::new(&test_file, &src_dir, &target_dir);
        
        // Should detect various asset patterns
        assert!(visitor.is_asset_path("image.jpg"));
        assert!(visitor.is_asset_path("@/assets/icon.svg"));
        assert!(visitor.is_asset_path("/public/video.mp4"));
        assert!(visitor.is_asset_path("./media/audio.mp3"));
        
        // Should not detect non-assets
        assert!(!visitor.is_asset_path("component"));
        assert!(!visitor.is_asset_path("some text"));
        assert!(!visitor.is_asset_path("http://example.com"));
    }

    #[test]
    fn test_generate_target_path() {
        let (_temp_dir, project_root, _src_dir) = create_test_structure();
        
        let asset = AssetReference {
            original_path: "poster.jpg".to_string(),
            resolved_path: project_root.join("public/poster.jpg"),
            asset_type: AssetType::Image,
            reference_type: ReferenceType::StringLiteral,
        };
        
        let target_dir = project_root.join("public/locales/en");
        let target_path = asset.generate_target_path(&target_dir);
        
        assert_eq!(target_path, target_dir.join("poster.jpg"));
    }

    #[test]
    fn test_update_string_literal_directly() {
        let (_temp_dir, project_root, src_dir) = create_test_structure();
        let test_file = src_dir.join("test.tsx");
        let target_dir = project_root.join("target");
        
        let mut visitor = AssetVisitor::new(&test_file, &src_dir, &target_dir);
        
        // Add path mapping
        visitor.add_path_mapping("video.mp4".to_string(), "/locales/en/video.mp4".to_string());
        
        // Test the update_string_literal method directly
        let mut str_node = swc_ecma_ast::Str {
            span: swc_common::DUMMY_SP,
            value: "video.mp4".into(),
            raw: None,
        };
        
        visitor.update_string_literal(&mut str_node);
        assert_eq!(str_node.value.to_string(), "/locales/en/video.mp4");
    }
} 