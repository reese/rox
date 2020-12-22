use crate::roxc::function_translator::FunctionTranslator;
use crate::roxc::vm::function::Function;
use crate::roxc::vm::VM;
use crate::roxc::{analyse_program, Result, Statement};
use std::path::PathBuf;

pub struct Compiler {
    function: Function,
}

impl Compiler {
    pub fn new() -> Self {
        let function = Function::new_main();
        Compiler { function }
    }

    pub(crate) fn compile(
        mut self,
        declarations: Vec<Box<Statement>>,
    ) -> Result<Function> {
        match self.compile_statements(&declarations) {
            Err(e) => Err(e),
            Ok(_) => Ok(self.function),
        }
    }

    pub(crate) fn finish(
        function: Function,
        _path: impl Into<PathBuf> + Sized,
    ) -> bool {
        let mut vm = VM::new();
        vm.interpret(function).unwrap();
        true
    }

    fn compile_statements(
        &mut self,
        declarations: &[Box<Statement>],
    ) -> Result<()> {
        let tagged_statements = analyse_program(declarations.to_vec())?;
        let mut translator = FunctionTranslator::new(
            self.function.get_mut_chunk(),
            Vec::new(),
            0,
        );
        translator.translate_function(tagged_statements.as_slice())?;
        Ok(())
    }
}
