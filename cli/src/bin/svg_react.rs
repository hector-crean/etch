use dotenv::dotenv;
use etch_core::walk::FileWalker;
use etch_svg::SvgParser;
use etch_tsx::visitor::svg_react_visitor::{
    Action, Callback, Event, 
    OpenModalOptions, CloseModalOptions, ToggleModalOptions,
    OpenSheetOptions, CloseSheetOptions,
    ShowToastOptions, SelectTabOptions, ToggleAccordionOptions,
    OpenDropdownOptions, CloseDropdownOptions
};
use etch_tsx::{file::visit_tsx_file_mut, visitor};
use log::info;
use std::{collections::HashSet, path::Path};
use serde_json::json;


const ROOT_DIR: &str = r#"/Users/hectorcrean/rust/etch/example-svgs"#;

fn main() {
    dotenv().ok();
    env_logger::init();

    let walker = FileWalker::new(["svg"]);

    let _ = walker.visit(ROOT_DIR, |path, _| {
        let svg = std::fs::read_to_string(path)?;

        let parser = SvgParser::new(&svg);
        let react_component = parser.to_react_component("Page").unwrap();

        let file_stem = path.file_stem().unwrap_or_default();

        let new_path = path
            .parent()
            .unwrap()
            .join(format!("{}/page.tsx", file_stem.to_string_lossy()));

        std::fs::write(new_path.clone(), react_component)?;

        let mut visitor = visitor::svg_react_visitor::FigmaExportVisitor::new();

        // 1. Modal Actions
        visitor.register_callback(
            "blue-circle".to_string(), 
            Callback {
                trigger: Event::Click,
            action: Action::OpenModal(OpenModalOptions {
                    id: "modal-1".to_string(),
                }),
            }
        );
        
        visitor.register_callback(
            "close-button".to_string(), 
            Callback {
                trigger: Event::Click,
                action: Action::CloseModal(CloseModalOptions {
                    id: "modal-1".to_string(),
                }),
            }
        );
        
        visitor.register_callback(
            "toggle-button".to_string(), 
            Callback {
                trigger: Event::Click,
                action: Action::ToggleModal(ToggleModalOptions {
                    id: "modal-1".to_string(),
                }),
            }
        );
        
        // 2. Sheet/Drawer Actions
        visitor.register_callback(
            "menu-icon".to_string(), 
            Callback {
                trigger: Event::Click,
                action: Action::OpenSheet(OpenSheetOptions {
                    id: "side-menu".to_string(),
                    side: Some("left".to_string()),
                }),
            }
        );
        
        visitor.register_callback(
            "sheet-close".to_string(), 
            Callback {
                trigger: Event::Click,
                action: Action::CloseSheet(CloseSheetOptions {
                    id: "side-menu".to_string(),
                }),
            }
        );
        
        // 3. Toast Notifications
        visitor.register_callback(
            "notification-bell".to_string(), 
            Callback {
                trigger: Event::Click,
                action: Action::ShowToast(ShowToastOptions {
                    title: "New notification".to_string(),
                    description: Some("You have a new message".to_string()),
                    variant: Some("default".to_string()),
                }),
            }
        );
        
        visitor.register_callback(
            "error-icon".to_string(), 
            Callback {
                trigger: Event::Click,
                action: Action::ShowToast(ShowToastOptions {
                    title: "Error occurred".to_string(),
                    description: Some("Please try again later".to_string()),
                    variant: Some("destructive".to_string()),
                }),
            }
        );
        
        // 4. Tab Selection
        visitor.register_callback(
            "tab-item-1".to_string(), 
            Callback {
                trigger: Event::Click,
                action: Action::SelectTab(SelectTabOptions {
                    tabGroupId: "main-tabs".to_string(),
                    tabId: "tab1".to_string(),
                }),
            }
        );
        
        // 5. Accordion Toggle
        visitor.register_callback(
            "faq-item".to_string(), 
            Callback {
                trigger: Event::Click,
                action: Action::ToggleAccordion(ToggleAccordionOptions {
                    id: "faq-1".to_string(),
                }),
            }
        );
        
        // 6. Dropdown Actions
        visitor.register_callback(
            "dropdown-trigger".to_string(), 
            Callback {
                trigger: Event::Click,
                action: Action::OpenDropdown(OpenDropdownOptions {
                    id: "user-menu".to_string(),
                }),
            }
        );
        
        visitor.register_callback(
            "dropdown-close".to_string(), 
            Callback {
                trigger: Event::MouseLeave,
                action: Action::CloseDropdown(CloseDropdownOptions {
                    id: "user-menu".to_string(),
                }),
            }
        );
        
        // Different event types demonstration
        visitor.register_callback(
            "hover-area".to_string(), 
            Callback {
                trigger: Event::MouseEnter,
                action: Action::ShowToast(ShowToastOptions {
                    title: "Hovered!".to_string(),
                    description: None,
                    variant: None,
                }),
            }
        );
        
        visitor.register_callback(
            "input-field".to_string(), 
            Callback {
                trigger: Event::Focus,
                action: Action::OpenDropdown(OpenDropdownOptions {
                    id: "suggestions".to_string(),
                }),
            }
        );

        let (tsx, visitor) = visit_tsx_file_mut(new_path.clone(), visitor)?;

        std::fs::write(new_path.clone(), tsx)?;

        Ok(())
    });
}
