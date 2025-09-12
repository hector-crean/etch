use std::{collections::HashMap, env};

use etch_figma::{TextNodeExt, JSXElementExt};
use figma_api::{
    apis::{configuration::Configuration, files_api},
    models::{CanvasNode, SubcanvasNode},
};
use log::info;



#[derive(Default)]
pub struct SubNodeWalker {
    /// Maps node IDs to their TSX string representation
    tsx_nodes: HashMap<String, String>,
    /// Maps node IDs to their readable names/paths for reference
    node_names: HashMap<String, String>,
}


impl SubNodeWalker {

     // Recursively walk all nodes and log text with hierarchical path
 fn walk_subnode(&mut self, node: &SubcanvasNode, path: &mut Vec<String>) {
    match node {
        SubcanvasNode::Text(text_node) => {
            let text = text_node.characters.trim();
            if !text.is_empty() {
                let path_str = path.join(" > ");
                let node_id = text_node.id.clone();
                
                let tsx = text_node.to_jsx();
                let tsx_string = tsx.to_string().unwrap();

                info!("{} ({}): {}", path_str, node_id, tsx_string);

                self.tsx_nodes.insert(node_id.clone(), tsx_string);
                self.node_names.insert(node_id, path_str);
            }
        }
        SubcanvasNode::Frame(frame) => {
            path.push(frame.name.clone());
            for child in &frame.children {
                self.walk_subnode(child, path);
            }
            path.pop();
        }
        SubcanvasNode::Group(group) => {
            path.push(group.name.clone());
            for child in &group.children {
                self.walk_subnode(child, path);
            }
            path.pop();
        }
        SubcanvasNode::Component(component) => {
            path.push(component.name.clone());
            for child in &component.children {
                self.walk_subnode(child, path);
            }
            path.pop();
        }
        SubcanvasNode::Instance(instance) => {
            path.push(instance.name.clone());
            for child in &instance.children {
                self.walk_subnode(child, path);
            }
            path.pop();
        }
        SubcanvasNode::Section(section) => {
            path.push(section.name.clone());
            for child in &section.children {
                self.walk_subnode(child, path);
            }
            path.pop();
        }
       
        // Other variants do not contain children or are not relevant for text traversal
        _ => {}
    }
}

    /// Generate a TypeScript/JavaScript file with exported TSX components
    pub fn generate_tsx_export(&self, canvas_name: &str) -> String {
        let mut output = String::new();
        
        // Add header comment
        output.push_str(&format!("// Auto-generated TSX components from Figma canvas: {}\n", canvas_name));
        output.push_str("// This file contains React JSX components extracted from Figma text nodes\n\n");
        
        // Add React import
        output.push_str("import React from 'react';\n\n");
        
        // Add type definitions
        output.push_str("export interface FigmaTextComponent {\n");
        output.push_str("  id: string;\n");
        output.push_str("  name: string;\n");
        output.push_str("  component: () => JSX.Element;\n");
        output.push_str("}\n\n");
        
        // Generate individual component functions
        for (node_id, tsx_string) in &self.tsx_nodes {
            let component_name = self.sanitize_component_name(node_id);
            let default_name = node_id.clone();
            let readable_name = self.node_names.get(node_id).unwrap_or(&default_name);
            
            output.push_str(&format!("// Component for: {}\n", readable_name));
            output.push_str(&format!("export const {} = (): JSX.Element => {};\n\n", component_name, tsx_string));
        }
        
        // Generate the main export object
        output.push_str("// Main export object with all components keyed by Figma node ID\n");
        output.push_str("export const figmaComponents: Record<string, FigmaTextComponent> = {\n");
        
        for (node_id, _) in &self.tsx_nodes {
            let component_name = self.sanitize_component_name(node_id);
            let default_name = node_id.clone();
            let readable_name = self.node_names.get(node_id).unwrap_or(&default_name);
            
            output.push_str(&format!("  '{}': {{\n", node_id));
            output.push_str(&format!("    id: '{}',\n", node_id));
            output.push_str(&format!("    name: '{}',\n", readable_name.replace("'", "\\'")));
            output.push_str(&format!("    component: {},\n", component_name));
            output.push_str("  },\n");
        }
        
        output.push_str("};\n\n");
        
        // Add default export
        output.push_str("export default figmaComponents;\n");
        
        output
    }
    
    /// Convert a Figma node ID to a valid React component name
    fn sanitize_component_name(&self, node_id: &str) -> String {
        // Remove colons and other invalid characters, ensure it starts with uppercase
        let sanitized = node_id
            .replace(":", "_")
            .replace("-", "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();
        
        format!("FigmaText_{}", sanitized)
    }

}


fn walk_canvas(canvas: &CanvasNode) -> SubNodeWalker {
    let mut path = vec![canvas.name.clone()];

    let mut walker = SubNodeWalker::default();
    for child in &canvas.children {
        walker.walk_subnode(child, &mut path);
    }
    walker
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let token = env::var("X_FIGMA_TOKEN").expect("X_FIGMA_TOKEN environment variable not set");
    let file_key = "m4UGniGw5YMoSPXbKvEfrW";

    // Create a configuration with the token
    let mut config = Configuration::default();
    config.api_key = Some(figma_api::apis::configuration::ApiKey {
        prefix: None,
        key: token,
    });

    info!("Fetching file from Figma...");


    // Call the get_file endpoint
    let file = files_api::get_file(&config, file_key, None, None, None, None, None, None).await?;

   

    for canvas in &file.document.children {
        let walker = walk_canvas(canvas);
        
        // Generate TypeScript file with TSX components
        let tsx_content = walker.generate_tsx_export(&canvas.name);
        let tsx_filename = format!("{}.tsx", canvas.name.replace(" ", "_").replace("/", "_"));
        std::fs::write(&tsx_filename, &tsx_content)?;
        info!("Generated TSX file: {} with {} components", tsx_filename, walker.tsx_nodes.len());
        
        // Also generate JSON for debugging/reference
        let json = serde_json::to_string_pretty(&walker.tsx_nodes)?;
        let json_filename = format!("{}-tsx.json", canvas.id);
        std::fs::write(&json_filename, json)?;
    }

    // If needed later, we can also write the whole file JSON to disk
    // let json = serde_json::to_string_pretty(&file)?;
    // let out_path = env::var("FIGMA_OUTPUT_JSON").unwrap_or_else(|_| "figma_file.json".to_string());
    // std::fs::write(&out_path, json)?;
    // info!("Saved Figma file JSON to {}", out_path);

    Ok(())
}
