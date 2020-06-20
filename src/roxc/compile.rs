use std::str;

use crate::roxc::tagged_syntax::{TaggedDeclaration, TaggedStatement};
use crate::roxc::{
    analyse_program, get_type_from_name, Declaration, FunctionDeclaration,
    FunctionTranslator, RoxError, RoxResult, Stack,
};
use cranelift::codegen;
use cranelift::prelude::*;
use cranelift_codegen::isa::CallConv;
use cranelift_module::{Backend, DataContext, Linkage, Module};
use im::HashMap;
use lalrpop_util::lexer::Token;
use lalrpop_util::ErrorRecovery;
use std::borrow::Borrow;

lalrpop_mod!(#[allow(clippy::all)] pub rox_parser);

type LalrpopParseError<'input> =
    ErrorRecovery<usize, Token<'input>, &'static str>;

pub struct Compiler<T: Backend> {
    function_builder_context: FunctionBuilderContext,
    data_context: DataContext,
    module: Module<T>,
    environment_stack: Stack<HashMap<String, Variable>>,
    function_stack: Stack<HashMap<String, FunctionDeclaration>>,
}

impl<T: Backend> Compiler<T> {
    pub fn new(module: Module<T>) -> Self {
        let mut environment_stack = Stack::new();
        environment_stack.push(HashMap::new());
        let mut function_stack = Stack::new();
        function_stack.push(HashMap::new());
        function_stack.top_mut().insert(
            "puts".to_string(),
            FunctionDeclaration {
                name: "puts".to_string(),
                params: vec![("arg".to_string(), "str".to_string())],
                return_type: None,
            },
        );

        Compiler {
            function_builder_context: FunctionBuilderContext::new(),
            data_context: DataContext::new(),
            module,
            environment_stack,
            function_stack,
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

    pub fn finish(self) -> T::Product {
        self.module.finish()
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
        let tagged_declarations = analyse_program(declarations);
        tagged_declarations.iter().for_each(|declaration| {
            self.translate_declaration(declaration).unwrap();
        });
        Ok(())
    }

    pub(crate) fn translate_declaration(
        &mut self,
        declaration: &TaggedDeclaration,
    ) -> RoxResult<()> {
        match declaration {
            TaggedDeclaration::Function(func_declaration) => {
                let mut codegen_context = self.module.make_context();
                match func_declaration.borrow() {
                    TaggedStatement::FunctionDeclaration(
                        func_declaration,
                        block,
                    ) => {
                        let FunctionDeclaration {
                            name: func_name,
                            params,
                            return_type,
                        } = func_declaration;
                        let mut signature = Signature::new(CallConv::SystemV);
                        params.iter().for_each(|(_, type_str)| {
                            let codegen_type =
                                get_type_from_name(type_str, &self.module);
                            signature.params.push(AbiParam::new(codegen_type));
                        });

                        if let Some(return_) = return_type {
                            signature.returns.push(AbiParam::new(
                                get_type_from_name(
                                    return_.as_ref(),
                                    &self.module,
                                ),
                            ));
                        }

                        codegen_context.func.name = func_name.parse().unwrap();
                        codegen_context.func.signature = signature;

                        let mut builder = FunctionBuilder::new(
                            &mut codegen_context.func,
                            &mut self.function_builder_context,
                        );

                        let function_declaration = FunctionDeclaration {
                            name: func_name.clone(),
                            params: params.clone(),
                            return_type: return_type.clone(),
                        };
                        self.function_stack
                            .top_mut()
                            .insert(func_name.clone(), function_declaration);

                        let mut function_translator = FunctionTranslator::new(
                            &mut builder,
                            &mut self.data_context,
                            &mut self.environment_stack,
                            &mut self.function_stack,
                            &mut self.module,
                        );

                        function_translator.translate_function(&params, block);

                        let func = self
                            .module
                            .declare_function(
                                &func_name,
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
