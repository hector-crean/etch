We effectivelty want to recreate figma-make using the figma api, and the SWC, which enables us to generate tsx. 


Figma has quite a rich representation. 
- text
- vectors
- containers / auto-layout
- effects (svgs/css filters?)


We can look at them in all their glory by examping the `SubCanvasNode` enum:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SubcanvasNode {
    #[serde(rename="BOOLEAN_OPERATION")]
    BooleanOperation(Box<models::BooleanOperationNode>),
    #[serde(rename="COMPONENT")]
    Component(Box<models::ComponentNode>),
    #[serde(rename="COMPONENT_SET")]
    ComponentSet(Box<models::ComponentSetNode>),
    #[serde(rename="CONNECTOR")]
    Connector(Box<models::ConnectorNode>),
    #[serde(rename="ELLIPSE")]
    Ellipse(Box<models::EllipseNode>),
    #[serde(rename="EMBED")]
    Embed(Box<models::EmbedNode>),
    #[serde(rename="FRAME")]
    Frame(Box<models::FrameNode>),
    #[serde(rename="GROUP")]
    Group(Box<models::GroupNode>),
    #[serde(rename="INSTANCE")]
    Instance(Box<models::InstanceNode>),
    #[serde(rename="LINE")]
    Line(Box<models::LineNode>),
    #[serde(rename="LINK_UNFURL")]
    LinkUnfurl(Box<models::LinkUnfurlNode>),
    #[serde(rename="RECTANGLE")]
    Rectangle(Box<models::RectangleNode>),
    #[serde(rename="REGULAR_POLYGON")]
    RegularPolygon(Box<models::RegularPolygonNode>),
    #[serde(rename="SECTION")]
    Section(Box<models::SectionNode>),
    #[serde(rename="SHAPE_WITH_TEXT")]
    ShapeWithText(Box<models::ShapeWithTextNode>),
    #[serde(rename="SLICE")]
    Slice(Box<models::SliceNode>),
    #[serde(rename="STAR")]
    Star(Box<models::StarNode>),
    #[serde(rename="STICKY")]
    Sticky(Box<models::StickyNode>),
    #[serde(rename="TABLE")]
    Table(Box<models::TableNode>),
    #[serde(rename="TABLE_CELL")]
    TableCell(Box<models::TableCellNode>),
    #[serde(rename="TEXT")]
    Text(Box<models::TextNode>),
    #[serde(rename="TEXT_PATH")]
    TextPath(Box<models::TextPathNode>),
    #[serde(rename="TRANSFORM_GROUP")]
    TransformGroup(Box<models::TransformGroupNode>),
    #[serde(rename="VECTOR")]
    Vector(Box<models::VectorNode>),
    #[serde(rename="WASHI_TAPE")]
    WashiTape(Box<models::WashiTapeNode>),
    #[serde(rename="WIDGET")]
    Widget(Box<models::WidgetNode>),
}

```
We're interested in 3 the following: 

`Vector` is the general class of mathematical objects defining a shape. Figma defines several first class citizens within this grouping: `Ellipse`, `Line`, `Rectangle`, `RegularPolygon` Exported as svgs, these respectively are defined as a `<circle>` / `<ellipse>`, `<line>`, `<rect>`, `<polygon>`. The `Vector` in the general sesne can be represented as a `<path>`. 

A `Frame` is effectively a `<div>`, with `Group`/`Frame` beign used interchangeably for `<g>` within svgs.
A `Frame` defined a new coordinate system. By default, a `Frame` defines what css refers to as a relative coordinate system. i.e. the children are absolutely positoned in relation to the origin of the frame. Auto-layout can be appled, which is analogous to css flexbox or grid.


`Text` represents rich text: raw text composed with spans of styling. 



# Text


A text node is quite a beast:

```rust
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextNode {
    /// A string uniquely identifying this node within the document.
    #[serde(rename = "id")]
    pub id: String,
    /// The name given to the node by the user in the tool.
    #[serde(rename = "name")]
    pub name: String,
    /// Whether or not the node is visible on the canvas.
    #[serde(rename = "visible", skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    /// If true, layer is locked and cannot be edited
    #[serde(rename = "locked", skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    /// Whether the layer is fixed while the parent is scrolling
    #[serde(rename = "isFixed", skip_serializing_if = "Option::is_none")]
    pub is_fixed: Option<bool>,
    /// How layer should be treated when the frame is resized
    #[serde(rename = "scrollBehavior")]
    pub scroll_behavior: ScrollBehavior,
    /// The rotation of the node, if not 0.
    #[serde(rename = "rotation", skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f64>,
    /// A mapping of a layer's property to component property name of component properties attached to this node. The component property name can be used to look up more information on the corresponding component's or component set's componentPropertyDefinitions.
    #[serde(rename = "componentPropertyReferences", skip_serializing_if = "Option::is_none")]
    pub component_property_references: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "pluginData", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub plugin_data: Option<Option<serde_json::Value>>,
    #[serde(rename = "sharedPluginData", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub shared_plugin_data: Option<Option<serde_json::Value>>,
    #[serde(rename = "boundVariables", skip_serializing_if = "Option::is_none")]
    pub bound_variables: Option<Box<models::IsLayerTraitBoundVariables>>,
    /// A mapping of variable collection ID to mode ID representing the explicitly set modes for this node.
    #[serde(rename = "explicitVariableModes", skip_serializing_if = "Option::is_none")]
    pub explicit_variable_modes: Option<std::collections::HashMap<String, String>>,
    /// How this node blends with nodes behind it in the scene (see blend mode section for more details)
    #[serde(rename = "blendMode")]
    pub blend_mode: models::BlendMode,
    /// Opacity of the node
    #[serde(rename = "opacity", skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    /// Keep height and width constrained to same ratio.
    #[serde(rename = "preserveRatio", skip_serializing_if = "Option::is_none")]
    pub preserve_ratio: Option<bool>,
    /// Horizontal and vertical layout constraints for node.
    #[serde(rename = "constraints", skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Box<models::LayoutConstraint>>,
    /// A transformation matrix is standard way in computer graphics to represent translation and rotation. These are the top two rows of a 3x3 matrix. The bottom row of the matrix is assumed to be [0, 0, 1]. This is known as an affine transform and is enough to represent translation, rotation, and skew.  The identity transform is [[1, 0, 0], [0, 1, 0]].  A translation matrix will typically look like:  ``` [[1, 0, tx],   [0, 1, ty]] ```  and a rotation matrix will typically look like:  ``` [[cos(angle), sin(angle), 0],   [-sin(angle), cos(angle), 0]] ```  Another way to think about this transform is as three vectors:  - The x axis (t[0][0], t[1][0]) - The y axis (t[0][1], t[1][1]) - The translation offset (t[0][2], t[1][2])  The most common usage of the Transform matrix is the `relativeTransform property`. This particular usage of the matrix has a few additional restrictions. The translation offset can take on any value but we do enforce that the axis vectors are unit vectors (i.e. have length 1). The axes are not required to be at 90° angles to each other.
    #[serde(rename = "relativeTransform", skip_serializing_if = "Option::is_none")]
    pub relative_transform: Option<Vec<Vec<f64>>>,
    /// Width and height of element. This is different from the width and height of the bounding box in that the absolute bounding box represents the element after scaling and rotation. Only present if `geometry=paths` is passed.
    #[serde(rename = "size", skip_serializing_if = "Option::is_none")]
    pub size: Option<Box<models::Vector>>,
    ///  Determines if the layer should stretch along the parent's counter axis. This property is only provided for direct children of auto-layout frames.  - `INHERIT` - `STRETCH`  In previous versions of auto layout, determined how the layer is aligned inside an auto-layout frame. This property is only provided for direct children of auto-layout frames.  - `MIN` - `CENTER` - `MAX` - `STRETCH`  In horizontal auto-layout frames, \"MIN\" and \"MAX\" correspond to \"TOP\" and \"BOTTOM\". In vertical auto-layout frames, \"MIN\" and \"MAX\" correspond to \"LEFT\" and \"RIGHT\".
    #[serde(rename = "layoutAlign", skip_serializing_if = "Option::is_none")]
    pub layout_align: Option<LayoutAlign>,
    /// This property is applicable only for direct children of auto-layout frames, ignored otherwise. Determines whether a layer should stretch along the parent's primary axis. A `0` corresponds to a fixed size and `1` corresponds to stretch.
    #[serde(rename = "layoutGrow", skip_serializing_if = "Option::is_none")]
    pub layout_grow: Option<f64>,
    /// Determines whether a layer's size and position should be determined by auto-layout settings or manually adjustable.
    #[serde(rename = "layoutPositioning", skip_serializing_if = "Option::is_none")]
    pub layout_positioning: Option<LayoutPositioning>,
    /// The minimum width of the frame. This property is only applicable for auto-layout frames or direct children of auto-layout frames.
    #[serde(rename = "minWidth", skip_serializing_if = "Option::is_none")]
    pub min_width: Option<f64>,
    /// The maximum width of the frame. This property is only applicable for auto-layout frames or direct children of auto-layout frames.
    #[serde(rename = "maxWidth", skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f64>,
    /// The minimum height of the frame. This property is only applicable for auto-layout frames or direct children of auto-layout frames.
    #[serde(rename = "minHeight", skip_serializing_if = "Option::is_none")]
    pub min_height: Option<f64>,
    /// The maximum height of the frame. This property is only applicable for auto-layout frames or direct children of auto-layout frames.
    #[serde(rename = "maxHeight", skip_serializing_if = "Option::is_none")]
    pub max_height: Option<f64>,
    /// The horizontal sizing setting on this auto-layout frame or frame child. - `FIXED` - `HUG`: only valid on auto-layout frames and text nodes - `FILL`: only valid on auto-layout frame children
    #[serde(rename = "layoutSizingHorizontal", skip_serializing_if = "Option::is_none")]
    pub layout_sizing_horizontal: Option<LayoutSizingHorizontal>,
    /// The vertical sizing setting on this auto-layout frame or frame child. - `FIXED` - `HUG`: only valid on auto-layout frames and text nodes - `FILL`: only valid on auto-layout frame children
    #[serde(rename = "layoutSizingVertical", skip_serializing_if = "Option::is_none")]
    pub layout_sizing_vertical: Option<LayoutSizingVertical>,
    /// An array of fill paints applied to the node.
    #[serde(rename = "fills")]
    pub fills: Vec<models::Paint>,
    /// A mapping of a StyleType to style ID (see Style) of styles present on this node. The style ID can be used to look up more information about the style in the top-level styles field.
    #[serde(rename = "styles", skip_serializing_if = "Option::is_none")]
    pub styles: Option<std::collections::HashMap<String, String>>,
    /// An array of stroke paints applied to the node.
    #[serde(rename = "strokes", skip_serializing_if = "Option::is_none")]
    pub strokes: Option<Vec<models::Paint>>,
    /// The weight of strokes on the node.
    #[serde(rename = "strokeWeight", skip_serializing_if = "Option::is_none")]
    pub stroke_weight: Option<f64>,
    /// Position of stroke relative to vector outline, as a string enum  - `INSIDE`: stroke drawn inside the shape boundary - `OUTSIDE`: stroke drawn outside the shape boundary - `CENTER`: stroke drawn centered along the shape boundary
    #[serde(rename = "strokeAlign", skip_serializing_if = "Option::is_none")]
    pub stroke_align: Option<StrokeAlign>,
    /// A string enum with value of \"MITER\", \"BEVEL\", or \"ROUND\", describing how corners in vector paths are rendered.
    #[serde(rename = "strokeJoin", skip_serializing_if = "Option::is_none")]
    pub stroke_join: Option<StrokeJoin>,
    /// An array of floating point numbers describing the pattern of dash length and gap lengths that the vector stroke will use when drawn.  For example a value of [1, 2] indicates that the stroke will be drawn with a dash of length 1 followed by a gap of length 2, repeated.
    #[serde(rename = "strokeDashes", skip_serializing_if = "Option::is_none")]
    pub stroke_dashes: Option<Vec<f64>>,
    /// Only specified if parameter `geometry=paths` is used. An array of paths representing the object fill.
    #[serde(rename = "fillGeometry", skip_serializing_if = "Option::is_none")]
    pub fill_geometry: Option<Vec<models::Path>>,
    /// Only specified if parameter `geometry=paths` is used. An array of paths representing the object stroke.
    #[serde(rename = "strokeGeometry", skip_serializing_if = "Option::is_none")]
    pub stroke_geometry: Option<Vec<models::Path>>,
    /// A string enum describing the end caps of vector paths.
    #[serde(rename = "strokeCap", skip_serializing_if = "Option::is_none")]
    pub stroke_cap: Option<StrokeCap>,
    /// Only valid if `strokeJoin` is \"MITER\". The corner angle, in degrees, below which `strokeJoin` will be set to \"BEVEL\" to avoid super sharp corners. By default this is 28.96 degrees.
    #[serde(rename = "strokeMiterAngle", skip_serializing_if = "Option::is_none")]
    pub stroke_miter_angle: Option<f64>,
    /// An array of export settings representing images to export from the node.
    #[serde(rename = "exportSettings", skip_serializing_if = "Option::is_none")]
    pub export_settings: Option<Vec<models::ExportSetting>>,
    /// An array of effects attached to this node (see effects section for more details)
    #[serde(rename = "effects")]
    pub effects: Vec<models::Effect>,
    /// Does this node mask sibling nodes in front of it?
    #[serde(rename = "isMask", skip_serializing_if = "Option::is_none")]
    pub is_mask: Option<bool>,
    /// If this layer is a mask, this property describes the operation used to mask the layer's siblings. The value may be one of the following:  - ALPHA: the mask node's alpha channel will be used to determine the opacity of each pixel in the masked result. - VECTOR: if the mask node has visible fill paints, every pixel inside the node's fill regions will be fully visible in the masked result. If the mask has visible stroke paints, every pixel inside the node's stroke regions will be fully visible in the masked result. - LUMINANCE: the luminance value of each pixel of the mask node will be used to determine the opacity of that pixel in the masked result.
    #[serde(rename = "maskType", skip_serializing_if = "Option::is_none")]
    pub mask_type: Option<MaskType>,
    /// True if maskType is VECTOR. This field is deprecated; use maskType instead.
    #[serde(rename = "isMaskOutline", skip_serializing_if = "Option::is_none")]
    pub is_mask_outline: Option<bool>,
    /// Node ID of node to transition to in prototyping
    #[serde(rename = "transitionNodeID", skip_serializing_if = "Option::is_none")]
    pub transition_node_id: Option<String>,
    /// The duration of the prototyping transition on this node (in milliseconds). This will override the default transition duration on the prototype, for this node.
    #[serde(rename = "transitionDuration", skip_serializing_if = "Option::is_none")]
    pub transition_duration: Option<f64>,
    /// The easing curve used in the prototyping transition on this node.
    #[serde(rename = "transitionEasing", skip_serializing_if = "Option::is_none")]
    pub transition_easing: Option<models::EasingType>,
    /// The raw characters in the text node.
    #[serde(rename = "characters")]
    pub characters: String,
    /// Style of text including font family and weight.
    #[serde(rename = "style")]
    pub style: Box<models::TypeStyle>,
    /// The array corresponds to characters in the text box, where each element references the 'styleOverrideTable' to apply specific styles to each character. The array's length can be less than or equal to the number of characters due to the removal of trailing zeros. Elements with a value of 0 indicate characters that use the default type style. If the array is shorter than the total number of characters, the characters beyond the array's length also use the default style.
    #[serde(rename = "characterStyleOverrides")]
    pub character_style_overrides: Vec<f64>,
    /// Internal property, preserved for backward compatibility. Avoid using this value.
    #[serde(rename = "layoutVersion", skip_serializing_if = "Option::is_none")]
    pub layout_version: Option<f64>,
    /// Map from ID to TypeStyle for looking up style overrides.
    #[serde(rename = "styleOverrideTable")]
    pub style_override_table: std::collections::HashMap<String, models::TypeStyle>,
    /// An array with the same number of elements as lines in the text node, where lines are delimited by newline or paragraph separator characters. Each element in the array corresponds to the list type of a specific line. List types are represented as string enums with one of these possible values:  - `NONE`: Not a list item. - `ORDERED`: Text is an ordered list (numbered). - `UNORDERED`: Text is an unordered list (bulleted).
    #[serde(rename = "lineTypes")]
    pub line_types: Vec<LineTypes>,
    /// An array with the same number of elements as lines in the text node, where lines are delimited by newline or paragraph separator characters. Each element in the array corresponds to the indentation level of a specific line.
    #[serde(rename = "lineIndentations")]
    pub line_indentations: Vec<f64>,
}
```

This is a fairly dizzng array of different proprties, which we would have to faithfully convert to HTML/SVG in an export. The most important are of course the characters themsleves, and the style. 


```rust
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TypeStyle {
    /// Font family of text (standard name).
    #[serde(rename = "fontFamily", skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    /// PostScript font name.
    #[serde(rename = "fontPostScriptName", skip_serializing_if = "Option::is_none")]
    pub font_post_script_name: Option<String>,
    /// Describes visual weight or emphasis, such as Bold or Italic.
    #[serde(rename = "fontStyle", skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    /// Whether or not text is italicized.
    #[serde(rename = "italic", skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    /// Numeric font weight.
    #[serde(rename = "fontWeight", skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<f64>,
    /// Font size in px.
    #[serde(rename = "fontSize", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    /// Text casing applied to the node, default is the original casing.
    #[serde(rename = "textCase", skip_serializing_if = "Option::is_none")]
    pub text_case: Option<TextCase>,
    /// Horizontal text alignment as string enum.
    #[serde(rename = "textAlignHorizontal", skip_serializing_if = "Option::is_none")]
    pub text_align_horizontal: Option<TextAlignHorizontal>,
    /// Vertical text alignment as string enum.
    #[serde(rename = "textAlignVertical", skip_serializing_if = "Option::is_none")]
    pub text_align_vertical: Option<TextAlignVertical>,
    /// Space between characters in px.
    #[serde(rename = "letterSpacing", skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<f64>,
    /// An array of fill paints applied to the characters.
    #[serde(rename = "fills", skip_serializing_if = "Option::is_none")]
    pub fills: Option<Vec<models::Paint>>,
    /// Link to a URL or frame.
    #[serde(rename = "hyperlink", skip_serializing_if = "Option::is_none")]
    pub hyperlink: Option<Box<models::Hyperlink>>,
    /// A map of OpenType feature flags to 1 or 0, 1 if it is enabled and 0 if it is disabled. Note that some flags aren't reflected here. For example, SMCP (small caps) is still represented by the `textCase` field.
    #[serde(rename = "opentypeFlags", skip_serializing_if = "Option::is_none")]
    pub opentype_flags: Option<std::collections::HashMap<String, f64>>,
    /// Indicates how the font weight was overridden when there is a text style override.
    #[serde(rename = "semanticWeight", skip_serializing_if = "Option::is_none")]
    pub semantic_weight: Option<SemanticWeight>,
    /// Indicates how the font style was overridden when there is a text style override.
    #[serde(rename = "semanticItalic", skip_serializing_if = "Option::is_none")]
    pub semantic_italic: Option<SemanticItalic>,
    /// Space between paragraphs in px, 0 if not present.
    #[serde(rename = "paragraphSpacing", skip_serializing_if = "Option::is_none")]
    pub paragraph_spacing: Option<f64>,
    /// Paragraph indentation in px, 0 if not present.
    #[serde(rename = "paragraphIndent", skip_serializing_if = "Option::is_none")]
    pub paragraph_indent: Option<f64>,
    /// Space between list items in px, 0 if not present.
    #[serde(rename = "listSpacing", skip_serializing_if = "Option::is_none")]
    pub list_spacing: Option<f64>,
    /// Text decoration applied to the node, default is none.
    #[serde(rename = "textDecoration", skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<TextDecoration>,
    /// Dimensions along which text will auto resize, default is that the text does not auto-resize. TRUNCATE means that the text will be shortened and trailing text will be replaced with \"…\" if the text contents is larger than the bounds. `TRUNCATE` as a return value is deprecated and will be removed in a future version. Read from `textTruncation` instead.
    #[serde(rename = "textAutoResize", skip_serializing_if = "Option::is_none")]
    pub text_auto_resize: Option<TextAutoResize>,
    /// Whether this text node will truncate with an ellipsis when the text contents is larger than the text node.
    #[serde(rename = "textTruncation", skip_serializing_if = "Option::is_none")]
    pub text_truncation: Option<TextTruncation>,
    /// When `textTruncation: \"ENDING\"` is set, `maxLines` determines how many lines a text node can grow to before it truncates.
    #[serde(rename = "maxLines", skip_serializing_if = "Option::is_none")]
    pub max_lines: Option<f64>,
    /// Line height in px.
    #[serde(rename = "lineHeightPx", skip_serializing_if = "Option::is_none")]
    pub line_height_px: Option<f64>,
    /// Line height as a percentage of normal line height. This is deprecated; in a future version of the API only lineHeightPx and lineHeightPercentFontSize will be returned.
    #[serde(rename = "lineHeightPercent", skip_serializing_if = "Option::is_none")]
    pub line_height_percent: Option<f64>,
    /// Line height as a percentage of the font size. Only returned when `lineHeightPercent` (deprecated) is not 100.
    #[serde(rename = "lineHeightPercentFontSize", skip_serializing_if = "Option::is_none")]
    pub line_height_percent_font_size: Option<f64>,
    /// The unit of the line height value specified by the user.
    #[serde(rename = "lineHeightUnit", skip_serializing_if = "Option::is_none")]
    pub line_height_unit: Option<LineHeightUnit>,
    /// Whether or not this style has overrides over a text style. The possible fields to override are semanticWeight, semanticItalic, hyperlink, and textDecoration. If this is true, then those fields are overrides if present.
    #[serde(rename = "isOverrideOverTextStyle", skip_serializing_if = "Option::is_none")]
    pub is_override_over_text_style: Option<bool>,
    #[serde(rename = "boundVariables", skip_serializing_if = "Option::is_none")]
    pub bound_variables: Option<Box<models::TypeStyleAllOfBoundVariables>>,
}
```



# Frame



```rust
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameNode {
    /// A string uniquely identifying this node within the document.
    #[serde(rename = "id")]
    pub id: String,
    /// The name given to the node by the user in the tool.
    #[serde(rename = "name")]
    pub name: String,
    /// Whether or not the node is visible on the canvas.
    #[serde(rename = "visible", skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    /// If true, layer is locked and cannot be edited
    #[serde(rename = "locked", skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    /// Whether the layer is fixed while the parent is scrolling
    #[serde(rename = "isFixed", skip_serializing_if = "Option::is_none")]
    pub is_fixed: Option<bool>,
    /// How layer should be treated when the frame is resized
    #[serde(rename = "scrollBehavior")]
    pub scroll_behavior: ScrollBehavior,
    /// The rotation of the node, if not 0.
    #[serde(rename = "rotation", skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f64>,
    /// A mapping of a layer's property to component property name of component properties attached to this node. The component property name can be used to look up more information on the corresponding component's or component set's componentPropertyDefinitions.
    #[serde(rename = "componentPropertyReferences", skip_serializing_if = "Option::is_none")]
    pub component_property_references: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "pluginData", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub plugin_data: Option<Option<serde_json::Value>>,
    #[serde(rename = "sharedPluginData", default, with = "::serde_with::rust::double_option", skip_serializing_if = "Option::is_none")]
    pub shared_plugin_data: Option<Option<serde_json::Value>>,
    #[serde(rename = "boundVariables", skip_serializing_if = "Option::is_none")]
    pub bound_variables: Option<Box<models::IsLayerTraitBoundVariables>>,
    /// A mapping of variable collection ID to mode ID representing the explicitly set modes for this node.
    #[serde(rename = "explicitVariableModes", skip_serializing_if = "Option::is_none")]
    pub explicit_variable_modes: Option<std::collections::HashMap<String, String>>,
    /// How this node blends with nodes behind it in the scene (see blend mode section for more details)
    #[serde(rename = "blendMode")]
    pub blend_mode: models::BlendMode,
    /// Opacity of the node
    #[serde(rename = "opacity", skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    /// An array of nodes that are direct children of this node
    #[serde(rename = "children")]
    pub children: Vec<models::SubcanvasNode>,
    /// Keep height and width constrained to same ratio.
    #[serde(rename = "preserveRatio", skip_serializing_if = "Option::is_none")]
    pub preserve_ratio: Option<bool>,
    /// Horizontal and vertical layout constraints for node.
    #[serde(rename = "constraints", skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Box<models::LayoutConstraint>>,
    /// A transformation matrix is standard way in computer graphics to represent translation and rotation. These are the top two rows of a 3x3 matrix. The bottom row of the matrix is assumed to be [0, 0, 1]. This is known as an affine transform and is enough to represent translation, rotation, and skew.  The identity transform is [[1, 0, 0], [0, 1, 0]].  A translation matrix will typically look like:  ``` [[1, 0, tx],   [0, 1, ty]] ```  and a rotation matrix will typically look like:  ``` [[cos(angle), sin(angle), 0],   [-sin(angle), cos(angle), 0]] ```  Another way to think about this transform is as three vectors:  - The x axis (t[0][0], t[1][0]) - The y axis (t[0][1], t[1][1]) - The translation offset (t[0][2], t[1][2])  The most common usage of the Transform matrix is the `relativeTransform property`. This particular usage of the matrix has a few additional restrictions. The translation offset can take on any value but we do enforce that the axis vectors are unit vectors (i.e. have length 1). The axes are not required to be at 90° angles to each other.
    #[serde(rename = "relativeTransform", skip_serializing_if = "Option::is_none")]
    pub relative_transform: Option<Vec<Vec<f64>>>,
    /// Width and height of element. This is different from the width and height of the bounding box in that the absolute bounding box represents the element after scaling and rotation. Only present if `geometry=paths` is passed.
    #[serde(rename = "size", skip_serializing_if = "Option::is_none")]
    pub size: Option<Box<models::Vector>>,
    ///  Determines if the layer should stretch along the parent's counter axis. This property is only provided for direct children of auto-layout frames.  - `INHERIT` - `STRETCH`  In previous versions of auto layout, determined how the layer is aligned inside an auto-layout frame. This property is only provided for direct children of auto-layout frames.  - `MIN` - `CENTER` - `MAX` - `STRETCH`  In horizontal auto-layout frames, \"MIN\" and \"MAX\" correspond to \"TOP\" and \"BOTTOM\". In vertical auto-layout frames, \"MIN\" and \"MAX\" correspond to \"LEFT\" and \"RIGHT\".
    #[serde(rename = "layoutAlign", skip_serializing_if = "Option::is_none")]
    pub layout_align: Option<LayoutAlign>,
    /// This property is applicable only for direct children of auto-layout frames, ignored otherwise. Determines whether a layer should stretch along the parent's primary axis. A `0` corresponds to a fixed size and `1` corresponds to stretch.
    #[serde(rename = "layoutGrow", skip_serializing_if = "Option::is_none")]
    pub layout_grow: Option<f64>,
    /// Determines whether a layer's size and position should be determined by auto-layout settings or manually adjustable.
    #[serde(rename = "layoutPositioning", skip_serializing_if = "Option::is_none")]
    pub layout_positioning: Option<LayoutPositioning>,
    /// The minimum width of the frame. This property is only applicable for auto-layout frames or direct children of auto-layout frames.
    #[serde(rename = "minWidth", skip_serializing_if = "Option::is_none")]
    pub min_width: Option<f64>,
    /// The maximum width of the frame. This property is only applicable for auto-layout frames or direct children of auto-layout frames.
    #[serde(rename = "maxWidth", skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f64>,
    /// The minimum height of the frame. This property is only applicable for auto-layout frames or direct children of auto-layout frames.
    #[serde(rename = "minHeight", skip_serializing_if = "Option::is_none")]
    pub min_height: Option<f64>,
    /// The maximum height of the frame. This property is only applicable for auto-layout frames or direct children of auto-layout frames.
    #[serde(rename = "maxHeight", skip_serializing_if = "Option::is_none")]
    pub max_height: Option<f64>,
    /// The horizontal sizing setting on this auto-layout frame or frame child. - `FIXED` - `HUG`: only valid on auto-layout frames and text nodes - `FILL`: only valid on auto-layout frame children
    #[serde(rename = "layoutSizingHorizontal", skip_serializing_if = "Option::is_none")]
    pub layout_sizing_horizontal: Option<LayoutSizingHorizontal>,
    /// The vertical sizing setting on this auto-layout frame or frame child. - `FIXED` - `HUG`: only valid on auto-layout frames and text nodes - `FILL`: only valid on auto-layout frame children
    #[serde(rename = "layoutSizingVertical", skip_serializing_if = "Option::is_none")]
    pub layout_sizing_vertical: Option<LayoutSizingVertical>,
    /// Whether or not this node clip content outside of its bounds
    #[serde(rename = "clipsContent")]
    pub clips_content: bool,
    /// Background of the node. This is deprecated, as backgrounds for frames are now in the `fills` field.
    #[serde(rename = "background", skip_serializing_if = "Option::is_none")]
    pub background: Option<Vec<models::Paint>>,
    /// Background color of the node. This is deprecated, as frames now support more than a solid color as a background. Please use the `fills` field instead.
    #[serde(rename = "backgroundColor", skip_serializing_if = "Option::is_none")]
    pub background_color: Option<Box<models::Rgba>>,
    /// An array of layout grids attached to this node (see layout grids section for more details). GROUP nodes do not have this attribute
    #[serde(rename = "layoutGrids", skip_serializing_if = "Option::is_none")]
    pub layout_grids: Option<Vec<models::LayoutGrid>>,
    /// Whether a node has primary axis scrolling, horizontal or vertical.
    #[serde(rename = "overflowDirection", skip_serializing_if = "Option::is_none")]
    pub overflow_direction: Option<OverflowDirection>,
    /// Whether this layer uses auto-layout to position its children.
    #[serde(rename = "layoutMode", skip_serializing_if = "Option::is_none")]
    pub layout_mode: Option<LayoutMode>,
    /// Whether the primary axis has a fixed length (determined by the user) or an automatic length (determined by the layout engine). This property is only applicable for auto-layout frames.
    #[serde(rename = "primaryAxisSizingMode", skip_serializing_if = "Option::is_none")]
    pub primary_axis_sizing_mode: Option<PrimaryAxisSizingMode>,
    /// Whether the counter axis has a fixed length (determined by the user) or an automatic length (determined by the layout engine). This property is only applicable for auto-layout frames.
    #[serde(rename = "counterAxisSizingMode", skip_serializing_if = "Option::is_none")]
    pub counter_axis_sizing_mode: Option<CounterAxisSizingMode>,
    /// Determines how the auto-layout frame's children should be aligned in the primary axis direction. This property is only applicable for auto-layout frames.
    #[serde(rename = "primaryAxisAlignItems", skip_serializing_if = "Option::is_none")]
    pub primary_axis_align_items: Option<PrimaryAxisAlignItems>,
    /// Determines how the auto-layout frame's children should be aligned in the counter axis direction. This property is only applicable for auto-layout frames.
    #[serde(rename = "counterAxisAlignItems", skip_serializing_if = "Option::is_none")]
    pub counter_axis_align_items: Option<CounterAxisAlignItems>,
    /// The padding between the left border of the frame and its children. This property is only applicable for auto-layout frames.
    #[serde(rename = "paddingLeft", skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<f64>,
    /// The padding between the right border of the frame and its children. This property is only applicable for auto-layout frames.
    #[serde(rename = "paddingRight", skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<f64>,
    /// The padding between the top border of the frame and its children. This property is only applicable for auto-layout frames.
    #[serde(rename = "paddingTop", skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<f64>,
    /// The padding between the bottom border of the frame and its children. This property is only applicable for auto-layout frames.
    #[serde(rename = "paddingBottom", skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<f64>,
    /// The distance between children of the frame. Can be negative. This property is only applicable for auto-layout frames.
    #[serde(rename = "itemSpacing", skip_serializing_if = "Option::is_none")]
    pub item_spacing: Option<f64>,
    /// Determines the canvas stacking order of layers in this frame. When true, the first layer will be draw on top. This property is only applicable for auto-layout frames.
    #[serde(rename = "itemReverseZIndex", skip_serializing_if = "Option::is_none")]
    pub item_reverse_z_index: Option<bool>,
    /// Determines whether strokes are included in layout calculations. When true, auto-layout frames behave like css \"box-sizing: border-box\". This property is only applicable for auto-layout frames.
    #[serde(rename = "strokesIncludedInLayout", skip_serializing_if = "Option::is_none")]
    pub strokes_included_in_layout: Option<bool>,
    /// Whether this auto-layout frame has wrapping enabled.
    #[serde(rename = "layoutWrap", skip_serializing_if = "Option::is_none")]
    pub layout_wrap: Option<LayoutWrap>,
    /// The distance between wrapped tracks of an auto-layout frame. This property is only applicable for auto-layout frames with `layoutWrap: \"WRAP\"`
    #[serde(rename = "counterAxisSpacing", skip_serializing_if = "Option::is_none")]
    pub counter_axis_spacing: Option<f64>,
    /// Determines how the auto-layout frame’s wrapped tracks should be aligned in the counter axis direction. This property is only applicable for auto-layout frames with `layoutWrap: \"WRAP\"`.
    #[serde(rename = "counterAxisAlignContent", skip_serializing_if = "Option::is_none")]
    pub counter_axis_align_content: Option<CounterAxisAlignContent>,
    /// Radius of each corner if a single radius is set for all corners
    #[serde(rename = "cornerRadius", skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f64>,
    /// A value that lets you control how \"smooth\" the corners are. Ranges from 0 to 1. 0 is the default and means that the corner is perfectly circular. A value of 0.6 means the corner matches the iOS 7 \"squircle\" icon shape. Other values produce various other curves.
    #[serde(rename = "cornerSmoothing", skip_serializing_if = "Option::is_none")]
    pub corner_smoothing: Option<f64>,
    /// Array of length 4 of the radius of each corner of the frame, starting in the top left and proceeding clockwise.  Values are given in the order top-left, top-right, bottom-right, bottom-left.
    #[serde(rename = "rectangleCornerRadii", skip_serializing_if = "Option::is_none")]
    pub rectangle_corner_radii: Option<Vec<f64>>,
    /// An array of fill paints applied to the node.
    #[serde(rename = "fills")]
    pub fills: Vec<models::Paint>,
    /// A mapping of a StyleType to style ID (see Style) of styles present on this node. The style ID can be used to look up more information about the style in the top-level styles field.
    #[serde(rename = "styles", skip_serializing_if = "Option::is_none")]
    pub styles: Option<std::collections::HashMap<String, String>>,
    /// An array of stroke paints applied to the node.
    #[serde(rename = "strokes", skip_serializing_if = "Option::is_none")]
    pub strokes: Option<Vec<models::Paint>>,
    /// The weight of strokes on the node.
    #[serde(rename = "strokeWeight", skip_serializing_if = "Option::is_none")]
    pub stroke_weight: Option<f64>,
    /// Position of stroke relative to vector outline, as a string enum  - `INSIDE`: stroke drawn inside the shape boundary - `OUTSIDE`: stroke drawn outside the shape boundary - `CENTER`: stroke drawn centered along the shape boundary
    #[serde(rename = "strokeAlign", skip_serializing_if = "Option::is_none")]
    pub stroke_align: Option<StrokeAlign>,
    /// A string enum with value of \"MITER\", \"BEVEL\", or \"ROUND\", describing how corners in vector paths are rendered.
    #[serde(rename = "strokeJoin", skip_serializing_if = "Option::is_none")]
    pub stroke_join: Option<StrokeJoin>,
    /// An array of floating point numbers describing the pattern of dash length and gap lengths that the vector stroke will use when drawn.  For example a value of [1, 2] indicates that the stroke will be drawn with a dash of length 1 followed by a gap of length 2, repeated.
    #[serde(rename = "strokeDashes", skip_serializing_if = "Option::is_none")]
    pub stroke_dashes: Option<Vec<f64>>,
    /// Only specified if parameter `geometry=paths` is used. An array of paths representing the object fill.
    #[serde(rename = "fillGeometry", skip_serializing_if = "Option::is_none")]
    pub fill_geometry: Option<Vec<models::Path>>,
    /// Only specified if parameter `geometry=paths` is used. An array of paths representing the object stroke.
    #[serde(rename = "strokeGeometry", skip_serializing_if = "Option::is_none")]
    pub stroke_geometry: Option<Vec<models::Path>>,
    /// A string enum describing the end caps of vector paths.
    #[serde(rename = "strokeCap", skip_serializing_if = "Option::is_none")]
    pub stroke_cap: Option<StrokeCap>,
    /// Only valid if `strokeJoin` is \"MITER\". The corner angle, in degrees, below which `strokeJoin` will be set to \"BEVEL\" to avoid super sharp corners. By default this is 28.96 degrees.
    #[serde(rename = "strokeMiterAngle", skip_serializing_if = "Option::is_none")]
    pub stroke_miter_angle: Option<f64>,
    /// An array of export settings representing images to export from the node.
    #[serde(rename = "exportSettings", skip_serializing_if = "Option::is_none")]
    pub export_settings: Option<Vec<models::ExportSetting>>,
    /// An array of effects attached to this node (see effects section for more details)
    #[serde(rename = "effects")]
    pub effects: Vec<models::Effect>,
    /// Does this node mask sibling nodes in front of it?
    #[serde(rename = "isMask", skip_serializing_if = "Option::is_none")]
    pub is_mask: Option<bool>,
    /// If this layer is a mask, this property describes the operation used to mask the layer's siblings. The value may be one of the following:  - ALPHA: the mask node's alpha channel will be used to determine the opacity of each pixel in the masked result. - VECTOR: if the mask node has visible fill paints, every pixel inside the node's fill regions will be fully visible in the masked result. If the mask has visible stroke paints, every pixel inside the node's stroke regions will be fully visible in the masked result. - LUMINANCE: the luminance value of each pixel of the mask node will be used to determine the opacity of that pixel in the masked result.
    #[serde(rename = "maskType", skip_serializing_if = "Option::is_none")]
    pub mask_type: Option<MaskType>,
    /// True if maskType is VECTOR. This field is deprecated; use maskType instead.
    #[serde(rename = "isMaskOutline", skip_serializing_if = "Option::is_none")]
    pub is_mask_outline: Option<bool>,
    /// Node ID of node to transition to in prototyping
    #[serde(rename = "transitionNodeID", skip_serializing_if = "Option::is_none")]
    pub transition_node_id: Option<String>,
    /// The duration of the prototyping transition on this node (in milliseconds). This will override the default transition duration on the prototype, for this node.
    #[serde(rename = "transitionDuration", skip_serializing_if = "Option::is_none")]
    pub transition_duration: Option<f64>,
    /// The easing curve used in the prototyping transition on this node.
    #[serde(rename = "transitionEasing", skip_serializing_if = "Option::is_none")]
    pub transition_easing: Option<models::EasingType>,
    /// An object including the top, bottom, left, and right stroke weights. Only returned if individual stroke weights are used.
    #[serde(rename = "individualStrokeWeights", skip_serializing_if = "Option::is_none")]
    pub individual_stroke_weights: Option<Box<models::StrokeWeights>>,
}
```


Figma make, for instance, exports a 'header' frame as the following: 

```tsx

function Header() {
  return (
    <div className="content-stretch flex items-center justify-between relative w-full" data-name="header">
      <div className="box-border content-stretch flex flex-col gap-[4px] items-center p-[20px] relative shadow-[0px_0px_40px_0px_#ffffff] shrink-0 flex-1 max-w-[904px]" data-name="title">
        <div className="font-['Georgia:Regular',_'Noto_Sans:Regular',_sans-serif] leading-[normal] relative shrink-0 text-[#440099] text-[40px] w-full" style={{ fontVariationSettings: "'CTGR' 0, 'wdth' 100, 'wght' 400" }}>
          <p className="mb-0">NTM Are Uniquely Suited to Survive in the </p>
          <p>Environment, Leading to NTM Transmission</p>
        </div>
      </div>
      <div className="box-border content-stretch flex items-center p-[20px] relative shadow-[0px_4px_4px_0px_rgba(0,0,0,0.25)] shrink-0" data-name="pagination">
        <div className="bg-[#440099] box-border content-stretch flex gap-[10px] items-center justify-center pl-[12px] pr-[8px] py-[4px] relative rounded-bl-[100px] rounded-tl-[100px] shrink-0" data-name="btn">
          <div aria-hidden="true" className="absolute border border-solid border-white inset-0 pointer-events-none rounded-bl-[100px] rounded-tl-[100px]" />
          <p className="font-['Inter:Semi_Bold',_sans-serif] font-semibold leading-[normal] not-italic relative shrink-0 text-[12px] text-nowrap text-white whitespace-pre">NTM Lung Disease</p>
        </div>
        <div className="bg-white box-border content-stretch flex gap-[10px] items-center justify-center pl-[8px] pr-[12px] py-[4px] relative rounded-br-[100px] rounded-tr-[100px] shrink-0" data-name="btn">
          <p className="font-['Inter:Semi_Bold',_sans-serif] font-semibold leading-[normal] not-italic relative shrink-0 text-[#440099] text-[12px] text-nowrap whitespace-pre">
            <span className="uppercase">screen</span> 3/30
          </p>
        </div>
      </div>
    </div>
  );
}


```

How do we create a mapping function that converts between representations?




# Vector

Put svg path string in extetnal typescript file for better readability. i.e. :

```ts
export default {
p10978a80: "M40 65C40.5523 65 41 64.3284 41 63.5C41 62.6716 40.5523 62 40 62C39.4477 62 39 62.6716 39 63.5C39 64.3284 39.4477 65 40 65Z",
p11365cc0: "M38.1866 44.6791H42.4679C42.9903 44.6791 43.4174 44.2553 43.4174 43.7296C43.4174 43.2039 42.9936 42.78 42.4679 42.78H38.1866C37.6641 42.78 37.237 43.2071 37.237 43.7296C37.237 44.252 37.6609 44.6791 38.1866 44.6791Z",
}
```

and then we can reference them in our tsx:

```tsx

<path d={svgPaths.p10978a80} fill="url(#paint0_linear_1_1239)" />

```

We can store the base64 png images, referenced by svgs in <image/> componnets in png files, and reference that







# Components

We may be missing a trick with components. We want reuseablty, and components presumably enable us to keep our codebase more DRY

