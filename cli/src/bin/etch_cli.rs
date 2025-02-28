use std::{fs::File, io::Write};

use cli::{Cli, Commands, HtmlCommands};
use clap::Parser;

use etch_html::{file, visitor};
use etch_core::walk::FileWalker;
use log::info;
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    env_logger::init();

    info!("Starting CLI");

    let cli = Cli::parse();

    let walker = FileWalker::new(["html"]);


    if let Some(config_path) = cli.config.as_deref() {
        info!("Value for config: {}", config_path.display());
    }


    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    if let Some(cmd) = &cli.cmd { if let Commands::Html { cmd } = cmd { match cmd {

        HtmlCommands::ExtractSvgs { root_dir, output_dir, svg_import_type, preserve_structure,  asset_dir} => {
            info!("Extracting SVGs from {}", root_dir.display());

            let _ = walker.visit(root_dir, |path, _| {

                let visitor = visitor::SvgExtractVisitor::new(*svg_import_type, asset_dir.clone());
                let (updated_dom, visitor) = file::process_html_file(path, visitor)?;

                // Calculate relative path if preserve_structure is true
                let output_path = if *preserve_structure {
                    let rel_path = path.strip_prefix(root_dir).unwrap_or(path);
                    output_dir.join(rel_path)
                } else {
                    output_dir.join(path.file_name().unwrap())
                };

                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut file = File::create(output_path)?;
                file.write_all(updated_dom.as_bytes())?;

                for (key, value) in visitor.svgs().iter() {
                    // For SVGs, use the same directory as the HTML file
                    let svg_output_path = if *preserve_structure {
                        let rel_path = path.strip_prefix(root_dir).unwrap_or(path);
                        let parent = rel_path.parent().unwrap_or_else(|| std::path::Path::new(""));
                        let base_path = output_dir.join(parent);
                        // Add asset_dir if it exists
                        if let Some(asset_dir) = asset_dir {
                            base_path.join(asset_dir).join(format!("{}.svg", key))
                        } else {
                            base_path.join(format!("{}.svg", key))
                        }
                    } else {
                        if let Some(asset_dir) = asset_dir {
                            output_dir.join(asset_dir).join(format!("{}.svg", key))
                        } else {
                            output_dir.join(format!("{}.svg", key))
                        }
                    };

                    if let Some(parent) = svg_output_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let mut file = File::create(svg_output_path)?;
                    file.write_all(value.as_bytes())?;
                }

                info!("Extracting HTML from {}", path.display());
                Ok(())
            });


        }
       
    } } }

    // Continued program logic goes here...
}