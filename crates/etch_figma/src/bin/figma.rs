use std::env;

use etch_figma::{
    tsx::visitor::TsxVisitor, walker::Walker, CodeGenConfig, CodeGenSystem, FileNamingStrategy, TsxGenerator
};
use std::path::PathBuf;
use figma_api::{
    apis::{configuration::Configuration, files_api},
};
use log::info;

/// Process a single canvas and generate TSX files
fn process_canvas(canvas: &figma_api::models::CanvasNode) -> Result<etch_figma::CodeGenResult, Box<dyn std::error::Error>> {
    // Extract ALL node components using the unified visitor
    let tsx_visitor = Walker::new(TsxVisitor::new()).walk_canvas(canvas);
    
    // Create TSX generator for exportable components only
    let tsx_generator = TsxGenerator::new()
        .with_react_imports(true)
        .with_separate_files(true)
        .with_exportable_only(true);
    
    // Configure output directory
    let config = CodeGenConfig {
        output_dir: PathBuf::from("generated").join(&canvas.name),
        separate_files: true,
        exportable_only: true,
        file_naming: FileNamingStrategy::ComponentName,
    };
    
    // Get stats for logging before creating the system
    let exportable_count = tsx_visitor.exportable_root_jsx_elements_by_name().len();
    let total_root_components = tsx_visitor.root_nodes().len();
    let total_nodes = tsx_visitor.jsx_elements().len();
    let node_types_count = tsx_visitor.node_types().len();
    
    info!("Processing canvas '{}' with {} exportable components out of {} total root components (total {} nodes from {} node types)", 
          canvas.name,
          exportable_count,
          total_root_components,
          total_nodes,
          node_types_count);
    
    // Create codegen system
    let codegen_system = CodeGenSystem::new(tsx_visitor, tsx_generator, config);
    
    // Generate and write files to filesystem
    let result = codegen_system.generate_and_write()?;
    
    // Log generated files
    for (file_path, _) in &result.files {
        info!("Generated exportable component file: {}", file_path.display());
    }
    
    Ok(result)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let token = env::var("X_FIGMA_TOKEN").expect("X_FIGMA_TOKEN environment variable not set");
    let file_key = "m4UGniGw5YMoSPXbKvEfrW";

    // Create a configuration with the token
    let mut config = Configuration::default();
    config.api_key = Some(figma_api::apis::configuration::ApiKey {
        prefix: None,
        key: token,
    });

    info!("Fetching file from Figma...");


    // Call the get_file endpoint
    let file = files_api::get_file(&config, file_key, None, None, None, None, None, None).await?;

   

    // Process all canvases functionally
    let codegens: Result<Vec<_>, _> = file.document.children
        .iter()
        .map(|canvas| process_canvas(canvas))
        .collect();
    
    let codegens = codegens?;
    
    

    Ok(())
}
