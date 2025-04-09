use cli::figma_conversion::{FigmaConversionError, Project};
use dotenv::dotenv;
use log::info;
use std::path::Path;

fn main() -> Result<(), FigmaConversionError> {
    dotenv().ok();
    env_logger::init();

    // let base_dir = r#"C:\Users\Hector.C\rust\etch\figma-app\src\app\(pages)"#;
    let pages_dir = r#"/Users/hectorcrean/rust/etch/figma-app/src/app/(pages)"#;

    let app_config_path = r#"/Users/hectorcrean/rust/etch/figma-app/src/app.config.json"#;

    info!("Loading project from file: {}", app_config_path);

    let project = Project::from_file(pages_dir, app_config_path)?;

    info!("Project loaded with {} entries", project.file_tree.len());

    info!("Starting project conversion...");
    project.run()?;
    info!("Project conversion completed successfully");

    Ok(())
}
