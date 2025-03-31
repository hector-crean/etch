use dotenv::dotenv;
use etch_core::walk::FileWalker;
use etch_tsx::{file::visit_tsx_file_mut, visitor};
use log::info;
use std::collections::HashSet;

const ROOT_DIR: &str = r#"/Users/hectorcrean/typescript/RVM-2429613-Clinical-Trial-Website/src"#;

fn get_color_name(hex: &str) -> &str {
    let name = match hex.to_lowercase().as_str() {
        "[#3a9b54]" => "forest-green",
        "[#13d377]" => "emerald-green",
        "[#333333]" => "charcoal",
        "[#1e6b5c]" => "deep-teal",
        "[#0e4343]" => "dark-pine",
        "[#70ffe5" => "aqua-mint",
        _ => hex,
    };
    name
}

fn main() {
    dotenv().ok();
    env_logger::init();
    info!("Looking for color theme in: {}", ROOT_DIR);

    let mut global_colors = HashSet::<String>::new();

    let walker = FileWalker::new(["tsx"]);

    let _ = walker.visit(ROOT_DIR, |path, _| {



        let visitor = visitor::svg_react_visitor::SvgReactVisitor::new(tsx, "SvgComponent".to_string(), true);
        let (tsx, visitor) = visit_tsx_file_mut(path, visitor)?;

        std::fs::write(path, tsx)?;

        Ok(())
    });

    println!("Colors: {:?}", global_colors);
}
