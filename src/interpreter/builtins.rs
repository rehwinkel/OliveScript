use super::object::{Object, RefObject};
use std::collections::HashMap;

fn native_print(args: Vec<Object>) -> Object {
    println!(
        "{}",
        args.iter()
            .map(|o| o.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );
    Object::new_none()
}

fn native_len(args: Vec<Object>) -> Object {
    Object::new_integer(match &args[0] {
        Object::Pointer { value: v } => match &**v {
            RefObject::Bendy { data } => data.len() as i64,
            RefObject::List { data } => data.len() as i64,
            RefObject::String { value } => value.len() as i64,
            _ => return Object::None,
        },
        _ => return Object::None,
    })
}

pub fn get_functions() -> HashMap<String, Object> {
    let mut functions = HashMap::new();
    functions.insert(
        String::from("print"),
        Object::new_native(1, native_print as fn(Vec<Object>) -> Object),
    );
    functions.insert(
        String::from("len"),
        Object::new_native(1, native_len as fn(Vec<Object>) -> Object),
    );
    functions
}
