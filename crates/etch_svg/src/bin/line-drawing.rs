use std::fs;

use etch_svg::SvgParser;



fn main() {
    let svg_content = r#"
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
        <line x1="0" y1="0" x2="100" y2="100" stroke="black" stroke-width="2" />
    </svg>
    "#;

    let svg_parser = SvgParser::new(svg_content);


    let motion = svg_parser.to_motion_tsx().unwrap();

    fs::write("motion.tsx", motion).unwrap();
}
