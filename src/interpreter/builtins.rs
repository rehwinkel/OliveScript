use super::data::Data;
use super::garbage::Garbage;
use std::collections::HashMap;

fn native_print(args: Vec<Garbage<Data>>) -> Data {
    println!(
        "{}",
        args.iter()
            .map(|o| o.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );
    Data::None
}

fn native_len(args: Vec<Garbage<Data>>) -> Data {
    Data::Integer {
        value: match &*args[0] {
            Data::Bendy { data } => data.len() as i64,
            Data::List { data } => data.len() as i64,
            Data::String { value } => value.len() as i64,
            _ => return Data::None,
        },
    }
}

pub fn get_functions() -> HashMap<String, Data> {
    let mut functions = HashMap::new();
    functions.insert(
        String::from("print"),
        Data::Native {
            arg_count: 1,
            closure: native_print as fn(Vec<Garbage<Data>>) -> Data,
        },
    );
    functions.insert(
        String::from("len"),
        Data::Native {
            arg_count: 1,
            closure: native_len as fn(Vec<Garbage<Data>>) -> Data,
        },
    );
    functions
}
