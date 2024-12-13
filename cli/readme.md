# CLI Documentation

A command-line tool for processing HTML, TSX, and Markdown files.

## Etch HTML CLI Usage Guide

This guide provides instructions on how to use the Etch HTML command-line interface (CLI) for processing HTML, TSX, and Markdown files. The CLI is designed to perform various tasks, including extracting SVGs from HTML documents, managing TSX transformations, and working with Markdown content.

## Installation

1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) and `cargo` installed.
2. Clone the repository:
   ```bash
   git clone <repository-url>
   cd <repository-directory>
   ```
3. Build the CLI:
   ```bash
   cargo build --release
   ```
4. Run the CLI:
   ```bash
   cargo run --release
   ```

> Note: The compiled binary will be located in `target/release`. You can run it from there using `./etch_html_cli`

## Global Options

- `-c, --config <FILE>`: Specify a custom configuration file path to override default settings.

## Commands

The CLI provides three main categories of commands:
1. HTML Commands
2. TSX Commands
3. Markdown Commands (Md)

> Use `--help` with any subcommand to view more details

### HTML Commands

HTML-specific operations are grouped under the `html` command:

./etch_html_cli html [SUBCOMMAND]

#### extract-svgs Subcommand

Extracts SVG elements from HTML files, writes them to a specified output directory, and optionally transforms the structure of the output directories.

**Flags and Arguments:**
- `-r, --root-dir <PATH>`: The root directory containing HTML files to process
- `-o, --output-dir <PATH>`: The directory where extracted SVGs will be saved
- `-s, --svg-import-type <SvgImportType>`: Determines how SVGs are imported
  - Possible values are defined by the SvgImportType enum in the source code
- `-p, --preserve-structure`: When set, the directory structure from root_dir is replicated in output_dir
- `-a, --asset-dir <PATH>`: Optional path to an additional asset directory

**Example Usage:**
```bash
./etch_html_cli html extract-svgs \
    --root-dir ./input_html \
    --output-dir ./input_html \
    --svg-import-type object \
    --preserve-structure
```

