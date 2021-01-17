use crate::roxc::local::Local;
use crate::roxc::vm::function::Function;
use crate::roxc::vm::object::Object;
use crate::roxc::vm::{Chunk, OpCode, Value};
use crate::roxc::{parser, Result, RoxError};
use crate::roxc::{TaggedExpression, TaggedStatement};
use std::rc::Rc;

pub(crate) struct FunctionTranslator<'c> {
    enclosing_function: Option<&'c FunctionTranslator<'c>>,
    chunk: &'c mut Chunk,
    locals: Vec<Local>,
    scope_depth: i32,
}

impl<'c> FunctionTranslator<'c> {
    pub(crate) fn new(
        chunk: &'c mut Chunk,
        locals: Vec<Local>,
        scope_depth: i32,
    ) -> Self {
        FunctionTranslator {
            enclosing_function: None,
            chunk,
            locals,
            scope_depth,
        }
    }

    pub(crate) fn new_local_function(
        chunk: &'c mut Chunk,
        locals: Vec<Local>,
        scope_depth: i32,
        enclosing_function: &'c FunctionTranslator<'c>,
    ) -> Self {
        FunctionTranslator {
            chunk,
            locals,
            scope_depth,
            enclosing_function: Some(enclosing_function),
        }
    }

    pub(crate) fn translate_statements(
        mut self,
        block: &[TaggedStatement],
    ) -> Result<&'c mut Chunk> {
        // Claim an initial stack slot for the VM
        self.locals.push(Local::new(String::new(), 0));
        block
            .iter()
            .map(|statement| self.translate_statement(statement))
            .collect::<Result<Vec<_>>>()?;
        Ok(self.chunk)
    }

    fn translate_statement(
        &mut self,
        statement: &TaggedStatement,
    ) -> Result<()> {
        use TaggedStatement::*;
        match &statement {
            StructDeclaration => todo!(),
            Block(statements) => {
                self.start_scope();
                statements
                    .iter()
                    .map(|s| self.translate_statement(s))
                    .collect::<Result<Vec<_>>>()?;
                self.end_scope();
                Ok(())
            }
            // Notes about variables:
            // As of right now, the concept of "global" variables
            // (i.e. late-bound static variables) doesn't really exist.
            // All of Rox's variables are "local," as if the entire program
            // is run inside of one giant function.
            // Because all variables are bound at compile time, we don't actually
            // _need_ to track them by name
            Variable(name, expression, _type) => {
                self.translate_expression(expression);
                let ident_is_already_declared = |local: &Local| {
                    local.depth >= self.scope_depth
                        && local.name == name.clone()
                };
                if self.locals.iter().rev().any(ident_is_already_declared) {
                    return Err(RoxError::with_file_placeholder(
                        "Identifier already declared in this scope.",
                    ));
                }
                self.locals
                    .push(Local::new(name.to_string(), self.scope_depth));
                Ok(())
            }
            Assignment(name, right_expr, _type) => {
                self.translate_expression(right_expr);
                self.chunk
                    .write(OpCode::AssignVariable(self.resolve_local(name)));
                Ok(())
            }
            Expression(expression) => {
                self.translate_expression(expression);
                // Pop residual value off the stack
                self.chunk.write(OpCode::Pop);
                Ok(())
            }
            FunctionDeclaration(declaration, block) => {
                self.start_scope();
                declaration.params.iter().for_each(|(param, _)| {
                    self.locals
                        .push(Local::new(param.to_string(), self.scope_depth))
                });
                let mut new_chunk = Chunk::new();
                let translator = FunctionTranslator::new_local_function(
                    &mut new_chunk,
                    self.locals.to_vec(),
                    self.scope_depth + 1,
                    &self,
                );
                translator.translate_statements(block)?;
                let arity = declaration.params.len() as u8;
                let new_function =
                    Function::new(arity, new_chunk, declaration.name.clone());
                self.end_scope();
                self.chunk.add_constant(Value::Obj(Rc::new(Object::Function(
                    new_function,
                ))));
                self.locals.push(Local::new(
                    declaration.name.clone(),
                    self.scope_depth,
                ));
                Ok(())
            }
            // TODO: Do we need external functions like this?
            // if it's in a VM? I think we can provide all of that
            // directly from Rust.
            //
            // The `extern` tag merely declares the function to the type checker
            // The linker will then try to dynamically link the function call
            // if one exists. For the most part, we use this as a way to use
            // `libc` functions, but this could potentially be used to link a
            // Rust runtime library, but that's still undetermined.
            ExternFunctionDeclaration(_decl) => {
                todo!()
                // self.functions.insert(decl.name.clone(), decl.clone());
            }
            Return(maybe_expression) => {
                if self.scope_depth == 0 {
                    return Err(RoxError::with_file_placeholder(
                        "Cannot return from top-level.",
                    ));
                }
                if let Some(expr) = maybe_expression.as_ref() {
                    self.translate_expression(expr)
                } else {
                    self.chunk.add_constant(Value::Unit)
                }
                self.chunk.write(OpCode::Return);
                Ok(())
            }
            IfElse(conditional, if_statements, else_statements_maybe) => {
                self.translate_expression(conditional);
                let then_jump = self.emit_jump(OpCode::JumpIfFalse);
                if_statements
                    .iter()
                    .map(|statement| self.translate_statement(statement))
                    .collect::<Result<Vec<_>>>()?;
                let else_jump = self.emit_jump(OpCode::Jump);
                self.patch_jump(then_jump);
                if let Some(else_statements) = else_statements_maybe {
                    else_statements
                        .iter()
                        .map(|statement| self.translate_statement(statement))
                        .collect::<Result<Vec<_>>>()?;
                }
                self.patch_jump(else_jump);
                Ok(())
            }
            While(conditional, body) => {
                let loop_start = self.chunk.opcodes.len();
                self.translate_expression(conditional.as_ref());

                let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
                body.iter()
                    .map(|statement| self.translate_statement(statement))
                    .collect::<Result<Vec<_>>>()?;
                self.emit_loop(loop_start);
                self.patch_jump(exit_jump);
                self.chunk.write(OpCode::Pop);
                Ok(())
            }
        }
    }

    pub fn translate_expression(&mut self, expression: &TaggedExpression) {
        use TaggedExpression::*;
        match expression {
            Access(_, _, _) => todo!(),
            Or(left_expr, right_expr) => {
                self.translate_expression(left_expr);
                let else_jump = self.emit_jump(OpCode::JumpIfFalse);
                let end_jump = self.emit_jump(OpCode::Jump);

                self.patch_jump(else_jump);
                self.chunk.write(OpCode::Pop);

                self.translate_expression(right_expr);
                self.patch_jump(end_jump);
            }
            And(left_expr, right_expr) => {
                self.translate_expression(left_expr);
                let end_jump = self.emit_jump(OpCode::JumpIfFalse);
                self.chunk.write(OpCode::Pop);
                self.translate_expression(right_expr);
                self.patch_jump(end_jump);
            }
            Boolean(bool) => match bool {
                true => self.chunk.write(OpCode::True),
                false => self.chunk.write(OpCode::False),
            },
            FunctionCall(_function_name, args, _rox_type) => {
                args.iter().for_each(|arg| self.translate_expression(arg));
                self.chunk.write(OpCode::Call(args.len()));
            }
            Array(_tagged_expressions, _type_) => todo!(),
            // TODO: escape characters, template strings
            String(string) => {
                self.chunk
                    .add_constant(Value::create_string(string.clone()));
            }
            Identifier(name, _rox_type) => self
                .chunk
                .write(OpCode::ReadVariable(self.resolve_local(name))),
            StructInstantiation(_struct_type, _fields) => todo!(),
            Operation(left, operation, right) => {
                self.translate_expression(left);
                self.translate_expression(right);
                let op = match operation {
                    parser::Operation::Add => OpCode::Add,
                    parser::Operation::Multiply => OpCode::Multiply,
                    parser::Operation::Subtract => OpCode::Subtract,
                    parser::Operation::Divide => OpCode::Divide,
                    parser::Operation::Equals => OpCode::Equal,
                    parser::Operation::GreaterThan => OpCode::Greater,
                    parser::Operation::LessThan => OpCode::Less,
                    parser::Operation::NotEquals => {
                        self.chunk.write(OpCode::Not);
                        OpCode::Equal
                    }
                };
                self.chunk.write(op);
            }
            Number(num) => {
                let value = Value::Number(*num);
                self.chunk.add_constant(value);
            }
            Unary(unary, expr, _type) => {
                self.translate_expression(expr);
                match unary {
                    parser::Unary::Negate => self.chunk.write(OpCode::Negate),
                    parser::Unary::Not => self.chunk.write(OpCode::Not),
                }
            }
        }
    }

    fn start_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        let mut index = self.locals.len() - 1;
        while !self.locals.is_empty()
            && self.locals[index].depth > self.scope_depth
        {
            self.chunk.write(OpCode::Pop);
            self.locals.pop();
            index -= 1;
        }
    }

    fn resolve_local(&self, name: &str) -> usize {
        for (index, local) in self.locals.iter().rev().enumerate() {
            if local.name == name {
                return index;
            }
        }
        unreachable!()
    }

    fn emit_jump(&mut self, instruction: OpCode) -> usize {
        self.chunk.write(instruction);
        self.chunk.write(OpCode::Placeholder);
        self.chunk.opcodes.len() - 1
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.chunk.write(OpCode::Loop);
        let offset = self.chunk.opcodes.len() - loop_start + 1;
        self.chunk.write(OpCode::Offset(offset));
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.opcodes.len() - offset - 1;
        self.chunk.opcodes[offset] = OpCode::Offset(jump);
    }
}
