use clap::Parser;
use colored::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use dotenv::dotenv;
use log::{error, info, warn};
use etch_tsx::{StatefulPipeline, AssetVisitor, AssetReference};
use glob::Pattern;

#[derive(Parser)]
#[command(name = "asset-relocator")]
#[command(about = "Find and relocate assets in TypeScript/React projects")]
struct Args {
    /// Source directory containing TSX files
    #[arg(short, long)]
    source_dir: PathBuf,

    /// Target directory where assets should be moved
    #[arg(short, long)]
    target_dir: PathBuf,

    /// Base directory for resolving relative paths (defaults to source_dir)
    #[arg(short, long)]
    base_dir: Option<PathBuf>,

    /// Dry run - don't actually move files or modify code
    #[arg(short, long)]
    dry_run: bool,

    /// File extensions to process (comma-separated, defaults to tsx,ts,jsx,js)
    #[arg(short, long, default_value = "tsx,ts,jsx,js")]
    extensions: String,

    /// Copy assets instead of moving them
    #[arg(short, long)]
    copy: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Only process files matching this pattern (glob-style, e.g., "src/data/**" or "**/data/*")
    #[arg(short = 'i', long)]
    include_pattern: Option<String>,
}

#[derive(Debug)]
struct AssetRelocationPlan {
    assets: Vec<AssetReference>,
    relocations: HashMap<PathBuf, PathBuf>, // old_path -> new_path
    affected_files: HashSet<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    let args = Args::parse();

    let base_dir = args.base_dir.unwrap_or_else(|| args.source_dir.clone());
    let extensions: Vec<String> = args.extensions.split(',').map(|s| s.trim().to_string()).collect();

    info!("{}", "🔍 Scanning for TypeScript/React files...".cyan());
    if let Some(ref pattern) = args.include_pattern {
        info!("  Using pattern filter: {}", pattern.cyan());
    }

    // Find all TypeScript/React files
    let tsx_files = find_tsx_files(&args.source_dir, &extensions, args.include_pattern.as_deref())?;
    info!("Found {} files to process", tsx_files.len().to_string().green());

    // Phase 1: Discover all assets
    info!("\n{}", "📋 Phase 1: Discovering assets...".cyan());
    let mut all_assets = Vec::new();
    let mut affected_files = HashSet::new();

    for tsx_file in &tsx_files {
        if args.verbose {
            info!("  Processing: {}", tsx_file.display().to_string().dimmed());
        }

        let visitor = AssetVisitor::new(tsx_file, &base_dir, &args.target_dir);
        let pipeline = StatefulPipeline::new(visitor);

        // Process the file to discover assets
        match pipeline.run(tsx_file) {
            Ok((_content, visitor)) => {
                let assets = visitor.assets();
                if !assets.is_empty() {
                    affected_files.insert(tsx_file.clone());
                    all_assets.extend(assets.iter().cloned());
                    
                    if args.verbose {
                        info!("    Found {} assets", assets.len().to_string().green());
                    }
                }
            }
            Err(e) => {
                error!("  {} Failed to process {}: {}", "❌".red(), tsx_file.display(), e);
            }
        }
    }

    // Phase 2: Plan asset relocations
    info!("\n{}", "📍 Phase 2: Planning asset relocations...".cyan());
    info!("  Total assets discovered: {}", all_assets.len().to_string().cyan());
    let mut relocation_plan = plan_asset_relocations(all_assets, &args.target_dir, args.verbose)?;
    relocation_plan.affected_files = affected_files;

    // Print summary
    print_relocation_summary(&relocation_plan, args.verbose);

    if args.dry_run {
        info!("\n{}", "🏃 Dry run mode - no changes will be made".yellow());
        return Ok(());
    }

    // Phase 3: Create target directory
    info!("\n{}", "📁 Phase 3: Creating target directory...".cyan());
    if !args.target_dir.exists() {
        fs::create_dir_all(&args.target_dir)?;
        info!("  Created directory: {}", args.target_dir.display().to_string().green());
    }

    // Phase 4: Move/copy assets
    info!("\n{}", "📦 Phase 4: Relocating assets...".cyan());
    relocate_assets(&relocation_plan.relocations, args.copy, args.verbose)?;

    // Phase 5: Update TSX files
    info!("\n{}", "✏️  Phase 5: Updating file references...".cyan());
    info!("  Files to update: {}", relocation_plan.affected_files.len().to_string().cyan());
    if args.verbose {
        for file in &relocation_plan.affected_files {
            info!("    {}", file.display().to_string().dimmed());
        }
    }
    update_tsx_files(&relocation_plan, &base_dir, &args.target_dir, args.verbose)?;

    info!("\n{}", "✅ Asset relocation completed successfully!".green());
    Ok(())
}

fn find_tsx_files(dir: &Path, extensions: &[String], include_pattern: Option<&str>) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    
    // Compile the glob pattern if provided
    let pattern = if let Some(pattern_str) = include_pattern {
        Some(Pattern::new(pattern_str).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid glob pattern: {}", e))
        })?)
    } else {
        None
    };
    
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            // Check file extension
            if let Some(ext) = path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    if extensions.contains(&ext_str.to_lowercase()) {
                        // If no pattern specified, include all files
                        if pattern.is_none() {
                            files.push(path.to_path_buf());
                        } else if let Some(ref p) = pattern {
                            // Check if the file path matches the pattern
                            let path_str = path.to_string_lossy();
                            if p.matches(&path_str) {
                                files.push(path.to_path_buf());
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(files)
}

fn plan_asset_relocations(
    assets: Vec<AssetReference>,
    target_dir: &Path,
    verbose: bool,
) -> Result<AssetRelocationPlan, std::io::Error> {
    let mut relocations = HashMap::new();
    let mut affected_files = HashSet::new();
    let mut missing_assets = Vec::new();
    let mut found_assets = Vec::new();
    
    for asset in &assets {
        if asset.resolved_path.exists() {
            let target_path = asset.generate_target_path(target_dir);
            relocations.insert(asset.resolved_path.clone(), target_path);
            found_assets.push(asset);
        } else {
            missing_assets.push(asset);
            if verbose {
                warn!("    Asset not found on disk: '{}' -> {}", 
                    asset.original_path, 
                    asset.resolved_path.display().to_string().dimmed()
                );
            }
        }
    }
    
    if verbose {
        if !found_assets.is_empty() {
            info!("  {} assets found on filesystem and will be relocated", found_assets.len().to_string().green());
        }
        if !missing_assets.is_empty() {
            warn!("  {} assets not found on filesystem (code references will still be updated)", missing_assets.len().to_string().yellow());
        }
    }
    
    Ok(AssetRelocationPlan {
        assets,
        relocations,
        affected_files,
    })
}

fn print_relocation_summary(plan: &AssetRelocationPlan, verbose: bool) {
    info!("  Assets to relocate: {}", plan.relocations.len().to_string().green());
    
    if verbose {
        for (old_path, new_path) in &plan.relocations {
            info!("    {} → {}", 
                old_path.display().to_string().dimmed(),
                new_path.display().to_string().green()
            );
        }
    }
}

fn relocate_assets(
    relocations: &HashMap<PathBuf, PathBuf>,
    copy_mode: bool,
    verbose: bool,
) -> Result<(), std::io::Error> {
    for (old_path, new_path) in relocations {
        if verbose {
            let action = if copy_mode { "Copying" } else { "Moving" };
            info!("  {}: {} → {}", 
                action,
                old_path.display().to_string().dimmed(),
                new_path.display().to_string().green()
            );
        }

        // Ensure parent directory exists
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if copy_mode {
            fs::copy(old_path, new_path)?;
        } else {
            fs::rename(old_path, new_path)?;
        }
    }
    Ok(())
}

fn update_tsx_files(
    plan: &AssetRelocationPlan,
    base_dir: &Path,
    target_dir: &Path,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build path mappings for string replacement
    let mut path_mappings = HashMap::new();
    let project_root_ref = base_dir.parent().unwrap_or(base_dir);
    
    for asset in &plan.assets {
        let new_target_path = asset.generate_target_path(target_dir);
        let new_path = match &asset.reference_type {
            etch_tsx::ReferenceType::ImportStatement => {
                if let Ok(relative) = new_target_path.strip_prefix(project_root_ref) {
                    format!("/{}", relative.display())
                } else {
                    new_target_path.display().to_string()
                }
            },
            _ => {
                 if let Ok(relative) = new_target_path.strip_prefix(project_root_ref.join("public")) {
                    format!("/{}", relative.display())
                } else if let Ok(relative) = new_target_path.strip_prefix(project_root_ref) {
                    format!("/{}", relative.display())
                }
                 else {
                    new_target_path.display().to_string()
                }
            }
        };
        path_mappings.insert(asset.original_path.clone(), new_path);
    }

    if verbose {
        info!("  Created {} path mappings for string replacement:", path_mappings.len().to_string().cyan());
        for (old, new) in &path_mappings {
            info!("    '{}' -> '{}'", old.dimmed(), new.green());
        }
    }

    // Process each affected file with string replacement
    for file_path in &plan.affected_files {
        let mut content = fs::read_to_string(file_path)?;
        let mut modified_content = content.clone();
        
        // Apply string replacements
        for (old_path, new_path) in &path_mappings {
            // Replace quoted strings (for object properties, imports, and JSX attributes)
            let old_quoted_double = format!("\"{}\"", old_path);
            let new_quoted_double = format!("\"{}\"", new_path);
            modified_content = modified_content.replace(&old_quoted_double, &new_quoted_double);
            
            // Replace single-quoted strings
            let old_quoted_single = format!("'{}'", old_path);
            let new_quoted_single = format!("'{}'", new_path);
            modified_content = modified_content.replace(&old_quoted_single, &new_quoted_single);
        }

        // Write the updated content if changed
        if modified_content != content {
            info!("  Updating: {}", file_path.display().to_string().green());
            fs::write(file_path, modified_content)?;
        } else if verbose {
            info!("  No changes needed for: {}", file_path.display().to_string().dimmed());
        }
    }

    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();
        
        // Create directory structure
        let src_dir = project_root.join("src");
        let data_dir = src_dir.join("data");
        let public_dir = project_root.join("public");
        let assets_dir = src_dir.join("assets");
        
        fs::create_dir_all(&data_dir).unwrap();
        fs::create_dir_all(&public_dir).unwrap();
        fs::create_dir_all(&assets_dir).unwrap();
        
        // Create test assets
        fs::write(assets_dir.join("icon.svg"), "<svg>icon</svg>").unwrap();
        fs::write(public_dir.join("poster.jpg"), "fake poster").unwrap();
        fs::write(public_dir.join("video.mp4"), "fake video").unwrap();
        
        // Create test data file
        let data_content = r#"
import icon from "@/assets/icon.svg";

export const testData = {
    sections: [
        {
            blocks: [
                {
                    type: "Video",
                    props: {
                        url: "video.mp4",
                        poster: "poster.jpg",
                        icon: icon,
                    }
                }
            ]
        }
    ]
};
"#;
        fs::write(data_dir.join("test.tsx"), data_content).unwrap();
        
        (temp_dir, project_root)
    }

    #[test]
    fn test_complete_asset_relocation_workflow() {
        let (_temp_dir, project_root) = create_test_project();
        let src_dir = project_root.join("src");
        let target_dir = project_root.join("public/locales/en");
        
        // Create target directory
        fs::create_dir_all(&target_dir).unwrap();
        
        // Step 1: Discover assets
        let extensions = vec!["tsx".to_string()];
        let tsx_files = find_tsx_files(&src_dir, &extensions, None).unwrap();
        assert!(!tsx_files.is_empty(), "Should find TSX files");
        
        let mut all_assets = Vec::new();
        let mut affected_files = HashSet::new();
        
        for tsx_file in &tsx_files {
            let visitor = AssetVisitor::new(tsx_file, &src_dir, &target_dir);
            let pipeline = StatefulPipeline::new(visitor);
            
            match pipeline.run(tsx_file) {
                Ok((_content, visitor)) => {
                    let assets = visitor.assets();
                    if !assets.is_empty() {
                        affected_files.insert(tsx_file.clone());
                        all_assets.extend(assets.iter().cloned());
                    }
                }
                Err(e) => panic!("Failed to process file: {}", e),
            }
        }
        
        assert!(!all_assets.is_empty(), "Should discover assets");
        assert!(!affected_files.is_empty(), "Should track affected files");
        
        // Step 2: Plan relocations
        let mut relocation_plan = plan_asset_relocations(all_assets, &target_dir, false).unwrap();
        relocation_plan.affected_files = affected_files;
        
        // Step 3: Test path mapping creation
        let mut path_mappings = HashMap::new();
        let project_root_ref = src_dir.parent().unwrap_or(&src_dir);
        
        for asset in &relocation_plan.assets {
            let new_target_path = asset.generate_target_path(&target_dir);
            let new_path = match &asset.reference_type {
                etch_tsx::ReferenceType::ImportStatement => {
                    if let Ok(relative) = new_target_path.strip_prefix(project_root_ref) {
                        format!("/{}", relative.display())
                    } else {
                        format!("/{}", new_target_path.file_name().unwrap_or_default().to_string_lossy())
                    }
                },
                _ => {
                    if let Ok(relative) = new_target_path.strip_prefix(project_root_ref) {
                        format!("/{}", relative.display())
                    } else {
                        format!("/{}", new_target_path.file_name().unwrap_or_default().to_string_lossy())
                    }
                }
            };
            path_mappings.insert(asset.original_path.clone(), new_path);
        }
        
        assert!(!path_mappings.is_empty(), "Should create path mappings");
        
        // Verify specific mappings
        assert!(path_mappings.contains_key("@/assets/icon.svg"), "Should map TypeScript alias");
        assert!(path_mappings.contains_key("video.mp4"), "Should map bare filename");
        assert!(path_mappings.contains_key("poster.jpg"), "Should map bare filename");
        
        // Step 4: Test file updates (simulation)
        for file_path in &relocation_plan.affected_files {
            let mut visitor = AssetVisitor::new(file_path, &src_dir, &target_dir);
            
            // Add path mappings
            for (old_path, new_path) in &path_mappings {
                visitor.add_path_mapping(old_path.clone(), new_path.clone());
            }
            
            let pipeline = StatefulPipeline::new(visitor);
            
            match pipeline.run(file_path) {
                Ok((updated_content, _visitor)) => {
                    println!("Updated content:\n{}", updated_content);
                    println!("Path mappings: {:?}", path_mappings);
                    
                    // Verify that paths were updated
                    assert!(updated_content.contains("/public/locales/en/"), "Should contain updated paths");
                    assert!(!updated_content.contains("@/assets/icon.svg"), "Should not contain old TypeScript alias");
                    assert!(!updated_content.contains("url: \"video.mp4\""), "Should not contain old bare filename");
                }
                Err(e) => panic!("Failed to update file: {}", e),
            }
        }
    }
} 