use figma_api::models::{TextNode, TypeStyle};

/// Analyzes text nodes for special formatting like superscripts and subscripts
pub struct TextAnalyzer;

impl TextAnalyzer {
    /// Detect superscript ranges in text based on character style overrides
    pub fn detect_superscripts(text: &TextNode) -> Vec<SuperscriptRange> {
        let mut ranges = Vec::new();

        if text.character_style_overrides.is_empty() || text.style_override_table.is_empty() {
            return ranges;
        }

        let characters: Vec<char> = text.characters.chars().collect();
        let mut current_range_start: Option<usize> = None;
        let mut current_method: Option<DetectionMethod> = None;

        for (i, _) in characters.iter().enumerate() {
            if let Some(style_id) = Self::get_style_id_for_character(text, i) {
                if let Some(style) = text.style_override_table.get(&style_id) {
                    let method = Self::detect_superscript_method(style, &text.style);

                    if method.is_some() {
                        // Start or continue superscript range
                        if current_range_start.is_none() {
                            current_range_start = Some(i);
                            current_method = method;
                        }
                    } else {
                        // End current range if exists
                        if let Some(start) = current_range_start {
                            ranges.push(SuperscriptRange {
                                start,
                                end: i,
                                detected_method: current_method
                                    .unwrap_or(DetectionMethod::FontSizeAndOffset),
                            });
                            current_range_start = None;
                            current_method = None;
                        }
                    }
                }
            } else {
                // No style override - end current range if exists
                if let Some(start) = current_range_start {
                    ranges.push(SuperscriptRange {
                        start,
                        end: i,
                        detected_method: current_method
                            .unwrap_or(DetectionMethod::FontSizeAndOffset),
                    });
                    current_range_start = None;
                    current_method = None;
                }
            }
        }

        // Handle range that extends to end of text
        if let Some(start) = current_range_start {
            ranges.push(SuperscriptRange {
                start,
                end: characters.len(),
                detected_method: current_method.unwrap_or(DetectionMethod::FontSizeAndOffset),
            });
        }

        ranges
    }

    /// Detect subscript ranges in text
    pub fn detect_subscripts(text: &TextNode) -> Vec<SubscriptRange> {
        let mut ranges = Vec::new();

        if text.character_style_overrides.is_empty() || text.style_override_table.is_empty() {
            return ranges;
        }

        let characters: Vec<char> = text.characters.chars().collect();
        let mut current_range_start: Option<usize> = None;
        let mut current_method: Option<DetectionMethod> = None;

        for (i, _) in characters.iter().enumerate() {
            if let Some(style_id) = Self::get_style_id_for_character(text, i) {
                if let Some(style) = text.style_override_table.get(&style_id) {
                    let method = Self::detect_subscript_method(style, &text.style);

                    if method.is_some() {
                        if current_range_start.is_none() {
                            current_range_start = Some(i);
                            current_method = method;
                        }
                    } else {
                        if let Some(start) = current_range_start {
                            ranges.push(SubscriptRange {
                                start,
                                end: i,
                                detected_method: current_method
                                    .unwrap_or(DetectionMethod::FontSizeAndOffset),
                            });
                            current_range_start = None;
                            current_method = None;
                        }
                    }
                }
            } else {
                if let Some(start) = current_range_start {
                    ranges.push(SubscriptRange {
                        start,
                        end: i,
                        detected_method: current_method
                            .unwrap_or(DetectionMethod::FontSizeAndOffset),
                    });
                    current_range_start = None;
                    current_method = None;
                }
            }
        }

        if let Some(start) = current_range_start {
            ranges.push(SubscriptRange {
                start,
                end: characters.len(),
                detected_method: current_method.unwrap_or(DetectionMethod::FontSizeAndOffset),
            });
        }

        ranges
    }

    /// Get style ID for a specific character index
    fn get_style_id_for_character(text: &TextNode, index: usize) -> Option<String> {
        if index < text.character_style_overrides.len() {
            let override_val = text.character_style_overrides[index];
            if override_val > 0.0 {
                Some(override_val.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Detect if a style represents a superscript
    fn detect_superscript_method(
        style: &TypeStyle,
        base_style: &TypeStyle,
    ) -> Option<DetectionMethod> {
        // Method 1: Smaller font size with vertical offset
        if let (Some(style_font_size), Some(base_font_size)) =
            (style.font_size, base_style.font_size)
        {
            if style_font_size < base_font_size * 0.8 {
                // Check if there's a vertical offset (would need additional analysis)
                return Some(DetectionMethod::FontSizeAndOffset);
            }
        }

        // Method 2: Baseline shift (if available in TypeStyle)
        // Note: This would require checking if TypeStyle has baseline_shift property
        // For now, we'll use font size as the primary indicator

        // Method 3: Letter spacing changes (sometimes used for superscripts)
        if let (Some(style_letter_spacing), Some(base_letter_spacing)) =
            (style.letter_spacing, base_style.letter_spacing)
        {
            if (style_letter_spacing - base_letter_spacing).abs() > 1.0 {
                return Some(DetectionMethod::FontSizeAndOffset);
            }
        }

        None
    }

    /// Detect if a style represents a subscript
    fn detect_subscript_method(
        style: &TypeStyle,
        base_style: &TypeStyle,
    ) -> Option<DetectionMethod> {
        // Similar to superscript but looking for different patterns
        if let (Some(style_font_size), Some(base_font_size)) =
            (style.font_size, base_style.font_size)
        {
            if style_font_size < base_font_size * 0.8 {
                // Subscripts are typically smaller and positioned below baseline
                return Some(DetectionMethod::FontSizeAndOffset);
            }
        }

        None
    }

    /// Analyze text for any special formatting
    pub fn analyze_text_formatting(text: &TextNode) -> TextFormattingAnalysis {
        let superscripts = Self::detect_superscripts(text);
        let subscripts = Self::detect_subscripts(text);

        TextFormattingAnalysis {
            superscripts: superscripts.clone(),
            subscripts: subscripts.clone(),
            has_special_formatting: !superscripts.is_empty() || !subscripts.is_empty(),
        }
    }
}

/// Represents a range of characters that should be formatted as superscript
#[derive(Debug, Clone)]
pub struct SuperscriptRange {
    pub start: usize,
    pub end: usize,
    pub detected_method: DetectionMethod,
}

/// Represents a range of characters that should be formatted as subscript
#[derive(Debug, Clone)]
pub struct SubscriptRange {
    pub start: usize,
    pub end: usize,
    pub detected_method: DetectionMethod,
}

/// Method used to detect special formatting
#[derive(Debug, Clone)]
pub enum DetectionMethod {
    FontSizeAndOffset,
    BaselineShift,
    VerticalTransform,
    LetterSpacing,
}

/// Complete analysis of text formatting
#[derive(Debug, Clone)]
pub struct TextFormattingAnalysis {
    pub superscripts: Vec<SuperscriptRange>,
    pub subscripts: Vec<SubscriptRange>,
    pub has_special_formatting: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use figma_api::models::TypeStyle;

    fn create_test_text_node() -> TextNode {
        TextNode {
            id: "test".to_string(),
            name: "Test Text".to_string(),
            characters: "H2O".to_string(),
            character_style_overrides: vec![0.0, 1.0, 0.0], // Middle character has override
            style_override_table: {
                let mut table = std::collections::HashMap::new();
                table.insert(
                    "1".to_string(),
                    TypeStyle {
                        font_size: Some(8.0), // Smaller than base
                        ..Default::default()
                    },
                );
                table
            },
            style: Box::new(TypeStyle {
                font_size: Some(12.0), // Base font size
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_detect_superscripts() {
        let text = create_test_text_node();
        let superscripts = TextAnalyzer::detect_superscripts(&text);

        assert_eq!(superscripts.len(), 1);
        assert_eq!(superscripts[0].start, 1);
        assert_eq!(superscripts[0].end, 2);
    }

    #[test]
    fn test_detect_subscripts() {
        let text = create_test_text_node();
        let subscripts = TextAnalyzer::detect_subscripts(&text);

        // Should detect the same range as potential subscript
        assert_eq!(subscripts.len(), 1);
    }

    #[test]
    fn test_analyze_text_formatting() {
        let text = create_test_text_node();
        let analysis = TextAnalyzer::analyze_text_formatting(&text);

        assert!(analysis.has_special_formatting);
        assert!(!analysis.superscripts.is_empty());
    }

    #[test]
    fn test_no_style_overrides() {
        let text = TextNode {
            id: "test".to_string(),
            name: "Test".to_string(),
            characters: "Hello".to_string(),
            character_style_overrides: vec![],
            style_override_table: std::collections::HashMap::new(),
            style: Box::new(TypeStyle::default()),
            ..Default::default()
        };

        let superscripts = TextAnalyzer::detect_superscripts(&text);
        let subscripts = TextAnalyzer::detect_subscripts(&text);

        assert!(superscripts.is_empty());
        assert!(subscripts.is_empty());
    }

    #[test]
    fn test_get_style_id_for_character() {
        let text = create_test_text_node();

        assert_eq!(TextAnalyzer::get_style_id_for_character(&text, 0), None);
        assert_eq!(
            TextAnalyzer::get_style_id_for_character(&text, 1),
            Some("1".to_string())
        );
        assert_eq!(TextAnalyzer::get_style_id_for_character(&text, 2), None);
    }
}
