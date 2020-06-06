use std::{io, str};

use crate::roxc::{
    analyse_program, get_builtin_types, get_type_from_name, Declaration,
    FunctionTranslator, RoxError, RoxResult, Stack, Statement,
};
use cranelift::codegen;
use cranelift::prelude::*;
use cranelift_codegen::isa::CallConv;
use cranelift_module::{default_libcall_names, DataContext, Linkage, Module};
use cranelift_object::{ObjectBackend, ObjectBuilder};
use im::HashMap;
use lalrpop_util::lexer::Token;
use lalrpop_util::ErrorRecovery;
use std::borrow::Borrow;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use target_lexicon::Triple;

lalrpop_mod!(#[allow(clippy::all)] pub rox_parser);

type LalrpopParseError<'input> =
    ErrorRecovery<usize, Token<'input>, &'static str>;

pub struct Compiler {
    function_builder_context: FunctionBuilderContext,
    data_context: DataContext,
    module: Module<ObjectBackend>,
    environment_stack: Stack<HashMap<String, Variable>>,
}

impl Compiler {
    pub fn new(name: &str) -> Self {
        let mut flags_builder = cranelift::codegen::settings::builder();
        flags_builder.enable("is_pic").unwrap();
        flags_builder.enable("enable_verifier").unwrap();
        let flags = settings::Flags::new(flags_builder);
        let isa = codegen::isa::lookup(Triple::host()).unwrap().finish(flags);

        let builder = ObjectBuilder::new(isa, name, default_libcall_names());
        let module = cranelift_module::Module::new(builder);
        let mut environment_stack = Stack::new();
        environment_stack.push(HashMap::new());

        Compiler {
            function_builder_context: FunctionBuilderContext::new(),
            data_context: DataContext::new(),
            module,
            environment_stack,
        }
    }

    pub fn compile(&mut self, source: &str) -> RoxResult<()> {
        match self.parse_source_code(source) {
            Err(errors) => {
                println!("{:?}", errors);
                RoxError::compile_error()
            } // TODO: Properly convert errors
            Ok(declarations) => self.compile_declarations(&declarations),
        }
    }

    pub fn finish(self, output: &Path) -> io::Result<()> {
        let product = self.module.finish();
        let bytes = product.emit().unwrap();
        File::create(output)?
            .write_all(&bytes)
            .map_err(io::Error::into)
    }

    fn parse_source_code<'a>(
        &'a self,
        source: &'a str,
    ) -> Result<Vec<Declaration>, Vec<LalrpopParseError>> {
        let mut errors = Vec::new();
        let declarations = rox_parser::ProgramParser::new()
            .parse(&mut errors, source)
            .unwrap();
        match errors {
            empty_vec if empty_vec.is_empty() => Ok(declarations),
            error_vec => Err(error_vec),
        }
    }

    fn compile_declarations(
        &mut self,
        declarations: &[Declaration],
    ) -> RoxResult<()> {
        analyse_program(declarations);
        declarations.iter().for_each(|declaration| {
            self.translate_declaration(declaration).unwrap();
        });
        Ok(())
    }

    pub fn translate_declaration(
        &mut self,
        declaration: &Declaration,
    ) -> RoxResult<()> {
        match declaration {
            Declaration::Function(func_declaration) => {
                let mut codegen_context = self.module.make_context();
                match func_declaration.borrow() {
                    Statement::FunctionDeclaration(
                        func_name,
                        params,
                        return_type,
                        block,
                    ) => {
                        let mut signature = Signature::new(CallConv::SystemV);
                        params.iter().for_each(|(_, type_str)| {
                            let codegen_type = get_type_from_name(type_str);
                            signature.params.push(AbiParam::new(codegen_type));
                        });

                        codegen_context.func.name = func_name.parse().unwrap();
                        codegen_context.func.signature = signature;

                        let mut builder = FunctionBuilder::new(
                            &mut codegen_context.func,
                            &mut self.function_builder_context,
                        );
                        let mut function_translator = FunctionTranslator::new(
                            &mut builder,
                            &mut self.environment_stack,
                            &mut self.module,
                        );

                        function_translator.translate_function(
                            params,
                            return_type,
                            block,
                        );

                        let func = self
                            .module
                            .declare_function(
                                func_name,
                                Linkage::Export,
                                &codegen_context.func.signature,
                            )
                            .unwrap();
                        self.module
                            .define_function(
                                func,
                                &mut codegen_context,
                                &mut codegen::binemit::NullTrapSink {},
                            )
                            .unwrap();
                        self.module.clear_context(&mut codegen_context);
                    }
                    _ => unreachable!(),
                };
                self.module.clear_context(&mut codegen_context);
                self.module.finalize_definitions();
                Ok(())
            }
        }
    }
}
