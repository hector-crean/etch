use clap::Parser;
use dotenv::dotenv;
use etch_cli::figma_conversion::{FigmaConversionError, Project};
use etch_cli::{Cli, Commands};
use log::info;

fn main() -> Result<(), FigmaConversionError> {
  dotenv().ok();
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

  let args = Cli::parse();

  match args.cmd {
    Some(cmd) => match cmd {
      Commands::GenerateTsx {
        pages_dir,
        app_config_path,
      } => {
        info!("Loading project from file: {:?}", app_config_path);

        let project = Project::from_file(pages_dir, app_config_path)?;

        info!("Project loaded with {} entries", project.file_tree.len());

        info!("Starting project conversion...");
        project.run()?;
        info!("Project conversion completed successfully");
      }
    }
    None => {
      println!("No command provided");
    }
  }

  Ok(())
}
