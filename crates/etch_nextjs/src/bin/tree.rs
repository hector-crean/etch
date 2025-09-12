use etch_nextjs::Cli;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::new();
    
    // Target directory path
    let target_dir = Path::new("/Users/hectorcrean/typescript/INS107_Interactive_Patient_Journey/figma/flow");
    
    // Check if directory exists
    if !target_dir.exists() {
        eprintln!("Error: Directory does not exist: {}", target_dir.display());
        std::process::exit(1);
    }
    
    if !target_dir.is_dir() {
        eprintln!("Error: Path is not a directory: {}", target_dir.display());
        std::process::exit(1);
    }
    
    // Get the directory structure
    let structure = Cli::get_directory_structure::<()>(target_dir, target_dir)?;
    
    // Convert to JSON and print
    let json = serde_json::to_string_pretty(&structure)?;
    println!("{}", json);
    
    Ok(())
}