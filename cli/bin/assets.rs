use etch_html::{file, visitor::{self}};

use etch_core::walk::FileWalker;
use log::{info, error};
use dotenv::dotenv;
use std::collections::BTreeMap;
use std::io::{self, Write};
use etch_html::visitor::asset_visitor::UnusedAssetFinder;
use std::fs::File;
use std::path::PathBuf;

const ROOT_DIR: &str = "/Users/hectorcrean/typescript/OTS110_WebApp/src/content";

fn organize_assets_by_directory(assets: &[PathBuf]) -> BTreeMap<String, Vec<PathBuf>> {
    let mut organized: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    
    for asset in assets {
        let parent = asset.parent()
            .and_then(|p| p.to_str())
            .unwrap_or("root")
            .to_string();
            
        organized.entry(parent)
            .or_default()
            .push(asset.clone());
    }
    
    organized
}

fn calculate_total_size(assets: &[PathBuf]) -> Result<u64, io::Error> {
    let mut total_size = 0;
    for asset in assets {
        if let Ok(metadata) = std::fs::metadata(asset) {
            total_size += metadata.len();
        }
    }
    Ok(total_size)
}

fn write_asset_section(file: &mut File, title: &str, assets: &[PathBuf], start_index: usize) -> usize {
    let organized = organize_assets_by_directory(assets);
    let mut current_index = start_index;
    
    // Calculate total size for this section
    let total_size = calculate_total_size(assets).unwrap_or(0);
    let size_mb = total_size as f64 / 1_048_576.0; // Convert to MB
    
    // Add emoji based on the section title
    let emoji = if title.contains("Used") { "✅" } else { "❌" };
    let header = format!(
        "\n=== {} {} ===\nTotal count: {}\nTotal size: {:.2} MB\n\n",
        emoji, title, assets.len(), size_mb
    );
    let _ = file.write_all(header.as_bytes());
    
    for (directory, files) in organized.iter() {
        let dir_header = format!("Directory: {}\n", directory);
        let _ = file.write_all(dir_header.as_bytes());
        
        for asset in files {
            let line = format!("{}. {} {}\n", current_index, emoji, asset.display());
            let _ = file.write_all(line.as_bytes());
            current_index += 1;
        }
        let _ = file.write_all(b"\n");
    }
    
    current_index
}

fn main() {
    dotenv().ok();
    env_logger::init();
    info!("Starting dead link check in directory: {}", ROOT_DIR);

    let walker = FileWalker::new(["html"]);

    // Remove file logging setup
    let mut all_assets = Vec::new();

    let _ = walker.visit(ROOT_DIR, |path, _| {
        info!("Processing file: {}", path.display());
        
        let visitor = visitor::AssetVisitor::new(path);
        let (_, visitor) = file::process_html_file(path, visitor)?;
        let assets = visitor.assets();
        all_assets.extend(assets.iter().cloned());
        Ok(())
    });

    // Print assets to terminal instead of file
    for asset in &all_assets {
        info!("{}", asset);
    }

    // Example usage
    let mut finder = UnusedAssetFinder::new(ROOT_DIR);
    finder.register_used_assets(&all_assets);

    // Find unused assets and generate report
    match finder.find_unused_assets() {
        Ok(unused) => {
            let used = finder.get_used_assets();
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let log_filename = format!("assets_report_{}.log", timestamp);
            
            let mut file = match File::create(&log_filename) {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to create log file: {}", e);
                    return;
                }
            };

            // Calculate total sizes
            let used_size = calculate_total_size(&used).unwrap_or(0) as f64 / 1_048_576.0;
            let unused_size = calculate_total_size(&unused).unwrap_or(0) as f64 / 1_048_576.0;
            
            let header = format!(
                "=== Assets Usage Report ===\nGenerated: {}\n\n\
                 Total assets: {} ({:.2} MB)\n\
                 Used assets: {} ({:.2} MB)\n\
                 Unused assets: {} ({:.2} MB)\n\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                used.len() + unused.len(),
                used_size + unused_size,
                used.len(),
                used_size,
                unused.len(),
                unused_size
            );
            
            if let Err(e) = file.write_all(header.as_bytes()) {
                error!("Failed to write to log file: {}", e);
                return;
            }

            // Write used assets
            let last_index = write_asset_section(&mut file, "Used Assets", &used, 1);
            
            // Write unused assets
            let _ = write_asset_section(&mut file, "Unused Assets", &unused, last_index);

            info!("Asset usage report has been saved to: {}", log_filename);
            info!("Total assets: {}", used.len() + unused.len());
            info!("Used assets: {}", used.len());
            info!("Unused assets: {}", unused.len());
        },
        Err(e) => {
            error!("Failed to analyze assets: {}", e);
            error!("Please check file permissions and try again.");
        }
    }
}
