//! Custom visitor example: Creating your own visitor
//!
//! This example shows how to create a custom visitor that processes
//! Figma nodes in a specific way.

use etch_figma::core::{Walker, NodeVisitor, NodeContext};
use figma_api::models::{CanvasNode, FrameNode, TextNode, VectorNode};

/// A custom visitor that collects statistics about a Figma design
#[derive(Debug, Default)]
struct DesignStatsVisitor {
    frame_count: usize,
    text_count: usize,
    vector_count: usize,
    total_nodes: usize,
    component_names: Vec<String>,
}

impl NodeVisitor for DesignStatsVisitor {
    fn visit_frame(&mut self, frame: &FrameNode, _context: &NodeContext) {
        self.frame_count += 1;
        self.total_nodes += 1;
        
        // Collect component names (frames marked for export)
        if let Some(export_settings) = &frame.export_settings {
            if !export_settings.is_empty() {
                self.component_names.push(frame.name.clone());
            }
        }
    }

    fn visit_text(&mut self, _text: &TextNode, _context: &NodeContext) {
        self.text_count += 1;
        self.total_nodes += 1;
    }

    fn visit_vector(&mut self, _vector: &VectorNode, _context: &NodeContext) {
        self.vector_count += 1;
        self.total_nodes += 1;
    }
}

impl DesignStatsVisitor {
    fn print_report(&self) {
        println!("=== Design Statistics ===");
        println!("Total nodes: {}", self.total_nodes);
        println!("Frames: {}", self.frame_count);
        println!("Text elements: {}", self.text_count);
        println!("Vector graphics: {}", self.vector_count);
        println!("\nExportable components:");
        for name in &self.component_names {
            println!("  - {}", name);
        }
    }
}

fn main() {
    println!("Custom Visitor Example");
    println!("======================\n");
    
    // Create your custom visitor
    let visitor = DesignStatsVisitor::default();
    
    // Create a walker with your visitor
    let walker = Walker::new(visitor);
    
    // In a real application, you would:
    // 1. Fetch the Figma canvas
    // 2. Walk the tree: let completed = walker.walk_canvas(&canvas);
    // 3. Use the results: completed.print_report();
    
    println!("This example shows the pattern for creating custom visitors.");
    println!("Implement NodeVisitor trait for your own analysis or conversion logic!");
}

