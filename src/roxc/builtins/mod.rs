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
    let mut variable_env = HashMap::new();

    type_env.insert(
        "Number".to_string(),
        TypeValue::from(&Type::Apply(TypeConstructor::Number, Vec::new())),
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
        "Array".to_string(),
        TypeValue::Constructor(TypeConstructor::Array),
    );
    type_env.insert(
        "String".to_string(),
        TypeValue::from(&Type::Apply(TypeConstructor::Number, Vec::new())),
    );
    import_libc_bindings(&mut variable_env);
    let mut function_stack: Stack<HashMap<String, FunctionDeclaration>> =
        Stack::new();
    function_stack.push(HashMap::new());
    function_stack.top_mut().insert(
        "puts".to_string(),
        FunctionDeclaration {
            name: "puts".to_string(),
            params: vec![(
                "arg".to_string(),
                Type::Apply(TypeConstructor::String, Vec::new()),
            )],
            return_type: Type::Apply(TypeConstructor::Void, Vec::new()),
        },
    );
    (type_env, variable_env, function_stack)
}

fn import_libc_bindings(env: &mut VariableEnv) {
    env.insert(
        "puts".to_string(),
        Type::PolymorphicType(
            Vec::new(),
            Box::new(Type::Apply(
                TypeConstructor::Arrow,
                vec![
                    Type::Apply(TypeConstructor::String, Vec::new()),
                    Type::Apply(TypeConstructor::Void, Vec::new()),
                ],
            )),
        ),
    );
}
