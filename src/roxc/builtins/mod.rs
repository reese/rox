use crate::roxc::types::{new_function, Type};
use crate::roxc::{
    ArenaType, Env, Identifier, BOOL_TYPE_VAL, NUMBER_TYPE_VAL,
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
    let number_ident = Identifier::new_non_generic("Number".to_string());
    let void_ident = Identifier::new_non_generic("Void".to_string());
    let bool_ident = Identifier::new_non_generic("Bool".to_string());
    let string_ident = Identifier::new_non_generic("String".to_string());
    // These Vecs are empty because these operators are
    // built in and thus don't map to other types
    let mut types = vec![
        Type::new_operator(void_ident.clone(), vec![]),
        Type::new_operator(number_ident.clone(), vec![]),
        Type::new_operator(bool_ident.clone(), vec![]),
        Type::new_operator(string_ident.clone(), vec![]),
    ];
    let mut env = HashMap::new();
    env.insert(number_ident, NUMBER_TYPE_VAL);
    env.insert(bool_ident, BOOL_TYPE_VAL);
    env.insert(string_ident, STRING_TYPE_VAL);
    env.insert(void_ident, VOID_TYPE_VAL);
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
            env.insert(Identifier::from(func_name.clone()), func);
        });
}
