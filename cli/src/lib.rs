pub mod core;
pub mod figma_conversion;

use clap::{Parser, Subcommand};
use etch_html::visitor::svg_extractor_visitor::SvgImportType;
use figma_conversion::FigmaConversionError;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum EtchCliError {
    #[error(transparent)]
    FigmaConversionError(#[from] FigmaConversionError),
}

/// Command-line interface configuration structure
///
/// This structure defines the main CLI configuration and holds
/// the primary command-line arguments and subcommands.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Sets a custom config file path
    ///
    /// This optional argument allows users to specify a custom configuration
    /// file to be used instead of the default settings.
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// The subcommand to execute
    ///
    /// Represents the main operations that can be performed by the CLI tool.
    #[command(subcommand)]
    pub cmd: Option<Commands>,
}

/// Main command categories available in the CLI
///
/// This enum defines the top-level commands that can be executed,
/// including HTML processing, TSX handling, and Markdown operations.
#[derive(Subcommand)]
pub enum Commands {
    ///  Processing commands
    ///
    /// Contains subcommands specific to Figma file processing
    Figma {
        #[command(subcommand)]
        cmd: FigmaCommands,
    },
}

/// Figma-specific commands and operations
///
/// This enum contains all commands related to Figma processing,
/// including extracting React components from Figma.
///

#[derive(Subcommand)]
pub enum FigmaCommands {
    GenerateNextjsPages {
        /// The pages directory
        ///
        #[arg(short, long)]
        pages_dir: PathBuf,

        /// The app config path
        ///
        #[arg(short, long)]
        app_config_path: PathBuf,
    },
}
