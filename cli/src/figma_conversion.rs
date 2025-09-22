use etch_nextjs::*;
use etch_svg::SvgConverter;
use etch_tsx::pipeline::{Pipeline, StatefulPipeline};
use etch_tsx::visitor::filter_visitor::{FilterVisitor, GlowFilterProps};
use etch_tsx::visitor::framer_motion_visitor::{AnimationConfig, FramerMotionVisitor};
use etch_tsx::visitor::inject_callbacks_visitor::Callback;
use etch_tsx::visitor::inject_shadcn_ui_visitor::InjectShadcnUiVisitor;
use etch_tsx::visitor::inject_uuid_visitor::{InjectUuidPolicy, InjectUuidVisitor};
use etch_tsx::visitor::nextjs_visitor::Runtime;
use etch_tsx::visitor::xlink_visitor::XlinkBase64Extractor;
use etch_tsx::visitor::{
  inject_callbacks_visitor::InjectCallbacksVisitor, inject_shadcn_ui_visitor::ComponentWrapper,
  nextjs_visitor::NextjsVisitor,
};
use log::info;
use serde::{Deserialize, Serialize};
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
  pub filters: HashMap<String, GlowFilterProps>
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
    let file_content = std::fs::read_to_string(&path)?;
    let file_tree: Vec<AppRouterEntry<FigmaConversion>> = serde_json::from_str(&file_content)?;

    Ok(Self {
      base_dir: base_dir.as_ref().to_path_buf(),
      file_tree,
    })
  }

  // Helper function to normalize path strings to the current platform's format
  fn normalize_path_string(path_str: &str) -> String {
    if cfg!(windows) {
      // Convert forward slashes to backslashes on Windows
      path_str.replace('/', "\\")
    } else {
      // Keep as is on Unix-like systems
      path_str.to_string()
    }
  }

  pub fn run(&self) -> Result<(), FigmaConversionError> {
    // Start with empty path and build incrementally
    let current_path = PathBuf::new();
    self.process_entries(&self.file_tree, &current_path)
  }

  fn process_entries(
    &self,
    entries: &[AppRouterEntry<FigmaConversion>],
    current_path: &Path,
  ) -> Result<(), FigmaConversionError> {



    for entry in entries.iter() {
      match entry {
        AppRouterEntry::Directory(dir) => {
          // Build the path incrementally by joining current path with this directory's segment
          let dir_path = current_path.join(&dir.path_segment);
          info!("Processing directory: {:?}", dir_path);
          
          // Recursively process children with the updated path
          self.process_entries(&dir.children, &dir_path)?;
        }
        AppRouterEntry::File(file) => {
          // Build the file path incrementally by joining current path with this file's segment
          let file_path = current_path.join(&file.path_segment);
          info!("Processing file: {:?}", file_path);

          // Normalize the source file path string before converting to Path
          let normalized_source_path = Self::normalize_path_string(&file.data.source_file);
          let source_file_path = Path::new(&normalized_source_path);

          info!(
            "Attempting to read from source file: {:?}",
            source_file_path
          );
          let svg = match std::fs::read_to_string(source_file_path) {
            Ok(content) => content,
            Err(e) => {
              info!(
                "Error reading source file: {:?} - {:?}",
                source_file_path, e
              );
              return Err(FigmaConversionError::IoError(e));
            }
          };
          info!(
            "Successfully read SVG from source file: {:?}",
            source_file_path
          );

          let page = SvgConverter::new(&svg).to_react_component("Page").unwrap();
          info!("Converting SVG to React component");

          info!("Built file path: {:?}", file_path);

          // Construct full path for destination file using the incrementally built path
          let dest_file_path = self.base_dir.join(&file_path);

          // Ensure parent directory exists
          if let Some(parent) = dest_file_path.parent() {
            std::fs::create_dir_all(parent)?;
          }

          info!("Writing React component to: {:?}", dest_file_path);
          std::fs::write(&dest_file_path, page)?;

          // First, run the XlinkBase64Extractor separately to get extracted images
          // Create an "assets" subdirectory for assets to match the @/assets import prefix
          let asset_output_dir = self.base_dir.join("assets");
          let xlink_extractor = XlinkBase64Extractor::new_with_file_imports(Some(asset_output_dir), "@/assets/".to_string());
          let xlink_pipeline = StatefulPipeline::new(xlink_extractor);
          let (tsx_with_base64_extracted, extractor) = xlink_pipeline.run(dest_file_path.clone())?;
          
          // Write the intermediate result back to the file for other visitors
          std::fs::write(&dest_file_path, tsx_with_base64_extracted)?;
          
          // Log extracted images info
          if !extractor.images().is_empty() {
            info!("Extracted {} base64 images to assets/ directory", extractor.images().len());
            for image in extractor.images().values() {
              info!("  - {}.{} ({})", image.variable_name, image.file_extension, image.mime_type);
            }
          }

          // Now run the other visitors with the regular pipeline
          let mut pipeline = Pipeline::new();
          pipeline
            .add_visitor(InjectCallbacksVisitor::new(file.data.callbacks.clone()))
            .add_visitor(InjectShadcnUiVisitor::new(
              file.data.component_wrappers.clone(),
              file.data.action_imports.clone(),
            ))
            .add_visitor(FramerMotionVisitor::new(file.data.animations.clone()))
            .add_visitor(FilterVisitor::new(file.data.filters.clone()))
            .add_visitor(InjectUuidVisitor::new(InjectUuidPolicy::KeepExisting))
            .add_visitor(NextjsVisitor::new(Runtime::Client));

          let tsx = pipeline.run(dest_file_path.clone())?;

          info!("Writing final TSX to: {:?}", dest_file_path);
          std::fs::write(&dest_file_path, tsx)?;
        }
      }
    }
    Ok(())
  }
}
