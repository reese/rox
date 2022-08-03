use crate::roxc::{Identifier, Operation, Type};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::{
    BasicValue, BasicValueEnum, FloatValue, FunctionValue, InstructionValue,
    PointerValue,
};
use inkwell::{basic_block::BasicBlock, values::IntValue};
use inkwell::{builder::Builder, IntPredicate};
use inkwell::{AddressSpace, FloatPredicate};
use std::{collections::HashMap, convert::TryInto};

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
        environment: &'f HashMap<Identifier, PointerValue<'c>>,
        maybe_len: Option<usize>,
    ) -> Option<BasicTypeEnum<'c>> {
        match ty {
            Type::Apply(constructor, _type_arguments) => {
                use super::semant::TypeConstructor::*;
                match constructor {
                    Bool => Some(context.bool_type().into()),
                    Float => Some(context.f64_type().into()),
                    Int => Some(context.i32_type().into()),
                    String => Some(
                        // TODO: Should strings be built on top of the actual array implementation (below)?
                        context
                            .i8_type()
                            .array_type((maybe_len.unwrap() + 1) as u32)
                            .ptr_type(AddressSpace::Generic)
                            .into(),
                    ),
                    Void => None,
                    Array(inner_type) => {
                        let inner_type = CompilerState::get_type(
                            context,
                            inner_type,
                            environment,
                            maybe_len,
                        )
                        .unwrap();
                        let array_type = inner_type
                            .array_type(0)
                            .ptr_type(AddressSpace::Generic);
                        // TODO: This just store the array length as an i64, but should probably check if that's a reasonable assumption
                        // or if LLVM supports unsigned int types
                        //
                        // N.B. Struct types are represented as a struct where the first
                        // field is the inner type of the array, and the second is the length
                        // of the array.
                        // We'll need to do manual bound checks on these to prevent out-of-bounds access.
                        let struct_type = context.struct_type(
                            &[
                                array_type.as_basic_type_enum(),
                                context.i32_type().as_basic_type_enum(),
                            ],
                            false,
                        ); // Not packing these structs
                        Some(struct_type.as_basic_type_enum())
                    }
                    Arrow | Record(_) | FunctionType(_, _) | Unique(_) => {
                        todo!()
                    }
                }
            }
            Type::Variable(variable_name) => environment
                .get(variable_name)
                .map(|var| var.get_type().as_basic_type_enum()),
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

    pub unsafe fn build_array_access(
        &self,
        array_struct: PointerValue<'c>,
        index: IntValue<'c>,
    ) -> PointerValue<'c> {
        // Load array length and do a bounds check
        let _array_len =
            self.builder.build_struct_gep(array_struct, 1, "").unwrap();
        // TODO: bounds check here

        // index from struct
        let one = self.context.i64_type().const_int(1, false);
        let array_ref =
            dbg!(self.builder.build_struct_gep(array_struct, 0, "").unwrap());
        let loaded_array = self.build_load(array_ref).into_pointer_value();
        self.builder
            .build_in_bounds_gep(loaded_array, &[one, index], "")
    }

    pub fn build_load(&self, pointer: PointerValue<'c>) -> BasicValueEnum<'c> {
        self.builder.build_load(pointer, "")
    }

    pub fn build_store(
        &self,
        pointer: PointerValue<'c>,
        value: BasicValueEnum<'c>,
    ) {
        self.builder.build_store(pointer, value);
    }

    /// Allocates an array, loads items into that array, and returns
    /// a pointer to the array
    pub fn build_array_allocation_with_values(
        &self,
        items: &[BasicValueEnum<'c>],
        type_: BasicTypeEnum<'c>,
    ) -> PointerValue<'c> {
        let len = self.context.i32_type().const_int(items.len() as u64, false);
        let array_pointer_type = type_
            .into_struct_type()
            .get_field_type_at_index(0)
            .expect("Array type did not have inner array type at index 0")
            .into_pointer_type()
            .get_element_type()
            .into_array_type();
        let allocation =
            self.builder.build_array_alloca(array_pointer_type, len, "");
        let one = self.context.i64_type().const_int(1, false);

        items.iter().enumerate().for_each(|(index, item)| {
            let index = self.context.i16_type().const_int(index as u64, false);
            let pointer = unsafe {
                self.builder
                    .build_in_bounds_gep(allocation, &[one, index], "")
            };
            self.build_store(pointer, *item);
        });

        self.build_array_struct(allocation, len)
    }

    pub fn bool_literal(&self, boolean: bool) -> BasicValueEnum<'c> {
        self.context
            .bool_type()
            .const_int(boolean as u64, false)
            .into()
    }

    pub fn int_literal(&self, num: i32) -> BasicValueEnum<'c> {
        self.context
            .i32_type()
            .const_int(num.try_into().unwrap(), false)
            .into()
    }

    pub fn float_literal(&self, num: f64) -> BasicValueEnum<'c> {
        self.context.f64_type().const_float(num).into()
    }

    pub fn string_literal(&self, string: &str) -> BasicValueEnum<'c> {
        self.context.const_string(string.as_bytes(), false).into()
    }

    pub fn store_variable(
        &self,
        name: &str,
        value: BasicValueEnum<'c>,
    ) -> PointerValue<'c> {
        let allocation =
            self.create_entry_block_allocation(name, value.get_type());
        self.build_store(allocation, value);
        allocation
    }

    pub fn build_array_struct(
        &self,
        array_pointer: PointerValue<'c>,
        length_value: IntValue<'c>,
    ) -> PointerValue<'c> {
        let struct_allocation = self.builder.build_alloca(
            self.context.struct_type(
                &[
                    array_pointer.get_type().as_basic_type_enum(),
                    length_value.get_type().as_basic_type_enum(),
                ],
                false,
            ),
            "",
        );
        [
            array_pointer.as_basic_value_enum(),
            length_value.as_basic_value_enum(),
        ]
        .iter()
        .enumerate()
        .for_each(|(index, val)| {
            let struct_position = self
                .builder
                .build_struct_gep(struct_allocation, index as u32, "")
                .unwrap();
            self.build_store(struct_position, val.as_basic_value_enum());
        });
        struct_allocation
    }

    pub fn build_int_operation(
        &self,
        lval: IntValue<'c>,
        rval: IntValue<'c>,
        operation: &Operation,
    ) -> BasicValueEnum<'c> {
        use Operation::*;
        match operation {
            Add => self.builder.build_int_add(lval, rval, "tmpadd").into(),
            Subtract => self.builder.build_int_sub(lval, rval, "tmpsub").into(),
            Multiply => self.builder.build_int_mul(lval, rval, "tmpmul").into(),
            Divide => self
                .builder
                .build_int_signed_div(lval, rval, "tmpdiv")
                .into(),
            Equals => {
                let comparison = self.builder.build_int_compare(
                    IntPredicate::EQ,
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
                let comparison = self.builder.build_int_compare(
                    IntPredicate::NE,
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
                let comparison = self.builder.build_int_compare(
                    IntPredicate::SGT,
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
                let comparison = self.builder.build_int_compare(
                    IntPredicate::SLT,
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

    pub fn build_float_operation(
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
            .build_call(
                function,
                &args.iter().map(|a| (*a).into()).collect::<Vec<_>>(),
                "tmp",
            )
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
        self.builder.build_alloca(ty, name)
    }
}
