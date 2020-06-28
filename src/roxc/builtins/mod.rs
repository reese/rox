use crate::roxc::types::{new_function, Type};
use crate::roxc::{
    ArenaType, Env, Identifier, ARRAY_TYPE_VAL, BOOL_TYPE_VAL, NUMBER_TYPE_VAL,
    STRING_TYPE_VAL, VOID_TYPE_VAL,
};
use std::collections::HashMap;

fn get_libc_types() -> [(String, Vec<ArenaType>, Vec<ArenaType>); 1] {
    [(
        "puts".to_string(),
        vec![STRING_TYPE_VAL],
        Vec::<ArenaType>::new(),
    )]
}

pub(crate) fn get_builtin_types() -> (Vec<Type>, Env) {
    // Built-in types are special-cased structs with no fields
    let void_ident = Identifier::new_non_generic("Void".to_string());
    let number_ident = Identifier::new_non_generic("Number".to_string());
    let bool_ident = Identifier::new_non_generic("Bool".to_string());
    let string_ident = Identifier::new_non_generic("String".to_string());
    let array_ident = Identifier::new(
        "Array".to_string(),
        vec![Identifier::new_non_generic("T".to_string())],
    );
    let mut types = vec![
        Type::new_struct(void_ident.clone(), Vec::new(), HashMap::new()),
        Type::new_struct(number_ident.clone(), Vec::new(), HashMap::new()),
        Type::new_struct(bool_ident.clone(), Vec::new(), HashMap::new()),
        Type::new_struct(string_ident.clone(), Vec::new(), HashMap::new()),
        Type::new_struct(
            array_ident.clone(),
            vec!["T".to_string()],
            HashMap::new(),
        ),
    ];
    let mut env = HashMap::new();
    env.insert(number_ident.get_name(), NUMBER_TYPE_VAL);
    env.insert(bool_ident.get_name(), BOOL_TYPE_VAL);
    env.insert(string_ident.get_name(), STRING_TYPE_VAL);
    env.insert(void_ident.get_name(), VOID_TYPE_VAL);
    env.insert(array_ident.get_name(), ARRAY_TYPE_VAL);
    import_libc_bindings(&mut types, &mut env);
    (types, env)
}

fn import_libc_bindings(types: &mut Vec<Type>, env: &mut Env) {
    get_libc_types()
        .iter()
        .for_each(|(func_name, arg_types, return_types)| {
            let func = new_function(
                Identifier::new_non_generic(func_name.clone()),
                types,
                arg_types,
                return_types,
            );
            env.insert(func_name.clone(), func);
        });
}
