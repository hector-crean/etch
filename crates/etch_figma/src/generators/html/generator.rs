use super::visitor::HtmlVisitor;
use crate::core::config::{CodeGenResult, CodeGenerator};
use std::collections::HashMap;
use std::path::PathBuf;

/// HTML code generator that creates HTML from HtmlVisitor results
pub struct HtmlGenerator {
    /// Whether to include DOCTYPE declaration
    include_doctype: bool,
    /// Whether to create separate files for each component
    separate_files: bool,
    /// Whether to only generate exportable components
    exportable_only: bool,
    /// HTML template wrapper
    template: Option<String>,
}

impl HtmlGenerator {
    pub fn new() -> Self {
        Self {
            include_doctype: true,
            separate_files: true,
            exportable_only: false,
            template: None,
        }
    }

    pub fn with_doctype(mut self, include: bool) -> Self {
        self.include_doctype = include;
        self
    }

    pub fn with_separate_files(mut self, separate: bool) -> Self {
        self.separate_files = separate;
        self
    }

    pub fn with_exportable_only(mut self, exportable_only: bool) -> Self {
        self.exportable_only = exportable_only;
        self
    }

    pub fn with_template(mut self, template: String) -> Self {
        self.template = Some(template);
        self
    }
}

impl CodeGenerator<HtmlVisitor> for HtmlGenerator {
    fn generate(&self, visitor: &HtmlVisitor) -> crate::core::Result<CodeGenResult> {
        let mut result = CodeGenResult::new();

        if self.separate_files {
            self.generate_separate_files(visitor, &mut result)?;
        } else {
            self.generate_single_file(visitor, &mut result)?;
        }

        Ok(result)
    }

    fn file_extension(&self) -> &str {
        "html"
    }

    fn output_directory(&self) -> &str {
        "html"
    }
}

impl HtmlGenerator {
    /// Generate separate HTML files for each component
    fn generate_separate_files(
        &self,
        visitor: &HtmlVisitor,
        result: &mut CodeGenResult,
    ) -> crate::core::Result<()> {
        let root_elements = if self.exportable_only {
            visitor.exportable_root_html_elements_by_name()
        } else {
            visitor.root_html_elements_by_name()
        };

        for (component_name, (node_id, html_element)) in root_elements {
            let file_name = format!("{}.html", component_name);
            let file_path = PathBuf::from(file_name);

            let content = self.generate_html_file(&component_name, &html_element)?;
            result.add_file(file_path, content);
        }

        Ok(())
    }

    /// Generate a single HTML file with all components
    fn generate_single_file(
        &self,
        visitor: &HtmlVisitor,
        result: &mut CodeGenResult,
    ) -> crate::core::Result<()> {
        let root_elements = if self.exportable_only {
            visitor.exportable_root_html_elements_by_name()
        } else {
            visitor.root_html_elements_by_name()
        };

        let file_path = PathBuf::from("index.html");
        let content = self.generate_index_file(&root_elements)?;
        result.add_file(file_path, content);

        Ok(())
    }

    /// Generate a single HTML file
    fn generate_html_file(
        &self,
        component_name: &str,
        html_element: &str,
    ) -> crate::core::Result<String> {
        let mut content = String::new();

        // Add DOCTYPE if requested
        if self.include_doctype {
            content.push_str("<!DOCTYPE html>\n");
        }

        // Add HTML structure
        content.push_str("<html>\n");
        content.push_str("<head>\n");
        content.push_str(&format!("  <title>{}</title>\n", component_name));
        content.push_str("  <meta charset=\"utf-8\">\n");
        content.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
        );
        content.push_str("</head>\n");
        content.push_str("<body>\n");

        // Add the component content
        content.push_str(&format!("  <!-- {} -->\n", component_name));
        content.push_str(&html_element);
        content.push_str("\n");

        content.push_str("</body>\n");
        content.push_str("</html>\n");

        Ok(content)
    }

    /// Generate an index file with all components
    fn generate_index_file(
        &self,
        root_elements: &HashMap<String, (String, String)>,
    ) -> crate::core::Result<String> {
        let mut content = String::new();

        // Add DOCTYPE if requested
        if self.include_doctype {
            content.push_str("<!DOCTYPE html>\n");
        }

        // Add HTML structure
        content.push_str("<html>\n");
        content.push_str("<head>\n");
        content.push_str("  <title>Generated Components</title>\n");
        content.push_str("  <meta charset=\"utf-8\">\n");
        content.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
        );
        content.push_str("</head>\n");
        content.push_str("<body>\n");

        // Add navigation
        content.push_str("  <nav>\n");
        content.push_str("    <h2>Components</h2>\n");
        content.push_str("    <ul>\n");
        for component_name in root_elements.keys() {
            content.push_str(&format!(
                "      <li><a href=\"#{}\">{}</a></li>\n",
                component_name, component_name
            ));
        }
        content.push_str("    </ul>\n");
        content.push_str("  </nav>\n\n");

        // Add all components
        for (component_name, (_, html_element)) in root_elements {
            content.push_str(&format!("  <section id=\"{}\">\n", component_name));
            content.push_str(&format!("    <h2>{}</h2>\n", component_name));
            content.push_str(&html_element);
            content.push_str("\n  </section>\n\n");
        }

        content.push_str("</body>\n");
        content.push_str("</html>\n");

        Ok(content)
    }
}
