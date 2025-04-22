
use clap::Parser;
use cli::{
    Cli, Commands, EtchCliError, FigmaCommands,
    figma_conversion::Project,
};

use dotenv::dotenv;
use log::info;

fn main() -> Result<(), EtchCliError> {
    dotenv().ok();
    env_logger::init();

    info!("Starting CLI");

    let cli = Cli::parse();

    if let Some(Commands::Figma { cmd }) = cli.cmd { match cmd {
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
    } }

    Ok(())
}
