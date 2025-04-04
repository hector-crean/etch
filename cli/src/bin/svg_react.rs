use cli::figma_conversion::{FigmaConversionError, Project};
use dotenv::dotenv;
use log::info;
use std::path::Path;

fn main() -> Result<(), FigmaConversionError> {
    dotenv().ok();
    env_logger::init();

    let base_dir = r#"C:\Users\Hector.C\rust\etch\figma-app\src\app\(pages)"#;

    let file_tree_path = r#"C:\Users\Hector.C\rust\etch\figma-app\src\file-tree.json"#;
    info!("Loading project from file: {}", file_tree_path);

    let project = Project::from_file(base_dir, file_tree_path)?;
    info!("Project loaded with {} entries", project.file_tree.len());

    info!("Starting project conversion...");
    project.run()?;
    info!("Project conversion completed successfully");

    Ok(())
}

/// Format a TypeScript/TSX file using Prettier
fn format_tsx_file(path: &Path) -> std::io::Result<()> {
    use std::process::Command;

    let output = Command::new("npx")
        .args(["prettier", "--write", path.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error formatting TSX file: {}", error);
    }

    Ok(())
}
