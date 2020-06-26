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
    // These Vecs are empty because these operators are
    // built in and thus don't map to other types
    let mut types = vec![
        Type::new_operator(String::from("Void"), vec![]),
        Type::new_operator(String::from("Number"), vec![]),
        Type::new_operator(String::from("Bool"), vec![]),
        Type::new_operator(String::from("String"), vec![]),
    ];
    let mut env = HashMap::new();
    env.insert(String::from("Number"), NUMBER_TYPE_VAL);
    env.insert(String::from("Bool"), BOOL_TYPE_VAL);
    env.insert(String::from("String"), STRING_TYPE_VAL);
    env.insert(String::from("Void"), VOID_TYPE_VAL);
    import_libc_bindings(&mut types, &mut env);
    (types, env)
}

fn import_libc_bindings(types: &mut Vec<Type>, env: &mut Env) {
    get_libc_types()
        .iter()
        .for_each(|(func_name, arg_types, return_types)| {
            let func =
                new_function(func_name.clone(), types, arg_types, return_types);
            env.insert(func_name.clone(), func);
        });
}
