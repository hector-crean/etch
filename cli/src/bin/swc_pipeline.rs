use etch_cli::figma_conversion::{FigmaConversionError, Project};

use dotenv::dotenv;
use log::info;

fn main() -> Result<(), FigmaConversionError> {
  dotenv().ok();
  env_logger::init();

  // let pages_dir = r#"C:\Users\Hector.C\typescript\ser135-new\frontend\src\app\(pages)"#;
  let pages_dir = r#"/Users/hectorcrean/typescript/INS107_Interactive_Patient_Journey/src/app/(pages)"#;

  // let app_config_path = r#"C:\Users\Hector.C\typescript\ser135-new\frontend\src\app.config.json"#;
  let app_config_path = r#"/Users/hectorcrean/typescript/INS107_Interactive_Patient_Journey/src/app.config.json"#;

  info!("Loading project from file: {}", app_config_path);

  let project = Project::from_file(pages_dir, app_config_path)?;

  info!("Project loaded with {} entries", project.file_tree.len());

  info!("Starting project conversion...");
  project.run()?;
  info!("Project conversion completed successfully");

  Ok(())
}
