use std::{fs::File, io::Write};

use clap::Parser;
use cli::{
    Cli, Commands, EtchCliError, FigmaCommands,
    figma_conversion::{FigmaConversionError, Project},
};

use dotenv::dotenv;
use etch_core::walk::FileWalker;
use etch_html::{file, visitor};
use log::info;

fn main() -> Result<(), EtchCliError> {
    dotenv().ok();
    env_logger::init();

    info!("Starting CLI");

    let cli = Cli::parse();

    match cli.cmd {
        Some(Commands::Figma { cmd }) => match cmd {
            FigmaCommands::GenerateNextjsPages {
                pages_dir,
                app_config_path,
            } => {
                let project = Project::from_file(pages_dir, app_config_path)?;

                info!("Project loaded with {} entries", project.file_tree.len());
                info!("Starting project conversion...");
                project.run()?;
                info!("Project conversion completed successfully");
            }
        },
        _ => {}
    }

    Ok(())
}
