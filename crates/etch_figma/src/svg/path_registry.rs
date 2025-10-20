use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Registry for managing SVG path data extraction
/// Generates hashed identifiers for paths and creates external path files
#[derive(Debug, Clone)]
pub struct PathRegistry {
    /// Maps path hash to actual path data
    paths: HashMap<String, String>,
}

impl PathRegistry {
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
        }
    }

    /// Registers a path and returns its hashed identifier
    /// Returns identifier like "p2cf2ca00" similar to Figma Make
    pub fn register_path(&mut self, path_data: &str) -> String {
        // Generate hash from path data
        let hash = self.generate_path_hash(path_data);

        // Check if this exact path already exists
        if self.paths.contains_key(&hash) {
            return hash;
        }

        // Store the path data
        self.paths.insert(hash.clone(), path_data.to_string());
        hash
    }

    /// Checks if a path should be extracted based on complexity
    pub fn should_extract_path(&self, path_data: &str) -> bool {
        // Extract if:
        // - Length is > 100 characters (complex path)
        // - Has many commands (> 5)
        let command_count = path_data.matches(char::is_alphabetic).count();
        let length = path_data.len();

        command_count > 5 || length > 100
    }

    /// Generates a hash identifier for a path (like "p2cf2ca00")
    fn generate_path_hash(&self, path_data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(path_data.as_bytes());
        let result = hasher.finalize();

        // Take first 8 hex chars and prefix with 'p'
        format!(
            "p{:x}",
            u32::from_be_bytes([result[0], result[1], result[2], result[3]])
        )
    }

    /// Generates TypeScript file content with all paths
    /// Format: export default { p2cf2ca00: "M10,10...", ... }
    pub fn generate_ts_file(&self) -> String {
        let mut content = String::from("// Auto-generated SVG path data\n");
        content.push_str("// Do not edit manually\n\n");
        content.push_str("export default {\n");

        let mut sorted_paths: Vec<_> = self.paths.iter().collect();
        sorted_paths.sort_by_key(|(hash, _)| *hash);

        for (hash, path_data) in sorted_paths {
            // Escape quotes in path data
            let escaped = path_data.replace('\\', "\\\\").replace('"', "\\\"");
            content.push_str(&format!("  {}: \"{}\",\n", hash, escaped));
        }

        content.push_str("};\n");
        content
    }

    /// Gets all registered paths
    pub fn paths(&self) -> &HashMap<String, String> {
        &self.paths
    }

    /// Returns true if any paths have been registered
    pub fn has_paths(&self) -> bool {
        !self.paths.is_empty()
    }

    /// Gets the import statement for the path file
    pub fn get_import_statement(&self, relative_path: &str) -> String {
        format!("import svgPaths from \"{}\";\n", relative_path)
    }
}

impl Default for PathRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_path() {
        let mut registry = PathRegistry::new();

        let path1 = "M10,10 L90,90";
        let hash1 = registry.register_path(path1);

        assert!(hash1.starts_with('p'));
        assert_eq!(hash1.len(), 9); // 'p' + 8 hex chars
        assert_eq!(registry.paths().len(), 1);
    }

    #[test]
    fn test_duplicate_paths() {
        let mut registry = PathRegistry::new();

        let path = "M10,10 L90,90";
        let hash1 = registry.register_path(path);
        let hash2 = registry.register_path(path);

        assert_eq!(hash1, hash2);
        assert_eq!(registry.paths().len(), 1);
    }

    #[test]
    fn test_should_extract_path() {
        let registry = PathRegistry::new();

        // Simple path - should not extract
        let simple = "M0,0 L10,10";
        assert!(!registry.should_extract_path(simple));

        // Complex path - should extract (long)
        let complex = "M10,10 L20,20 L30,30 L40,40 L50,50 L60,60 L70,70 L80,80 L90,90 L100,100 L110,110 L120,120";
        assert!(registry.should_extract_path(complex));

        // Complex path - should extract (many commands)
        let many_commands = "M0,0 L1,1 L2,2 L3,3 L4,4 L5,5 L6,6";
        assert!(registry.should_extract_path(many_commands));
    }

    #[test]
    fn test_generate_ts_file() {
        let mut registry = PathRegistry::new();

        registry.register_path("M10,10 L90,90");
        registry.register_path("M0,0 L100,100");

        let ts_content = registry.generate_ts_file();

        assert!(ts_content.contains("export default {"));
        assert!(ts_content.contains("M10,10 L90,90"));
        assert!(ts_content.contains("M0,0 L100,100"));
        assert!(ts_content.ends_with("};\n"));
    }

    #[test]
    fn test_has_paths() {
        let mut registry = PathRegistry::new();
        assert!(!registry.has_paths());

        registry.register_path("M10,10 L90,90");
        assert!(registry.has_paths());
    }

    #[test]
    fn test_get_import_statement() {
        let registry = PathRegistry::new();
        let import = registry.get_import_statement("./svg-paths");
        assert_eq!(import, "import svgPaths from \"./svg-paths\";\n");
    }
}
