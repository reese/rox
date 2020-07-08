use crate::roxc::{
    analyse_program, get_builtin_types, get_cranelift_type, Declaration,
    FunctionDeclaration, FunctionTranslator, Identifier, Result, Stack,
    TaggedDeclaration, TaggedStatement,
};
use cranelift::codegen;
use cranelift::prelude::*;
use cranelift_codegen::isa::CallConv;
use cranelift_module::{Backend, DataContext, Linkage, Module};
use std::borrow::Borrow;
use std::collections::HashMap;

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
        let (_, _, function_stack) = get_builtin_types();

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
        declarations: Vec<Declaration>,
    ) -> Result<Vec<()>> {
        self.compile_declarations(&declarations)
    }

    pub fn finish(self) -> T::Product {
        self.module.finish()
    }

    fn compile_declarations(
        &mut self,
        declarations: &[Declaration],
    ) -> Result<Vec<()>> {
        let tagged_declarations = analyse_program(declarations.to_vec())?;
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
                    TaggedStatement::ExternFunctionDeclaration(
                        function_declaration,
                    ) => {
                        self.function_stack.top_mut().insert(
                            function_declaration.name.clone(),
                            function_declaration.clone(),
                        );
                    }
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
                            let codegen_type = get_cranelift_type(
                                type_str,
                                self.module.target_config().pointer_type(),
                            );
                            if codegen_type != types::INVALID {
                                signature
                                    .params
                                    .push(AbiParam::new(codegen_type));
                            }
                        });

                        let codegen_type = get_cranelift_type(
                            return_type,
                            self.module.target_config().pointer_type(),
                        );
                        if codegen_type != types::INVALID {
                            signature.returns.push(AbiParam::new(codegen_type));
                        }

                        codegen_context.func.name =
                            func_name.clone().parse().unwrap();
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
                                func_name.clone().as_str(),
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
