use std::path::Path;
use swc_ecma_codegen::{Emitter, text_writer::JsWriter};
use swc_ecma_visit::VisitMut;

use crate::error::TsxError;
use crate::file::parse_tsx_file;

/// Pipeline for processing TypeScript files with multiple SWC visitors
pub struct Pipeline {
    visitors: Vec<Box<dyn VisitMut>>,
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
