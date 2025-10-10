use etch_figma::tsx::path_registry::PathRegistry;

fn main() {
    // Create a new path registry
    let mut registry = PathRegistry::new();

    // Example SVG paths from a Figma design
    let simple_path = "M10,10 L90,90";
    let complex_path = "M973.077 1169 C973.077 1195.23 968.5 1207.35 959.346 1207.35 C950.192 1207.35 945.615 1195.23 945.615 1169 C945.615 1142.77 950.192 1130.65 959.346 1130.65 C968.5 1130.65 973.077 1142.77 973.077 1169 Z";

    // Check if paths should be extracted
    println!(
        "Simple path should extract: {}",
        registry.should_extract_path(simple_path)
    );
    println!(
        "Complex path should extract: {}",
        registry.should_extract_path(complex_path)
    );

    // Register paths
    let hash1 = registry.register_path(complex_path);
    println!("\nRegistered complex path with hash: {}", hash1);

    // Register the same path again - should return same hash
    let hash2 = registry.register_path(complex_path);
    println!("Re-registered same path, got hash: {}", hash2);
    println!("Hashes match: {}", hash1 == hash2);

    // Register another path
    let another_path = "M0,0 L100,100 L100,0 Z";
    let hash3 = registry.register_path(another_path);
    println!("\nRegistered another path with hash: {}", hash3);

    // Generate TypeScript file
    println!("\n=== Generated TypeScript File ===\n");
    let ts_content = registry.generate_ts_file();
    println!("{}", ts_content);

    // Generate import statement
    println!("\n=== Import Statement ===");
    println!("{}", registry.get_import_statement("./svg-paths"));

    // Show usage example
    println!("\n=== Usage in Component ===");
    println!("import svgPaths from \"./svg-paths\";");
    println!();
    println!("function Component() {{");
    println!("  return (");
    println!("    <svg>");
    println!("      <path d={{svgPaths.{}}} fill=\"#440099\" />", hash1);
    println!("    </svg>");
    println!("  );");
    println!("}}");
}
