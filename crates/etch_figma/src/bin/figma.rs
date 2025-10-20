use std::env;
use std::fs;

use etch_figma::{
    CodeGenConfig, UnifiedPipelineBuilder,
    codegen_ext::{SvgContainerMode, TextInSvgMode, VectorExportStrategy},
};
use figma_api::apis::{configuration::Configuration, files_api};
use log::info;
use std::path::PathBuf;

/// Load cached Figma file or fetch from API
async fn get_figma_file(
    config: &Configuration,
    file_key: &str,
    use_cache: bool,
) -> Result<figma_api::models::InlineObject, Box<dyn std::error::Error>> {
    let cache_dir = PathBuf::from(".cache/figma");
    let cache_file = cache_dir.join(format!("{}.json", file_key));

    // Try to load from cache if enabled
    if use_cache && cache_file.exists() {
        info!("Loading file from cache: {}", cache_file.display());
        match fs::read_to_string(&cache_file) {
            Ok(json) => {
                if let Ok(file) = serde_json::from_str(&json) {
                    info!("Successfully loaded from cache");
                    return Ok(file);
                }
                info!("Cache invalid, fetching from API");
            }
            Err(e) => {
                info!("Failed to read cache: {}, fetching from API", e);
            }
        }
    }

    // Fetch from API
    info!("Fetching file from Figma API...");
    let file = files_api::get_file(config, file_key, None, None, None, None, None, None).await?;

    // Save to cache if enabled
    if use_cache {
        fs::create_dir_all(&cache_dir)?;
        let json = serde_json::to_string_pretty(&file)?;
        fs::write(&cache_file, json)?;
        info!("Saved to cache: {}", cache_file.display());
    }

    Ok(file)
}

/// Process a single canvas and generate TSX files
async fn process_canvas(
    canvas: &figma_api::models::CanvasNode,
    file_key: &str,
) -> Result<etch_figma::CodeGenResult, Box<dyn std::error::Error>> {
    // Configure output directory with new features
    let output_dir =
        PathBuf::from("C:\\Users\\Hector.C\\typescript\\figma-make\\src\\app").join(&canvas.name);

    let config = CodeGenConfig {
        output_dir: output_dir.clone(),
        separate_files: true,
        exportable_only: true,
        file_naming: etch_figma::FileNamingStrategy::ComponentName,
        use_css_variables: true, // Enable CSS variable theming
        extract_svg_paths: true, // Extract complex SVG paths
        svg_paths_filename: "svg-paths".to_string(),

        // New configuration options
        vector_export_strategy: VectorExportStrategy::FigmaApi,
        svg_postprocess_enabled: true,
        svg_container_mode: SvgContainerMode::WrapAll,
        svg_responsive_mode: true,
        detect_superscripts: true,
        text_in_svg_mode: TextInSvgMode::Auto,
        enable_container_queries: true,
        responsive_breakpoint_px: 640.0,
    };

    // Create unified pipeline with Figma generation + TSX post-processing
    let mut pipeline = UnifiedPipelineBuilder::new()
        .with_figma_config(config)
        .with_figma_svg_processing() // Add SVG superscript fixes and path optimization
        .with_uuid_injection() // Add UUID injection for React keys
        .with_file_key(file_key.to_string()) // Pass file key for Figma API SVG export
        .build();

    info!("Processing canvas '{}' with unified pipeline", canvas.name);

    // Process the canvas through the unified pipeline
    let codegen = pipeline.process_canvas(canvas).await?;

    // Write files to the filesystem
    codegen.write_to_filesystem(&output_dir)?;

    // Log generated files
    for (file_path, _) in &codegen.files {
        info!(
            "Generated exportable component file: {}",
            output_dir.join(file_path).display()
        );
    }

    Ok(codegen)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let token = env::var("X_FIGMA_TOKEN").expect("X_FIGMA_TOKEN environment variable not set");
    let file_key = "P9ptJxYplaIQ1hJ1IQ9z6w";

    // Create a configuration with the token
    let mut config = Configuration::default();
    config.api_key = Some(figma_api::apis::configuration::ApiKey {
        prefix: None,
        key: token,
    });

    // Check if caching is enabled
    let use_cache = env::var("FIGMA_CACHE_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if use_cache {
        info!("Cache enabled - will use cached responses when available");
    }

    // Fetch file (with caching if enabled)
    let file = get_figma_file(&config, file_key, use_cache).await?;

    // Process all canvases concurrently
    info!(
        "Processing {} canvases concurrently",
        file.document.children.len()
    );

    let tasks = file.document.children.iter().map(|canvas| async move {
        info!("Starting canvas: {}", canvas.name);
        let result = process_canvas(canvas, file_key).await;
        match &result {
            Ok(_) => info!("✓ Completed canvas: {}", canvas.name),
            Err(e) => log::error!("✗ Failed canvas {}: {}", canvas.name, e),
        }
        result
    });

    let codegens = futures::future::try_join_all(tasks).await?;

    info!("Successfully processed {} canvases", codegens.len());

    Ok(())
}
