use etch_tsx::file::visit_tsx_file_mut;
use etch_tsx::visitor::framer_motion_visitor::{
    AnimationConfig, AnimationType, FramerMotionVisitor,
};
use std::path::Path;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new visitor
    let mut visitor = FramerMotionVisitor::new(
        "/Users/hectorcrean/rust/etch/figma-app/src/app/(pages)/slides/page.tsx",
    );

    // Register animations for specific elements
    visitor.register_animation("Vector_3".to_string(), AnimationConfig {
        element_id: "OSA DISEASE BURDEN".to_string(),
        animation_type: AnimationType::PathDrawing,
        custom_delay: Some(0.5),
        stroke_color: Some("#ff0088".to_string()),
        inherit_children: true, // This is the default, so it's optional
    });

    // Apply the visitor to transform the file
    let dest_path =
        Path::new("/Users/hectorcrean/rust/etch/figma-app/src/app/(pages)/slides/page.tsx");
    let (mut tsx, _) = visit_tsx_file_mut(dest_path.to_path_buf(), visitor)?;

    // Add "use client" directive
    // Remove any existing "use client" directives
    let tsx = tsx
        .replace("\"use client\";\n", "")
        .replace("'use client';\n", "");

    // Add "use client" directive at the top
    let tsx = format!("\"use client\";\n\n{}", tsx);
    // Write the transformed file
    std::fs::write(dest_path, tsx)?;

    Ok(())
}
