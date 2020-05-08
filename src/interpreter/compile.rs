use std::{io, str};

use crate::interpreter::{
    Declaration, Expression, FunctionTranslator, InterpretError, Operation,
    RoxResult, Stack, Statement, Unary,
};
use cranelift::codegen;
use cranelift::prelude::*;
use cranelift_module::{default_libcall_names, DataContext, Linkage, Module};
use cranelift_object::{ObjectBackend, ObjectBuilder};
use im::HashMap;
use lalrpop_util::lexer::Token;
use lalrpop_util::ErrorRecovery;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use target_lexicon::Triple;

lalrpop_mod!(#[allow(clippy::all)] pub rox_parser);

type LalrpopParseError<'input> =
    ErrorRecovery<usize, Token<'input>, &'static str>;

pub struct Compiler {
    function_builder_context: FunctionBuilderContext,
    codegen_context: codegen::Context,
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
            codegen_context: module.make_context(),
            data_context: DataContext::new(),
            module,
            environment_stack,
        }
    }

    pub fn compile(&mut self, source: &str) -> RoxResult<()> {
        match self.parse_source_code(source) {
            Err(errors) => {
                println!("{:?}", errors);
                InterpretError::compile_error()
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
            Declaration::Function(func_name, params, block) => {
                params.iter().enumerate().for_each(|_| {
                    // for now, all params will be `Float` types
                    // TODO: Change this once we get type annotations/inference
                    self.codegen_context
                        .func
                        .signature
                        .params
                        .push(AbiParam::new(types::F64));
                });

                let mut builder = FunctionBuilder::new(
                    &mut self.codegen_context.func,
                    &mut self.function_builder_context,
                );

                // Create the block to emit code in, then seal it
                let entry_block = builder.create_block();
                builder.append_block_params_for_function_params(entry_block);
                builder.switch_to_block(entry_block);
                builder.seal_block(entry_block);

                let mut index = 0;
                let mut variables = HashMap::new();
                let mut func_translator = FunctionTranslator::new(
                    builder,
                    &mut variables,
                    &mut self.module,
                    &mut index,
                );
                block.iter().for_each(|statement| {
                    func_translator.translate_statement(statement);
                });

                func_translator.builder.ins().return_(&[]);
                func_translator.finalize();

                let id = self
                    .module
                    .declare_function(
                        &func_name,
                        Linkage::Export,
                        &self.codegen_context.func.signature,
                    )
                    .unwrap();
                self.module
                    .define_function(
                        id,
                        &mut self.codegen_context,
                        &mut codegen::binemit::NullTrapSink {},
                    )
                    .unwrap();
                self.module.clear_context(&mut self.codegen_context);
                self.module.finalize_definitions();
                Ok(())
            }
        }
    }
}
