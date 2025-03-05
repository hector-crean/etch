use dotenv::dotenv;
use etch_core::walk::FileWalker;
use etch_tsx::{file::visit_tsx_file_mut, visitor};
use log::info;
use std::collections::HashSet;

const ROOT_DIR: &str = r#"C:\Users\Hector.C\typescript\rvm-101\"#;

fn get_color_name(hex: &str) -> &str {
    let name = match hex.to_lowercase().as_str() {
        "[#3a9b54]" => "forest_green",
        "[#13d377]" => "emerald_green",
        "[#333333]" => "charcoal",
        "[#1e6b5c]" => "deep_teal",
        "[#0e4343]" => "dark_pine",
        "[#70ffe5" => "aqua_mint",
        _ => hex,
    };
    name
}

fn main() {
    dotenv().ok();
    env_logger::init();
    info!("Looking for color theme in: {}", ROOT_DIR);

    let global_colors = HashSet::<String>::new();

    let walker = FileWalker::new(["tsx"]);

    let _ = walker.visit(ROOT_DIR, |path, _| {
        let visitor =
            visitor::extract_color_theme::ColorThemeVisitor::new(|s| get_color_name(s).to_string());

        let (tsx, visitor) = visit_tsx_file_mut(path, visitor)?;

        let colors = visitor.colors();

        std::fs::write(path, tsx)?;

        Ok(())
    });
}
