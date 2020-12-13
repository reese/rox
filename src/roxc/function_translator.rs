use crate::roxc::local::Local;
use crate::roxc::vm::{Chunk, OpCode, Value};
use crate::roxc::{parser, Result, RoxError};
use crate::roxc::{TaggedExpression, TaggedStatement};

pub(crate) struct FunctionTranslator<'c> {
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
            chunk,
            locals,
            scope_depth,
        }
    }

    pub(crate) fn translate_function(
        &mut self,
        block: &[TaggedStatement],
    ) -> Result<()> {
        block
            .iter()
            .map(|statement| self.translate_statement(statement))
            .collect::<Result<Vec<_>>>()?;
        Ok(())
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
            FunctionDeclaration(..) => todo!(),
            // TODO: Do we need external functions like this
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
                if let Some(expr) = maybe_expression.as_ref() {
                    self.translate_expression(expr)
                }
                self.chunk.write(OpCode::Return);
                Ok(())
            }
            IfElse(conditional, if_statements, else_statements_maybe) => {
                self.translate_expression(conditional);
                let then_jump = self.emit_jump(OpCode::JumpIfFalse);
                self.chunk.write(OpCode::Pop);
                if_statements
                    .iter()
                    .map(|statement| self.translate_statement(statement))
                    .collect::<Result<Vec<_>>>()?;
                let else_jump = self.emit_jump(OpCode::Jump);
                self.patch_jump(then_jump);
                self.chunk.write(OpCode::Pop);
                if let Some(else_statements) = else_statements_maybe {
                    else_statements
                        .iter()
                        .map(|statement| self.translate_statement(statement))
                        .collect::<Result<Vec<_>>>()?;
                }
                self.patch_jump(else_jump);
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
            FunctionCall(_function_name, _args, _rox_type) => todo!(),
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

    fn resolve_local(&self, name: &String) -> usize {
        for (index, local) in self.locals.iter().rev().enumerate() {
            if &local.name == name {
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

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.opcodes.len() - offset - 1;
        self.chunk.opcodes[offset] = OpCode::Constant(jump);
    }
}
