use crate::roxc::parser::FunctionDeclaration;
use crate::roxc::stack::Stack;
use crate::roxc::{Type, TypeConstructor, TypeEnv, TypeValue, VariableEnv};
use std::collections::HashMap;

pub(crate) fn get_builtin_types() -> (
    TypeEnv,
    VariableEnv,
    Stack<HashMap<String, FunctionDeclaration>>,
) {
    let mut type_env = HashMap::new();
    let variable_env = HashMap::new();

    type_env.insert(
        "Int".to_string(),
        TypeValue::from(&Type::Apply(TypeConstructor::Int, Vec::new())),
    );
    type_env.insert(
        "Float".to_string(),
        TypeValue::from(&Type::Apply(TypeConstructor::Float, Vec::new())),
    );
    type_env.insert(
        "String".to_string(),
        TypeValue::from(&Type::Apply(TypeConstructor::String, Vec::new())),
    );
    type_env.insert(
        "Void".to_string(),
        TypeValue::from(&Type::Apply(TypeConstructor::Void, Vec::new())),
    );
    type_env.insert(
        "Bool".to_string(),
        TypeValue::from(&Type::Apply(TypeConstructor::Bool, Vec::new())),
    );
    let mut function_stack: Stack<HashMap<String, FunctionDeclaration>> =
        Stack::new();
    function_stack.push(HashMap::new());
    (type_env, variable_env, function_stack)
}
