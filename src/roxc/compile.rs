use crate::roxc::function_translator::FunctionTranslator;
use crate::roxc::vm::function::Function;
use crate::roxc::vm::VM;
use crate::roxc::{analyse_program, Result, Statement};
use std::{borrow::Cow, path::PathBuf, rc::Rc};

use super::{
    local::Local,
    vm::{native_function::NativeFuncHolder, Value},
};

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
        let mut locals = Vec::new();
        self.import_stdlib(&mut locals);
        let translator =
            FunctionTranslator::new(self.function.get_mut_chunk(), locals, 0);
        translator.translate_statements(tagged_statements.as_slice())?;
        Ok(())
    }

    fn import_stdlib(&mut self, locals: &mut Vec<Local>) {
        let chunk = self.function.get_mut_chunk();

        let func = Value::NativeFunction(NativeFuncHolder {
            inner: Rc::new(print as fn(Cow<'static, str>) -> ()),
        });
        let another_func = Value::NativeFunction(NativeFuncHolder {
            inner: Rc::new(print_number as fn(f64) -> ()),
        });
        chunk.add_constant(func);
        chunk.add_constant(another_func);
        locals.push(Local::new(String::from("print"), 0));
        locals.push(Local::new(String::from("print_number"), 0))
    }
}

fn print(str: Cow<'static, str>) {
    println!("{}", str);
}

fn print_number(num: f64) {
    println!("{}", num);
}
