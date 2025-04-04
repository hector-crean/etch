use dotenv::dotenv;
use etch_core::walk::FileWalker;
use etch_tsx::{file::visit_tsx_file_mut, visitor};
use log::info;
use std::collections::HashSet;

const ROOT_DIR: &str = r#"/Users/hectorcrean/typescript/RVM-2429613-Clinical-Trial-Website/src"#;

fn get_color_name(hex: &str) -> &str {
    
    (match hex.to_lowercase().as_str() {
        "[#3a9b54]" => "forest-green",
        "[#13d377]" => "emerald-green",
        "[#333333]" => "charcoal",
        "[#1e6b5c]" => "deep-teal",
        "[#0e4343]" => "dark-pine",
        "[#70ffe5" => "aqua-mint",
        _ => hex,
    }) as _
}

fn main() {
    dotenv().ok();
    env_logger::init();
    info!("Looking for color theme in: {}", ROOT_DIR);

    let mut global_colors = HashSet::<String>::new();

    let walker = FileWalker::new(["tsx"]);

    let _ = walker.visit(ROOT_DIR, |path, _| {
        let visitor =
            visitor::extract_color_theme::ColorThemeVisitor::new(|s| get_color_name(s).to_string());

        let (tsx, visitor) = visit_tsx_file_mut(path, visitor)?;

        let colors = visitor.colors();
        global_colors.extend(colors.iter().map(|s| s.to_string()));

        std::fs::write(path, tsx)?;

        Ok(())
    });

    println!("Colors: {:?}", global_colors);
}
