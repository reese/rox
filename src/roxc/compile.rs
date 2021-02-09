use crate::roxc::compiler_state::CompilerState;
use crate::roxc::{
    analyse_program, FunctionDeclaration, FunctionTranslator, Identifier,
    Result, RoxError, Stack, Statement, TaggedStatement, Type,
};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{BasicValue, FunctionValue, PointerValue};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Compiler<'module, 'ctx, 'm> {
    context: &'ctx Context,
    pub(crate) module: &'m Module<'ctx>,
    function_pass_manager: &'module PassManager<FunctionValue<'ctx>>,
    environment_stack:
        &'module mut Stack<HashMap<Identifier, PointerValue<'ctx>>>,
    function_stack:
        &'module mut Stack<HashMap<Identifier, FunctionDeclaration>>,
}

impl<'a, 'ctx, 'm> Compiler<'a, 'ctx, 'm> {
    pub fn new(
        context: &'ctx Context,
        module: &'m Module<'ctx>,
        function_pass_manager: &'a PassManager<FunctionValue<'ctx>>,
        environment_stack: &'a mut Stack<
            HashMap<Identifier, PointerValue<'ctx>>,
        >,
        function_stack: &'a mut Stack<HashMap<Identifier, FunctionDeclaration>>,
    ) -> Self {
        // TODO: Which of these do we actually want?
        function_pass_manager.add_instruction_combining_pass();
        function_pass_manager.add_reassociate_pass();
        function_pass_manager.add_gvn_pass();
        function_pass_manager.add_cfg_simplification_pass();
        function_pass_manager.add_basic_alias_analysis_pass();
        function_pass_manager.add_promote_memory_to_register_pass();
        function_pass_manager.add_instruction_combining_pass();
        function_pass_manager.add_reassociate_pass();

        function_pass_manager.initialize();

        Compiler {
            context,
            module,
            function_pass_manager,
            environment_stack,
            function_stack,
        }
    }

    pub fn compile(&mut self, declarations: Vec<Box<Statement>>) -> Result<()> {
        match self.compile_statements(&declarations) {
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    }

    pub fn finish(&self, path: impl Into<PathBuf> + Sized) -> bool {
        self.module.write_bitcode_to_path(&path.into())
    }

    fn compile_statements(
        &mut self,
        declarations: &[Box<Statement>],
    ) -> Result<Vec<()>> {
        let tagged_statements = analyse_program(declarations.to_vec())?;
        tagged_statements
            .iter()
            .map(|declaration| Ok(self.translate_declaration(declaration)?))
            .collect()
    }

    fn translate_declaration(
        &mut self,
        statement: &TaggedStatement,
    ) -> Result<()> {
        match statement {
            TaggedStatement::ExternFunctionDeclaration(
                function_declaration,
            ) => {
                self.compile_prototype(
                    function_declaration.name.clone(),
                    &function_declaration.params,
                    &function_declaration.return_type,
                );
                Ok(())
            }
            TaggedStatement::FunctionDeclaration(func_declaration, block) => {
                let FunctionDeclaration {
                    name: func_name,
                    params,
                    return_type,
                } = func_declaration;
                let mut fn_value = self.compile_prototype(
                    func_name.clone(),
                    params,
                    return_type,
                );
                let entry = self.context.append_basic_block(fn_value, "entry");
                let builder = self.context.create_builder();
                builder.position_at_end(entry);
                self.environment_stack.top_mut().reserve(params.len());

                fn_value.get_param_iter().enumerate().for_each(
                    |(index, arg)| {
                        let (arg_name, _ty) = params[index].clone();
                        let allocation = create_entry_block_allocation(
                            &builder,
                            arg_name.as_ref(),
                            &mut fn_value,
                            arg.get_type(),
                        );
                        builder.build_store(allocation, arg);
                        self.environment_stack
                            .top_mut()
                            .insert(params[index].0.clone(), allocation);
                    },
                );

                self.function_stack
                    .top_mut()
                    .insert(func_name.clone(), func_declaration.clone());

                let current_state = CompilerState::new(
                    builder,
                    &self.context,
                    fn_value,
                    &self.module,
                );

                let mut function_translator = FunctionTranslator::new(
                    &current_state,
                    self.environment_stack.top_mut(),
                    self.function_stack.top_mut(),
                );

                function_translator.translate_function(block);

                if fn_value.verify(true) {
                    self.function_pass_manager.run_on(&fn_value);
                    Ok(())
                } else {
                    fn_value.print_to_stderr();
                    Err(RoxError::with_file_placeholder(
                        "Invalid generated function",
                    ))
                }
            }
            // This is a no-op.
            // Struct _declarations_ are entirely for the type system,
            // and we'll later use the field order of the type
            // to instantiate the struct.
            TaggedStatement::StructDeclaration => Ok(()),
            TaggedStatement::Expression(_)
            | TaggedStatement::IfElse(_, _, _)
            | TaggedStatement::Return(_) => {
                Err(RoxError::with_file_placeholder(
                    "Cannot use expression at the top level",
                ))
            }
        }
    }

    /// Compile the function signature
    fn compile_prototype(
        &self,
        func_name: String,
        params: &[(Identifier, Type)],
        return_type: &Type,
    ) -> FunctionValue<'ctx> {
        let param_types = params
            .iter()
            .map(|(_ident, ty)| {
                CompilerState::get_type(
                    self.context,
                    ty,
                    self.environment_stack.top(),
                    Some(0),
                )
                .expect(&*format!(
                    "Cannot handle void parameter type or undefined type {:?}",
                    ty
                ))
            })
            .collect::<Vec<_>>();
        let fn_type = match CompilerState::get_type(
            self.context,
            return_type,
            self.environment_stack.top(),
            None,
        ) {
            Some(t) => t.fn_type(param_types.as_slice(), false),
            None => self
                .context
                .void_type()
                .fn_type(param_types.as_slice(), false),
        };
        let fn_value =
            self.module.add_function(func_name.as_str(), fn_type, None);
        fn_value
            .get_param_iter()
            .enumerate()
            .for_each(|(index, arg)| {
                arg.set_name(params.get(index).unwrap().0.as_str())
            });
        fn_value
    }
}

/// Allocate space on stack frame for function arguments
pub fn create_entry_block_allocation<'f>(
    builder: &Builder<'f>,
    name: &str,
    function: &mut FunctionValue,
    ty: BasicTypeEnum<'f>,
) -> PointerValue<'f> {
    let entry = function.get_first_basic_block().unwrap();
    if let Some(first_instruction) = entry.get_first_instruction() {
        builder.position_before(&first_instruction);
    } else {
        builder.position_at_end(entry);
    }
    builder.build_alloca(ty, name)
}
