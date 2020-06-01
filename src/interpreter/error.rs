use super::super::errors::{OliveError, OliveRuntimeError};
use std::collections::HashMap;

pub fn create_runtime_error(
    position: usize,
    code_pos_table: &HashMap<usize, usize>,
    filename: &str,
    source: Option<&str>,
    data: OliveRuntimeError,
) -> OliveError {
    if let Some(source) = source {
        OliveError::new_runtime_error(
            Some(*code_pos_table.get(&position).unwrap()),
            filename,
            source,
            data,
        )
    } else {
        OliveError::new_runtime_error(None, filename, "", data)
    }
}

pub fn create_type_error(
    position: usize,
    code_pos_table: &HashMap<usize, usize>,
    filename: &str,
    source: Option<&str>,
    expected: Vec<&str>,
    got: &str,
) -> OliveError {
    create_runtime_error(
        position,
        code_pos_table,
        filename,
        source,
        OliveRuntimeError::IncorrectType {
            expected: expected.into_iter().map(|s| String::from(s)).collect(),
            got: String::from(got),
        },
    )
}

pub fn create_variable_error(
    position: usize,
    code_pos_table: &HashMap<usize, usize>,
    filename: &str,
    source: Option<&str>,
    name: &str,
) -> OliveError {
    create_runtime_error(
        position,
        code_pos_table,
        filename,
        source,
        OliveRuntimeError::VariableNotFound {
            name: String::from(name),
        },
    )
}

pub fn create_binop_type_error(
    position: usize,
    code_pos_table: &HashMap<usize, usize>,
    filename: &str,
    source: Option<&str>,
    left: &str,
    right: &str,
) -> OliveError {
    create_runtime_error(
        position,
        code_pos_table,
        filename,
        source,
        OliveRuntimeError::UnmatchingTypes {
            left: String::from(left),
            right: String::from(right),
        },
    )
}

pub fn create_call_error(
    position: usize,
    code_pos_table: &HashMap<usize, usize>,
    filename: &str,
    source: Option<&str>,
    got: usize,
    expected: usize,
) -> OliveError {
    create_runtime_error(
        position,
        code_pos_table,
        filename,
        source,
        OliveRuntimeError::CallArgs {
            expected: expected,
            got: got,
        },
    )
}
