use crate::rc_dom::{Handle, NodeData};
use crate::visitor::NodeVisitor;
use colored::*;
use html5ever::{Attribute, QualName};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Asset {
    path: String,
    element: String,
    attribute: String,
    file: PathBuf,
}

#[derive(Error, Debug)]
pub enum AssetVisitorError {
    #[error("Invalid asset path: {}", .0.to_string().red())]
    InvalidPath(#[from] std::io::Error),
}

pub struct AssetVisitor {
    current_file: PathBuf,
    assets: Vec<Asset>,
}

impl AssetVisitor {
    pub fn new<P: AsRef<Path>>(current_file: P) -> Self {
        Self {
            current_file: current_file.as_ref().to_path_buf(),
            assets: Vec::new(),
        }
    }

    pub fn assets(&self) -> &Vec<Asset> {
        &self.assets
    }

    fn extract(&mut self, path: &str, element_type: &str, attribute: &str) {
        let asset = Asset::extract(path, &self.current_file, element_type, attribute);
        self.assets.push(asset);
    }
}

impl NodeVisitor for AssetVisitor {
    fn visit_element(
        &mut self,
        name: &QualName,
        attrs: &RefCell<Vec<Attribute>>,
        _template_contents: &RefCell<Option<Handle>>,
        _mathml_annotation_xml_integration_point: bool,
        _handle: &Handle,
    ) -> (Option<Handle>, bool) {
        let tag_name = name.local.as_ref();

        // Check attributes based on tag type
        let attrs = attrs.borrow();
        match tag_name {
            "img" => {
                // Check both src and srcset
                for attr in attrs.iter() {
                    match attr.name.local.as_ref() {
                        "src" | "srcset" => {
                            self.extract(&attr.value, "img", attr.name.local.as_ref())
                        }
                        _ => {}
                    }
                }
            }
            "link" => {
                // Only check stylesheet links
                if attrs.iter().any(|attr| {
                    attr.name.local.as_ref() == "rel" && attr.value.to_lowercase() == "stylesheet"
                }) {
                    if let Some(attr) = attrs.iter().find(|attr| attr.name.local.as_ref() == "href")
                    {
                        self.extract(&attr.value, "link", "href");
                    }
                }
            }
            "script" => {
                if let Some(attr) = attrs.iter().find(|attr| attr.name.local.as_ref() == "src") {
                    self.extract(&attr.value, "script", "src");
                }
            }
            "video" | "audio" => {
                for attr in attrs.iter() {
                    match attr.name.local.as_ref() {
                        "src" | "poster" => {
                            self.extract(&attr.value, tag_name, attr.name.local.as_ref())
                        }
                        _ => {}
                    }
                }
            }
            "source" => {
                for attr in attrs.iter() {
                    match attr.name.local.as_ref() {
                        "src" | "srcset" => {
                            self.extract(&attr.value, "source", attr.name.local.as_ref())
                        }
                        _ => {}
                    }
                }
            }
            "object" => {
                if let Some(attr) = attrs.iter().find(|attr| attr.name.local.as_ref() == "data") {
                    self.extract(&attr.value, "object", "data");
                }
            }
            "embed" => {
                if let Some(attr) = attrs.iter().find(|attr| attr.name.local.as_ref() == "src") {
                    self.extract(&attr.value, "embed", "src");
                }
            }
            "track" => {
                if let Some(attr) = attrs.iter().find(|attr| attr.name.local.as_ref() == "src") {
                    self.extract(&attr.value, "track", "src");
                }
            }
            "iframe" => {
                if let Some(attr) = attrs.iter().find(|attr| attr.name.local.as_ref() == "src") {
                    self.extract(&attr.value, "iframe", "src");
                }
            }
            _ => {}
        }

        (None, true)
    }
}

impl Asset {
    fn extract(path: &str, current_file: &Path, element: &str, attribute: &str) -> Self {
        // Get the directory containing the current HTML file
        let parent_dir = current_file.parent().unwrap_or(Path::new(""));
        
        // Generate absolute path based on whether the asset path is absolute or relative
        let absolute_path = if path.starts_with('/') {
            // Absolute path - relative to project root
            PathBuf::from(path)
        } else {
            // Relative path - resolve from the HTML file's directory
            // Normalize the path by removing ./ and handling ../
            let path = path.strip_prefix("./").unwrap_or(path);
            let mut normalized = parent_dir.join(path);
            if let Ok(canonical) = normalized.canonicalize() {
                normalized = canonical;
            }
            normalized
        };

        Asset {
            path: path.to_string(),
            element: element.to_string(),
            attribute: attribute.to_string(),
            file: absolute_path,
        }
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Asset {
            path,
            element,
            attribute,
            file,
        } = self;

        write!(
            f,
            "Asset reference:\n  → '{}'\n\nReferenced in:\n  → <{}> {} attribute\n  → at {}\n",
            path,
            element,
            attribute,
            file.display()
        )
    }
}

#[derive(Debug)]
pub struct UnusedAssetFinder {
    root_dir: PathBuf,
    used_assets: HashSet<PathBuf>,
    asset_extensions: HashSet<String>,
}

impl UnusedAssetFinder {
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        // Common asset extensions - can be made configurable
        let asset_extensions: HashSet<String> = vec![
            "png", "jpg", "jpeg", "gif", "svg", "webp", // Images
            // "css", "scss", "sass",                        // Stylesheets
            // "js", "mjs",                                  // JavaScript
            "mp3", "wav", "ogg", // Audio
            "mp4", "webm", "ogv", // Video
            // "woff", "woff2", "ttf", "eot",              // Fonts
            "pdf", "doc", "docx", // Documents
        ]
        .into_iter()
        .map(String::from)
        .collect();

        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
            used_assets: HashSet::new(),
            asset_extensions,
        }
    }

    pub fn register_used_assets(&mut self, assets: &[Asset]) {
        for asset in assets {
            if let Ok(canonical_path) = asset.file.canonicalize() {
                self.used_assets.insert(canonical_path);
            }
        }
    }

    pub fn find_unused_assets(&self) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut unused_assets = Vec::new();
        self.scan_directory(&self.root_dir, &mut unused_assets)?;
        Ok(unused_assets)
    }

    fn scan_directory(&self, dir: &Path, unused_assets: &mut Vec<PathBuf>) -> std::io::Result<()> {
        WalkDir::new(dir)
            .follow_links(true)
            .min_depth(1) // Skip the root directory itself
            .into_iter()
            // Pre-filter directories and file types
            .filter_entry(|e| {
                let file_type = e.file_type();
                if file_type.is_dir() {
                    let name = e.file_name().to_str().unwrap_or("");
                    !["node_modules", "target", ".git"].contains(&name)
                } else if file_type.is_file() {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| self.asset_extensions.contains(&ext.to_lowercase()))
                        .unwrap_or(false)
                } else {
                    false // Skip symlinks or other special files
                }
            })
            // Process only files that passed the filter
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .for_each(|entry| {
                if let Ok(canonical_path) = entry.path().canonicalize() {
                    if !self.used_assets.contains(&canonical_path) {
                        unused_assets.push(canonical_path);
                    }
                }
            });

        Ok(())
    }

    pub fn delete_unused_assets(&self) -> Result<(), std::io::Error> {
        let unused = self.find_unused_assets()?;
        for asset in unused {
            std::fs::remove_file(asset)?;
        }
        Ok(())
    }

    pub fn get_used_assets(&self) -> Vec<PathBuf> {
        self.used_assets.iter().cloned().collect()
    }
}
