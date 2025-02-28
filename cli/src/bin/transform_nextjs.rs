use etch_core::{walk::FileWalker};
use etch_tsx::{file::visit_tsx_file_mut, visitor};
use log::{info, warn, error};
use env_logger;
use dotenv::dotenv;
use tokio;
use std::collections::{HashMap, HashSet, BTreeMap};
use reqwest;
use url;
use chrono;
use std::io::{self, Write};
use etch_html::visitor::asset_visitor::UnusedAssetFinder;
use std::fs::File;
use std::path::PathBuf;
use color_name::Color;
use palette::{rgb::Rgb, Srgb, Hsl};

const ROOT_DIR: &str = "/Users/hectorcrean/typescript/RVM-2429613-Clinical-Trial-Website";

fn get_color_name(hex: &str) -> &str {
    let name = match hex.to_lowercase().as_str() {
        "#3a9b54" => "forest_green",
        "#13d377" => "emerald_green",
        "#333333" => "charcoal",
        "#1e6b5c" => "deep_teal",
        "#0e4343" => "dark_pine",
        "#70ffe5" => "aqua_mint",
        _ => "unnamed",
    };
    name
}

fn main() {
    dotenv().ok();
    env_logger::init();
    info!("Starting dead link check in directory: {}", ROOT_DIR);


    let mut global_colors = HashSet::<String>::new();

    let walker = FileWalker::new(["tsx"]);

    let _ = walker.visit(ROOT_DIR, |path, _| {
        info!("Processing file: {}", path.display());
        
        let visitor = visitor::extract_color_theme::ColorThemeVisitor::new("className", |s| s.to_string());

        let (tsx, visitor) = visit_tsx_file_mut(path, visitor)?;

        let colors = visitor.colors();


       
        Ok(())
    });



   
}
