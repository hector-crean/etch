use dotenv::dotenv;
use etch_core::walk::FileWalker;
use etch_nextjs::*;
use etch_svg::SvgConverter;
use etch_tsx::visitor::svg_react_visitor::{
    Action, Callback, CloseDropdownOptions, CloseModalOptions, CloseSheetOptions, ComponentWrapper,
    DialogOptions, DrawerOptions, Event, FigmaExportVisitor, HoverCardOptions, LinkOptions,
    OpenDropdownOptions, OpenModalOptions, OpenSheetOptions, PopoverOptions, SelectTabOptions,
    SheetOptions, ShowToastOptions, ToggleAccordionOptions, ToggleModalOptions, TooltipOptions,
};
use etch_tsx::{file::visit_tsx_file_mut, visitor};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::{collections::HashSet, path::Path};


#[derive(thiserror::Error, Debug)]
pub enum FigmaConversionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),
    #[error(transparent)]
    AppRouterError(#[from] etch_nextjs::AppRouterError),
    #[error(transparent)]
    TsxError(#[from] etch_tsx::file::TsxError),
}

pub struct Project {
    base_dir: PathBuf,
    file_tree: Vec<AppRouterEntry<FigmaExportVisitor>>,
}

impl Project {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir, file_tree: vec![] }
    }
    pub fn from_file<D: AsRef<Path>, P: AsRef<Path>>(base_dir: D, path: P) -> Result<Self, FigmaConversionError> {
        let file_content = std::fs::read_to_string(path)?;
        let file_tree: Vec<AppRouterEntry<FigmaExportVisitor>> =
            serde_json::from_str(&file_content)?;

        Ok(Self { base_dir: base_dir.as_ref().to_path_buf(), file_tree })
    }
    pub fn run(&self) -> Result<(), FigmaConversionError> {
        self.process_entries(&self.file_tree)
    }
    
    fn process_entries(&self, entries: &[AppRouterEntry<FigmaExportVisitor>]) -> Result<(), FigmaConversionError> {
        for entry in entries.iter() {
            match entry {
                AppRouterEntry::Directory(dir) => {
                    // Recursively process children of this directory
                    info!("Processing directory: {:?}", dir.relative_path);
                    self.process_entries(&dir.children)?;
                }
                AppRouterEntry::File(file) => {
                    info!("Processing file: {:?}", file.relative_path);

                    // Construct full path for source file
                    let source_file_path = &file.data.source_file;
                    let svg = std::fs::read_to_string(&source_file_path)?;
                    info!("Read SVG from source file: {:?}", source_file_path);
                    
                    let page = SvgConverter::new(&svg).to_react_component("Page").unwrap();
                    info!("Converting SVG to React component");
                    
                    // Construct full path for destination file
                    let dest_file_path = self.base_dir.join(&file.relative_path);
                    
                    // Ensure parent directory exists
                    if let Some(parent) = dest_file_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    
                    info!("Writing React component to: {:?}", dest_file_path);
                    std::fs::write(&dest_file_path, page)?;
                    
                    let mut visitor = FigmaExportVisitor::new(dest_file_path.clone());

                    // Log component wrappers and callbacks registration
                    info!("Registering {} component wrappers", file.data.component_wrappers.len());
                    for (id, wrapper) in file.data.component_wrappers.iter() {
                        visitor.register_component_wrapper(id.clone(), wrapper.clone());
                    }

                    info!("Registering callbacks for {} elements", file.data.callbacks.len());
                    for (id, callbacks) in file.data.callbacks.iter() {
                        for callback in callbacks.iter() {
                            visitor.register_callback(id.clone(), callback.clone());
                        }
                    }
                    
                    info!("Applying TSX visitor to file: {:?}", dest_file_path);
                    let (mut tsx, visitor) = visit_tsx_file_mut(dest_file_path.clone(), visitor)?;

                    // Add "use client"; directive and import Button component
                    tsx = format!(
                        "\"use client\";\n\nimport {{ Button }} from \"@/components/ui/button\";\n\n{}",
                        tsx
                    );
            
                    info!("Writing final TSX to: {:?}", dest_file_path);
                    std::fs::write(&dest_file_path, tsx)?;
            
                    // Format the TSX file using Prettier
                    format_tsx_file(&dest_file_path)?;
                }
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), FigmaConversionError> {
    dotenv().ok();
    env_logger::init();

    let base_dir = "/Users/hectorcrean/rust/etch/figma-app/src/app/(pages)";

    let file_tree_path = "/Users/hectorcrean/rust/etch/figma-app/src/file-tree.json";
    info!("Loading project from file: {}", file_tree_path);
    
    let project = Project::from_file(&base_dir, file_tree_path)?;
    info!("Project loaded with {} entries", project.file_tree.len());
    
    info!("Starting project conversion...");
    project.run()?;
    info!("Project conversion completed successfully");

    Ok(())
}

/// Format a TypeScript/TSX file using Prettier
fn format_tsx_file(path: &Path) -> std::io::Result<()> {
    use std::process::Command;

    let output = Command::new("npx")
        .args(["prettier", "--write", path.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error formatting TSX file: {}", error);
    }

    Ok(())
}


