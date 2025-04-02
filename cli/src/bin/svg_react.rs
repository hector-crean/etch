use dotenv::dotenv;
use etch_core::walk::FileWalker;
use etch_svg::SvgConverter;
use etch_tsx::visitor::svg_react_visitor::{
    Action, Callback, CloseDropdownOptions, CloseModalOptions, CloseSheetOptions, ComponentWrapper,
    DialogOptions, DrawerOptions, Event, HoverCardOptions, LinkOptions, OpenDropdownOptions,
    OpenModalOptions, OpenSheetOptions, PopoverOptions, SelectTabOptions, SheetOptions,
    ShowToastOptions, ToggleAccordionOptions, ToggleModalOptions, TooltipOptions,
};
use etch_tsx::{file::visit_tsx_file_mut, visitor};
use log::info;
use serde_json::json;
use std::{collections::HashSet, path::Path};

const SVG_ROOT_DIR: &str = r#"/Users/hectorcrean/rust/etch/figma-app/figma-export"#;
const TSX_ROOT_DIR: &str = r#"/Users/hectorcrean/rust/etch/figma-app/src/app"#;

fn main() {
    dotenv().ok();
    env_logger::init();

    let walker = FileWalker::new(["svg"]);

    let _ = walker.visit(SVG_ROOT_DIR, |path, relative_path| {
        let svg = std::fs::read_to_string(path)?;

        let converter = SvgConverter::new(&svg);
        let page = converter.to_react_component("Page").unwrap();

        let file_stem = relative_path.file_stem().unwrap_or_default();

        let new_relative_path = relative_path
            .parent()
            .unwrap()
            .join(format!("{}", file_stem.to_string_lossy()));

        let new_path = Path::new(TSX_ROOT_DIR)
            .join(new_relative_path)
            .join("page.tsx");

        // Create parent directories if they don't exist
        if let Some(parent) = new_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(new_path.clone(), page)?;

        let mut visitor = visitor::svg_react_visitor::FigmaExportVisitor::new();

        // 1. Modal Actions
        visitor.register_component_wrapper(
            "modal-button".to_string(),
            ComponentWrapper::Dialog(DialogOptions {
                id: "modal-1".to_string(),
                title: Some("Modal Title".to_string()),
                description: Some("Modal Description".to_string()),
                content: Some("Modal Content".to_string()),
                has_footer: Some(true),
                footer_buttons: Some(vec![]),
            }),
        );

        visitor.register_component_wrapper(
            "hover-card-button".to_string(),
            ComponentWrapper::HoverCard(HoverCardOptions {
                id: "hover-card-1".to_string(),
                trigger_id: None,
                title: Some("Hover Card Title".to_string()),
                description: Some("Hover Card Description".to_string()),
                content: Some("Hover Card Content".to_string()),
                open_delay: Some(100),
                close_delay: Some(100),
            }),
        );

        visitor.register_component_wrapper(
            "link-button".to_string(),
            ComponentWrapper::Link(LinkOptions {
                id: "link-1".to_string(),
                href: "https://www.google.com".to_string(),
                target: Some("_blank".to_string()),
                rel: Some("noopener noreferrer".to_string()),
                as_button: Some(true),
                variant: Some("default".to_string()),
                size: Some("default".to_string()),
            }),
        );

        // 2. Sheet/Drawer Actions
        visitor.register_component_wrapper(
            "popover-button".to_string(),
            ComponentWrapper::Popover(PopoverOptions {
                id: "popover-1".to_string(),
                trigger_id: None,
                title: Some("Popover Title".to_string()),
                description: Some("Popover Description".to_string()),
                content: Some("Popover Content".to_string()),
                alignment: Some("bottom".to_string()),
            }),
        );

        visitor.register_component_wrapper(
            "sheet-button".to_string(),
            ComponentWrapper::Sheet(SheetOptions {
                id: "sheet-1".to_string(),
                trigger_id: None,
                title: Some("Sheet Title".to_string()),
                description: Some("Sheet Description".to_string()),
                content: Some("Sheet Content".to_string()),
                side: Some("left".to_string()),
                has_footer: Some(true),
                footer_buttons: Some(vec![]),
            }),
        );

        // 3. Toast Notifications
        visitor.register_callback("notification-bell".to_string(), Callback {
            trigger: Event::Click,
            action: Action::Toast(ShowToastOptions {
                message: "New notification".to_string(),
            }),
        });

        visitor.register_component_wrapper(
            "tooltip-button".to_string(),
            ComponentWrapper::Tooltip(TooltipOptions {
                id: "tooltip-1".to_string(),
                trigger_id: None,
                content: "Tooltip Content".to_string(),
                side: Some("bottom".to_string()),
                align: Some("center".to_string()),
                delay_duration: Some(100),
                skip_delay_duration: Some(100),
            }),
        );

        visitor.register_component_wrapper(
            "drawer-button".to_string(),
            ComponentWrapper::Drawer(DrawerOptions {
                id: "dropdown-trigger".to_string(),
                title: Some("Drawer Title".to_string()),
                description: Some("Drawer Description".to_string()),
            }),
        );

        let (mut tsx, visitor) = visit_tsx_file_mut(new_path.clone(), visitor)?;

        // Add "use client"; directive and import Button component
        tsx = format!(
            "\"use client\";\n\nimport {{ Button }} from \"@/components/ui/button\";\n\n{}", 
            tsx
        );

        std::fs::write(new_path.clone(), tsx)?;

        // Format the TSX file using Prettier
        format_tsx_file(&new_path)?;

        Ok(())
    });
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
