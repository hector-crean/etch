use std::error::Error;
use svgr_rs::{transform, State, Config};

pub struct SvgConverter {
    content: String,
}

impl SvgConverter {
    pub fn new(svg_content: &str) -> Self {
        Self {
            content: svg_content.to_string(),
        }
    }

    // Create a React component from the SVG content
    pub fn to_react_component(&self, component_name: &str) -> Result<String, Box<dyn Error>> {
        // Use svgr_rs transform to convert SVG to React component
        let config = Config { typescript: true, ..Default::default()};
        let state = State { component_name: Some(component_name.to_string()), ..Default::default()};
        let jsx = transform(self.content.clone(), config, state)?;
        
        Ok(jsx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_to_tsx_file() {
        let svg_content = r#"<svg viewBox="0 0 100 100">
    <circle cx="50" cy="50" r="40"/>
</svg>"#;

        let converter = SvgConverter::new(svg_content);
        let react_component = converter.to_react_component("CircleIcon").unwrap();
        
        // Verify the component contains the React import
        assert!(react_component.contains("import React from 'react';"));
        
        // Verify the component name is used correctly
        assert!(react_component.contains("const CircleIcon = () =>"));
        
        // Verify export statement
        assert!(react_component.contains("export default CircleIcon;"));
    }
}



