use crate::roxc::function_translator::FunctionTranslator;
use crate::roxc::vm::{Chunk, VM};
use crate::roxc::{analyse_program, Result, Statement};
use std::path::PathBuf;

pub struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Self {
        let chunk = Chunk::new();
        Compiler { chunk }
    }

    pub fn compile(&mut self, declarations: Vec<Box<Statement>>) -> Result<()> {
        match self.compile_statements(&declarations) {
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    }

    pub fn finish(self, _path: impl Into<PathBuf> + Sized) -> bool {
        let mut vm = VM::new(self.chunk);
        vm.interpret().unwrap();
        true
    }

    fn compile_statements(
        &mut self,
        declarations: &[Box<Statement>],
    ) -> Result<()> {
        let tagged_statements = analyse_program(declarations.to_vec())?;
        let mut translator =
            FunctionTranslator::new(&mut self.chunk, Vec::new(), 0);
        translator.translate_function(tagged_statements.as_slice())?;
        Ok(())
    }
}
