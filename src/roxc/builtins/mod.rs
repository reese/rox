use crate::roxc::types::{new_function, Type};
use crate::roxc::{
    ArenaType, Env, BOOL_TYPE_VAL, NUMBER_TYPE_VAL, STRING_TYPE_VAL,
    VOID_TYPE_VAL,
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
    let initial_types = vec![
        VOID_TYPE_VAL,
        NUMBER_TYPE_VAL,
        BOOL_TYPE_VAL,
        STRING_TYPE_VAL,
    ];
    let mut env = HashMap::new();
    env.insert(String::from("Number"), NUMBER_TYPE_VAL);
    env.insert(String::from("Bool"), BOOL_TYPE_VAL);
    env.insert(String::from("String"), STRING_TYPE_VAL);
    let mut types = initial_types
        .iter()
        .map(|arena_type| Type::new_variable(*arena_type))
        .collect::<Vec<_>>();
    import_libc_bindings(&mut types, &mut env);
    (types, env)
}

fn import_libc_bindings(types: &mut Vec<Type>, env: &mut Env) {
    get_libc_types()
        .iter()
        .for_each(|(func_name, arg_types, return_types)| {
            let func = new_function(types, arg_types, return_types);
            env.insert(func_name.clone(), func);
        });
}
