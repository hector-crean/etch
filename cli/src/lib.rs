use std::path::PathBuf;

use clap::{Parser, Subcommand};
use etch_html::visitor::svg_extractor_visitor::SvgImportType;

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
    /// HTML processing commands
    /// 
    /// Contains subcommands specific to HTML file processing
    Html {
        #[command(subcommand)]
        cmd: HtmlCommands,
    },
    /// TSX file processing commands
    Tsx {
        /// When true, lists available test values
        #[arg(short, long)]
        list: bool,
    },
    /// Markdown file processing commands
    Md {
        /// When true, lists available test values
        #[arg(short, long)]
        list: bool,
    },
}

/// HTML-specific commands and operations
/// 
/// This enum contains all commands related to HTML processing,
/// including SVG extraction and other HTML-related operations.
#[derive(Subcommand)]
pub enum HtmlCommands {
    /// Extracts SVG elements from HTML files
    ExtractSvgs {
        /// The root directory containing HTML files to process
        /// 
        /// This directory will be searched for HTML files containing SVGs
        #[arg(short, long)]
        root_dir: PathBuf,
        
        /// The root directory where extracted SVGs will be saved
        /// 
        /// All processed SVG files will be written to this location
        /// We shall often just set this to the same as the root_dir
        #[arg(short, long)]
        output_dir: PathBuf,

        /// Specifies how SVGs should be imported
        /// 
        /// Determines the format and method used for SVG imports
        #[arg(short, long)]
        svg_import_type: SvgImportType,

        /// Controls output directory structure
        /// 
        /// When true, maintains the same directory structure as the input
        /// When false, places all SVGs directly in the output directory
        #[arg(short, long)]
        preserve_structure: bool,

        /// Optional asset directory path
        /// 
        /// Specifies an additional directory path to append
        #[arg(short, long)]
        asset_dir: Option<PathBuf>,
    },
}
