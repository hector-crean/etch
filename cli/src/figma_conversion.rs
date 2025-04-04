use dotenv::dotenv;
use etch_core::walk::FileWalker;
use etch_nextjs::*;
use etch_svg::SvgConverter;
use etch_tsx::pipeline::Pipeline;
use etch_tsx::visitor::framer_motion_visitor::{AnimationConfig, FramerMotionVisitor};
use etch_tsx::visitor::inject_shadcn_ui_visitor::InjectShadcnUiVisitor;
use etch_tsx::visitor::nextjs_visitor::Runtime;
use etch_tsx::visitor::{
    inject_callbacks_visitor::InjectCallbacksVisitor,
    inject_shadcn_ui_visitor::{
        CloseDropdownOptions, CloseModalOptions, CloseSheetOptions, ComponentWrapper,
        DialogOptions, DrawerOptions, HoverCardOptions, LinkOptions, OpenDropdownOptions,
        OpenModalOptions, OpenSheetOptions, PopoverOptions, SelectTabOptions, SheetOptions,
        ShowToastOptions, ToggleAccordionOptions, ToggleModalOptions, TooltipOptions,
    },
    nextjs_visitor::NextjsVisitor,
};
use etch_tsx::{file::visit_tsx_file_mut, visitor};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{collections::HashSet, path::Path};
use ts_rs::TS;

#[derive(thiserror::Error, Debug)]
pub enum FigmaConversionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ParseError(#[from] serde_json::Error),
    #[error(transparent)]
    AppRouterError(#[from] etch_nextjs::AppRouterError),
    #[error(transparent)]
    TsxError(#[from] etch_tsx::error::TsxError),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export)]
pub struct FigmaConversion {
    pub source_file: String,
    pub callbacks: HashMap<String, Vec<Callback>>,
    pub component_wrappers: HashMap<String, ComponentWrapper>, // Generic wrapper mapping
    pub action_imports: HashMap<String, HashSet<String>>,
    pub animations: HashMap<String, AnimationConfig>,
}

pub struct Project {
    pub base_dir: PathBuf,
    pub file_tree: Vec<AppRouterEntry<FigmaConversion>>,
}

impl Project {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            file_tree: vec![],
        }
    }
    pub fn from_file<D: AsRef<Path>, P: AsRef<Path>>(
        base_dir: D,
        path: P,
    ) -> Result<Self, FigmaConversionError> {
        let file_content = std::fs::read_to_string(path)?;
        let file_tree: Vec<AppRouterEntry<FigmaConversion>> = serde_json::from_str(&file_content)?;

        Ok(Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            file_tree,
        })
    }
    pub fn run(&self) -> Result<(), FigmaConversionError> {
        self.process_entries(&self.file_tree)
    }

    fn process_entries(
        &self,
        entries: &[AppRouterEntry<FigmaConversion>],
    ) -> Result<(), FigmaConversionError> {
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

                    let relative_path = Path::new(&file.relative_path);

                    info!("Relative path: {:?}", relative_path);

                    // Construct full path for destination file
                    let dest_file_path = self.base_dir.join(&relative_path);

                    // Ensure parent directory exists
                    if let Some(parent) = dest_file_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }

                    info!("Writing React component to: {:?}", dest_file_path);
                    std::fs::write(&dest_file_path, page)?;

                    let mut pipeline = Pipeline::new();

                    pipeline
                        .add_visitor(InjectCallbacksVisitor::new(file.data.callbacks.clone()))
                        .add_visitor(InjectShadcnUiVisitor::new(
                            file.data.component_wrappers.clone(),
                            file.data.action_imports.clone(),
                        ))
                        .add_visitor(FramerMotionVisitor::new(file.data.animations.clone()))
                        .add_visitor(NextjsVisitor::new(Runtime::Client));

                    let tsx = pipeline.run(dest_file_path.clone())?;

                    info!("Writing final TSX to: {:?}", dest_file_path);
                    std::fs::write(&dest_file_path, tsx)?;

                    // Format the TSX file using Prettier
                    // format_tsx_file(&dest_file_path)?;
                }
            }
        }
        Ok(())
    }
}
