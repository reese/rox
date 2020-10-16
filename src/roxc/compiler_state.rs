use crate::roxc::{Operation, Type};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{
    BasicValue, BasicValueEnum, FloatValue, FunctionValue, InstructionValue,
    PointerValue,
};
use inkwell::{AddressSpace, FloatPredicate};

pub struct CompilerState<'f, 'c> {
    builder: Builder<'c>,
    context: &'c Context,
    function: FunctionValue<'c>,
    module: &'f Module<'c>,
}

impl<'f, 'c> CompilerState<'f, 'c> {
    pub fn new(
        builder: Builder<'c>,
        context: &'c Context,
        function: FunctionValue<'c>,
        module: &'f Module<'c>,
    ) -> Self {
        Self {
            builder,
            context,
            function,
            module,
        }
    }

    pub fn get_context(&self) -> &'c Context {
        self.context
    }

    pub fn get_function(&self, name: &str) -> Option<FunctionValue<'c>> {
        self.module.get_function(name)
    }

    pub fn get_type(
        context: &'c Context,
        ty: &Type,
    ) -> Option<BasicTypeEnum<'c>> {
        match ty {
            Type::Apply(constructor, _) => {
                use super::semant::TypeConstructor::*;
                match constructor {
                    Bool => Some(context.bool_type().into()),
                    Number => Some(context.f64_type().into()),
                    String => Some(
                        context
                            .i64_type()
                            .array_type(10)
                            .ptr_type(AddressSpace::Generic)
                            .into(),
                    ),
                    Void => None,
                    x => todo!("Need to handle type constructor for: {:?}", x),
                }
            }
            Type::Variable(_) => unimplemented!(),
            Type::PolymorphicType(_formal_arguments, _types) => {
                unimplemented!()
            }
        }
    }

    pub fn build_return(
        &self,
        return_: Option<&dyn BasicValue<'c>>,
    ) -> InstructionValue<'c> {
        self.builder.build_return(return_)
    }

    pub fn append_basic_block(&self, block_tag: &str) -> BasicBlock<'c> {
        self.context.append_basic_block(self.function, block_tag)
    }

    pub fn build_conditional(
        &self,
        conditional_expression_value: FloatValue,
        branch_name: &str,
        if_block: BasicBlock,
        else_block: BasicBlock,
    ) {
        let zero_const = self.context.f64_type().const_zero();
        let conditional = self.builder.build_float_compare(
            FloatPredicate::ONE,
            conditional_expression_value,
            zero_const,
            branch_name,
        );
        self.builder.build_conditional_branch(
            conditional,
            if_block,
            else_block,
        );
    }

    pub fn build_fallback_branch(&self, merge_block: BasicBlock) {
        self.builder.build_unconditional_branch(merge_block);
    }

    pub fn bool_literal(&self, boolean: bool) -> BasicValueEnum<'c> {
        self.context
            .bool_type()
            .const_int(boolean as u64, false)
            .into()
    }

    pub fn number_literal(&self, num: f64) -> BasicValueEnum<'c> {
        self.context.f64_type().const_float(num).into()
    }

    pub fn string_literal(&self, string: &String) -> BasicValueEnum<'c> {
        self.context.const_string(string.as_bytes(), false).into()
    }

    pub fn load_variable(
        &self,
        pointer: PointerValue<'c>,
        name: &str,
    ) -> BasicValueEnum<'c> {
        self.builder.build_load(pointer, name)
    }

    pub fn store_variable(
        &self,
        name: &String,
        value: BasicValueEnum<'c>,
    ) -> PointerValue<'c> {
        let allocation =
            self.create_entry_block_allocation(name, value.get_type());
        self.builder.build_store(allocation, value);
        allocation
    }

    pub fn build_operation(
        &self,
        lval: FloatValue<'c>,
        rval: FloatValue<'c>,
        operation: &Operation,
    ) -> BasicValueEnum<'c> {
        use Operation::*;
        match operation {
            Add => self.builder.build_float_add(lval, rval, "tmpadd").into(),
            Subtract => {
                self.builder.build_float_sub(lval, rval, "tmpsub").into()
            }
            Multiply => {
                self.builder.build_float_mul(lval, rval, "tmpmul").into()
            }
            Divide => self.builder.build_float_div(lval, rval, "tmpdiv").into(),
            Equals => {
                let comparison = self.builder.build_float_compare(
                    FloatPredicate::OEQ,
                    lval,
                    rval,
                    "tmpcmp",
                );
                self.builder
                    .build_unsigned_int_to_float(
                        comparison,
                        self.context.f64_type(),
                        "tmpbool",
                    )
                    .into()
            }
            NotEquals => {
                let comparison = self.builder.build_float_compare(
                    FloatPredicate::ONE,
                    lval,
                    rval,
                    "tmpcmp",
                );
                self.builder
                    .build_unsigned_int_to_float(
                        comparison,
                        self.context.f64_type(),
                        "tmpbool",
                    )
                    .into()
            }
            GreaterThan => {
                let comparison = self.builder.build_float_compare(
                    FloatPredicate::OGT,
                    lval,
                    rval,
                    "tmpcmp",
                );
                self.builder
                    .build_unsigned_int_to_float(
                        comparison,
                        self.context.f64_type(),
                        "tmpbool",
                    )
                    .into()
            }
            LessThan => {
                let comparison = self.builder.build_float_compare(
                    FloatPredicate::OLT,
                    lval,
                    rval,
                    "tmpcmp",
                );
                self.builder
                    .build_unsigned_int_to_float(
                        comparison,
                        self.context.f64_type(),
                        "tmpbool",
                    )
                    .into()
            }
        }
    }

    pub fn function_call(
        &self,
        function: FunctionValue<'c>,
        args: &[BasicValueEnum<'c>],
    ) -> Option<BasicValueEnum<'c>> {
        self.builder
            .build_call(function, args, "tmp")
            .try_as_basic_value()
            .left()
    }

    pub fn position_at_end(&self, block: BasicBlock) {
        self.builder.position_at_end(block)
    }

    /// Allocate space on stack frame for function arguments
    fn create_entry_block_allocation(
        &self,
        name: &str,
        ty: BasicTypeEnum<'c>,
    ) -> PointerValue<'c> {
        let entry = self.function.get_first_basic_block().unwrap();
        if let Some(first_instruction) = entry.get_first_instruction() {
            self.builder.position_before(&first_instruction);
        } else {
            self.builder.position_at_end(entry);
        }
        self.builder.build_alloca(ty, name)
    }
}
