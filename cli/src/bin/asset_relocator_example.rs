// Example usage of the asset relocator
use std::path::PathBuf;

use etch_tsx::{StatefulPipeline, AssetVisitor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example: Process a single TSX file and discover assets
    let tsx_file = PathBuf::from("src/components/MyComponent.tsx");
    let base_dir = PathBuf::from("src");
    let target_dir = PathBuf::from("public/locales/en");
    
    println!("Processing: {}", tsx_file.display());
    
    // Create the asset visitor
    let visitor = AssetVisitor::new(&tsx_file, &base_dir, &target_dir);
    
    // Create the pipeline
    let pipeline = StatefulPipeline::new(visitor);
    
    // Process the file
    match pipeline.run(&tsx_file) {
        Ok((_updated_content, visitor)) => {
            let assets = visitor.assets();
            println!("Found {} assets:", assets.len());
            
            for asset in assets {
                println!("  - {} ({})", asset.original_path, match asset.asset_type {
                    etch_tsx::visitor::asset_visitor::AssetType::Image => "Image",
                    etch_tsx::visitor::asset_visitor::AssetType::Video => "Video", 
                    etch_tsx::visitor::asset_visitor::AssetType::Audio => "Audio",
                    etch_tsx::visitor::asset_visitor::AssetType::Document => "Document",
                    etch_tsx::visitor::asset_visitor::AssetType::Other(ref ext) => ext,
                });
            }
        }
        Err(e) => {
            eprintln!("Error processing file: {}", e);
        }
    }
    
    Ok(())
} 