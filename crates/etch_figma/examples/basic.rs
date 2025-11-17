//! Basic example: Simple Figma to TSX conversion
//!
//! This example demonstrates the simplest way to convert a Figma design
//! to React/TSX components.

use etch_figma::pipeline::UnifiedPipelineBuilder;
use etch_figma::core::CodeGenConfig;
use figma_api::models::CanvasNode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load your Figma file key from environment or config
    let file_key = std::env::var("FIGMA_FILE_KEY")
        .unwrap_or_else(|_| "your_figma_file_key".to_string());
    
    // Create a default configuration
    let config = CodeGenConfig::default();
    
    // Build the pipeline
    let mut pipeline = UnifiedPipelineBuilder::new()
        .with_figma_config(config)
        .with_file_key(file_key)
        .build();
    
    // For this example, we'll assume you have a canvas node
    // In a real application, you'd fetch this from the Figma API
    // let canvas = fetch_canvas_from_figma(&file_key).await?;
    
    // For demonstration, let's show how you would process a canvas:
    // let result = pipeline.process_canvas(&canvas).await?;
    
    // Write the generated code to the filesystem
    // result.write_to_filesystem(std::path::Path::new("./generated"))?;
    
    println!("Basic example completed!");
    println!("In a real application:");
    println!("  1. Fetch canvas from Figma API");
    println!("  2. Process with pipeline.process_canvas()");
    println!("  3. Write results to filesystem");
    
    Ok(())
}

/// Example of how you might fetch a canvas from Figma
/// (This requires the figma_rest_api_file crate)
#[allow(dead_code)]
async fn fetch_canvas_from_figma(_file_key: &str) -> Result<CanvasNode, Box<dyn std::error::Error>> {
    // Implementation would go here
    // This is just a placeholder to show the pattern
    unimplemented!("Implement Figma API fetching in your application")
}

