use std::str;

use crate::roxc::tagged_syntax::{TaggedDeclaration, TaggedStatement};
use crate::roxc::{
    analyse_program, Declaration, FunctionDeclaration, FunctionTranslator,
    Identifier, Result, RoxError, Stack,
};
use cranelift::codegen;
use cranelift::prelude::*;
use cranelift_codegen::isa::CallConv;
use cranelift_module::{Backend, DataContext, Linkage, Module};
use im::HashMap;
use std::borrow::Borrow;
use std::fs::read_to_string;
use std::path::PathBuf;

lalrpop_mod!(#[allow(clippy::all)] pub rox_parser);

pub struct Compiler<T: Backend> {
    function_builder_context: FunctionBuilderContext,
    data_context: DataContext,
    pub(crate) module: Module<T>,
    environment_stack: Stack<HashMap<Identifier, Variable>>,
    function_stack: Stack<HashMap<Identifier, FunctionDeclaration>>,
}

impl<T: Backend> Compiler<T> {
    pub fn new(module: Module<T>) -> Self {
        let mut environment_stack = Stack::new();
        environment_stack.push(HashMap::new());
        let mut function_stack = Stack::new();
        function_stack.push(HashMap::new());
        // TODO: Move this into where we add libc types earlier
        function_stack.top_mut().insert(
            Identifier::new_non_generic("puts".to_string()),
            FunctionDeclaration {
                name: Identifier::new_non_generic("puts".to_string()),
                generics: Vec::new(),
                params: vec![("arg".into(), "String".into())],
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

    pub fn compile(
        &mut self,
        file: impl Into<PathBuf> + std::clone::Clone,
    ) -> Result<Vec<()>> {
        let source = read_to_string(file.into()).unwrap();
        let declarations_result = self.parse_source_code(source.as_ref());
        match declarations_result {
            Ok(declarations) => self.compile_declarations(&declarations),
            Err(rox_error) => Err(rox_error),
        }
    }

    pub fn finish(self) -> T::Product {
        self.module.finish()
    }

    fn parse_source_code<'a>(
        &'a self,
        source: &'a str,
    ) -> Result<Vec<Declaration>> {
        let mut errors = Vec::new();
        let declarations = rox_parser::ProgramParser::new()
            .parse(&mut errors, source)
            .map_err(|e| {
                RoxError::from_parse_error(
                    &e,
                    PathBuf::from("./scratch/test.rox"),
                )
            })?;
        match errors {
            empty_vec if empty_vec.is_empty() => Ok(declarations),
            _ => todo!("We haven't implemented nicer errors for ErrorRecovery types yet.")
        }
    }

    fn compile_declarations(
        &mut self,
        declarations: &[Declaration],
    ) -> Result<Vec<()>> {
        let tagged_declarations = analyse_program(declarations)?;
        tagged_declarations
            .iter()
            .map(|declaration| Ok(self.translate_declaration(declaration)?))
            .collect()
    }

    fn translate_declaration(
        &mut self,
        declaration: &TaggedDeclaration,
    ) -> Result<()> {
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
                            generics,
                            params,
                            return_type,
                        } = func_declaration;
                        let mut signature = Signature::new(CallConv::SystemV);
                        params.iter().for_each(|(_, type_str)| {
                            let codegen_type = type_str.get_type(
                                self.module.target_config().pointer_type(),
                            );
                            signature.params.push(AbiParam::new(codegen_type));
                        });

                        if let Some(return_) = return_type {
                            signature.returns.push(AbiParam::new(
                                return_.get_type(
                                    self.module.target_config().pointer_type(),
                                ),
                            ));
                        }

                        codegen_context.func.name =
                            String::from(func_name.clone()).parse().unwrap();
                        codegen_context.func.signature = signature;

                        let mut builder = FunctionBuilder::new(
                            &mut codegen_context.func,
                            &mut self.function_builder_context,
                        );

                        let function_declaration = FunctionDeclaration {
                            name: func_name.clone(),
                            generics: generics.clone(),
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
                                String::from(func_name.clone()).as_str(),
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
