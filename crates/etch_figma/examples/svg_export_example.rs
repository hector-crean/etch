use etch_figma::tsx::{
    visitor::TsxVisitor, 
    svg_strategy::{SvgConfig, NodeAnalyzer},
    figma_svg_export::FigmaSvgExporter
};
use figma_api::models::VectorNode;

/// Example showing how to use the enhanced SVG export system
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure SVG export settings
    let svg_config = SvgConfig {
        use_external_paths: true,
        external_path_base: "assets/svg".to_string(),
        optimize: true,
        include_viewbox: true,
    };

    // Create visitor with SVG configuration
    let mut visitor = TsxVisitor::with_svg_config(svg_config);
    visitor.set_file_key("your-figma-file-key".to_string());

    // Example: Analyze a vector node
    let node_type = "Vector";
    let has_vector_content = true;
    let has_text_content = false;
    
    let should_use_svg = NodeAnalyzer::should_render_as_svg(
        node_type, 
        has_vector_content, 
        has_text_content
    );
    
    println!("Should render as SVG: {}", should_use_svg);

    // Example: Create SVG exporter
    let mut svg_exporter = FigmaSvgExporter::new(SvgConfig::default());
    
    // Example: Generate path mapping file
    let path_mapping = svg_exporter.generate_path_mapping();
    println!("Generated path mapping:\n{}", path_mapping);

    Ok(())
}

/// Example of how the generated TSX would look
fn example_generated_tsx() {
    // For simple vector nodes, you'd get inline SVG:
    let _inline_svg_example = r#"
<svg viewBox="0 0 100 100" data-name="icon" data-vector="true">
  <path d="M10,10 L90,90" fill="currentColor"/>
</svg>
"#;

    // For complex vector nodes, you'd get external references:
    let _external_svg_example = r#"
<svg viewBox="0 0 200 200" data-name="complex-icon" data-vector="true">
  <!-- External SVG: assets/svg/vector_123.svg -->
</svg>
"#;

    // For mixed content (HTML with SVG children):
    let _mixed_content_example = r#"
<div className="container" data-name="mixed-content">
  <p>Some text content</p>
  <svg viewBox="0 0 100 100" data-name="icon">
    <path d="M10,10 L90,90"/>
  </svg>
</div>
"#;
}
