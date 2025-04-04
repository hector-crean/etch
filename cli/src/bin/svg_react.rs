use cli::figma_conversion::{FigmaConversionError, Project};
use dotenv::dotenv;
use etch_core::walk::FileWalker;
use etch_nextjs::*;
use etch_svg::SvgConverter;
use etch_tsx::pipeline::Pipeline;
use etch_tsx::visitor::framer_motion_visitor::{AnimationConfig, FramerMotionVisitor};
use etch_tsx::visitor::nextjs_visitor::Runtime;
use etch_tsx::visitor::wrapper_and_callback_visitor::AddWrappersAndCallbacksVisitor;
use etch_tsx::visitor::{
    nextjs_visitor::NextjsVisitor,
    wrapper_and_callback_visitor::{
        Action, Callback, CloseDropdownOptions, CloseModalOptions, CloseSheetOptions,
        ComponentWrapper, DialogOptions, DrawerOptions, Event, HoverCardOptions, LinkOptions,
        OpenDropdownOptions, OpenModalOptions, OpenSheetOptions, PopoverOptions, SelectTabOptions,
        SheetOptions, ShowToastOptions, ToggleAccordionOptions, ToggleModalOptions, TooltipOptions,
    },
};
use etch_tsx::{file::visit_tsx_file_mut, visitor};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{collections::HashSet, path::Path};
use ts_rs::TS;

fn main() -> Result<(), FigmaConversionError> {
    dotenv().ok();
    env_logger::init();

    let base_dir = r#"C:\Users\Hector.C\rust\etch\figma-app\src\app\(pages)"#;

    let file_tree_path = r#"C:\Users\Hector.C\rust\etch\figma-app\src\file-tree.json"#;
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
