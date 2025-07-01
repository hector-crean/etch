use std::path::Path;
use swc_ecma_codegen::{Emitter, text_writer::JsWriter};
use swc_ecma_visit::VisitMut;

use crate::error::TsxError;
use crate::file::parse_tsx_file;

/// Pipeline for processing TypeScript files with multiple SWC visitors
pub struct Pipeline {
    visitors: Vec<Box<dyn VisitMut>>,
}

/// Pipeline that can return visitor state after processing
pub struct StatefulPipeline<V: VisitMut> {
    visitor: V,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    /// Create a new, empty pipeline
    pub fn new() -> Self {
        Self { visitors: vec![] }
    }

    /// Add a visitor to the pipeline
    pub fn add_visitor<V: VisitMut + 'static>(&mut self, visitor: V) -> &mut Self {
        self.visitors.push(Box::new(visitor));
        self
    }

    /// Run the pipeline on an input file and write the result to an output file
    pub fn run<I: AsRef<Path>>(&mut self, input_file: I) -> Result<String, TsxError> {
        // Parse the TypeScript source using the helper function
        let (cm, mut module) = parse_tsx_file(input_file)?;

        // Apply each visitor in order
        for visitor in &mut self.visitors {
            visitor.visit_mut_module(&mut module);
        }

        // Generate the output code
        let mut output = Vec::new();
        {
            let writer = JsWriter::new(cm.clone(), "\n", &mut output, None);
            let mut emitter = Emitter {
                cfg: Default::default(),
                cm: cm.clone(),
                comments: None,
                wr: writer,
            };
            emitter.emit_module(&module)?;
        }

        // Convert output to string and write to file
        let tsx = String::from_utf8(output)?;
        Ok(tsx)
    }
}

impl<V: VisitMut> StatefulPipeline<V> {
    /// Create a new stateful pipeline with a single visitor
    pub fn new(visitor: V) -> Self {
        Self { visitor }
    }

    /// Run the pipeline and return both the result and the visitor
    pub fn run<I: AsRef<Path>>(mut self, input_file: I) -> Result<(String, V), TsxError> {
        // Parse the TypeScript source using the helper function
        let (cm, mut module) = parse_tsx_file(input_file)?;

        // Apply the visitor
        self.visitor.visit_mut_module(&mut module);

        // Generate the output code
        let mut output = Vec::new();
        {
            let writer = JsWriter::new(cm.clone(), "\n", &mut output, None);
            let mut emitter = Emitter {
                cfg: Default::default(),
                cm: cm.clone(),
                comments: None,
                wr: writer,
            };
            emitter.emit_module(&module)?;
        }

        // Convert output to string
        let tsx = String::from_utf8(output)?;
        Ok((tsx, self.visitor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_ecma_ast::*;
    use swc_ecma_visit::VisitMut;
    use tempfile::NamedTempFile;
    use std::io::Write;

    /// Simple visitor that replaces "test" with "REPLACED"
    struct TestStringReplacer;

    impl VisitMut for TestStringReplacer {
        fn visit_mut_str(&mut self, node: &mut Str) {
            if node.value.to_string() == "test" {
                node.value = "REPLACED".into();
            }
        }
    }

    #[test]
    fn test_stateful_pipeline_string_replacement() {
        // Create a temporary file with test content
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"
const data = {
    url: "test",
    name: "other"
};
"#;
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Create pipeline with test visitor
        let visitor = TestStringReplacer;
        let pipeline = StatefulPipeline::new(visitor);

        // Run pipeline
        let result = pipeline.run(temp_file.path()).unwrap();
        let (updated_content, _visitor) = result;

        println!("Original content:\n{}", content);
        println!("Updated content:\n{}", updated_content);

        // Verify the replacement worked
        assert!(updated_content.contains("\"REPLACED\""));
        assert!(!updated_content.contains("\"test\""));
    }
}
