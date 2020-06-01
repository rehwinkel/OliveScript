use super::super::codegen::Code;
use super::super::errors::OliveError;
use super::error;
use super::garbage::{Garbage, ReferenceHolder};
use std::collections::HashMap;

pub enum Data {
    Function {
        args: Vec<String>,
        codes: Vec<Code>,
    },
    Integer {
        value: i64,
    },
    Float {
        value: f64,
    },
    Boolean {
        value: bool,
    },
    String {
        value: String,
    },
    List {
        data: Vec<Garbage<Data>>,
    },
    Bendy {
        data: HashMap<String, Garbage<Data>>,
    },
    Native {
        arg_count: u32,
        closure: fn(Vec<Garbage<Data>>) -> Data,
    },
    None,
}

impl Data {
    pub fn as_integer(
        &self,
        position: usize,
        code_pos_table: &HashMap<usize, usize>,
        filename: &str,
        source: Option<&str>,
    ) -> Result<i64, OliveError> {
        match self {
            Data::Integer { value } => Ok(*value),
            t => Err(error::create_type_error(
                position,
                code_pos_table,
                filename,
                source,
                vec!["integer"],
                t.get_name(),
            )),
        }
    }
    pub fn as_string(
        &self,
        position: usize,
        code_pos_table: &HashMap<usize, usize>,
        filename: &str,
        source: Option<&str>,
    ) -> Result<&str, OliveError> {
        match self {
            Data::String { value } => Ok(value),
            t => Err(error::create_type_error(
                position,
                code_pos_table,
                filename,
                source,
                vec!["string"],
                t.get_name(),
            )),
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            Data::Integer { value: _ } => "integer",
            Data::Float { value: _ } => "float",
            Data::None => "none",
            Data::Boolean { value: _ } => "boolean",
            Data::Function { args: _, codes: _ } => "function",
            Data::String { value: _ } => "string",
            Data::List { data: _ } => "list",
            Data::Bendy { data: _ } => "bendy",
            Data::Native {
                arg_count: _,
                closure: _,
            } => "native",
        }
    }
}

impl ReferenceHolder<Data> for Data {
    fn get_references(&self) -> Vec<Garbage<Data>> {
        match self {
            Data::List { data } => data.iter().map(|e| e.clone()).collect(),
            Data::Bendy { data } => data.values().map(|e| e.clone()).collect(),
            _ => Vec::new(),
        }
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Data::Integer { value: v1 } => match other {
                Data::Integer { value: v2 } => v1 == v2,
                _ => false,
            },
            Data::Float { value: v1 } => match other {
                Data::Float { value: v2 } => v1 == v2,
                _ => false,
            },
            Data::String { value: v1 } => match other {
                Data::String { value: v2 } => v1 == v2,
                _ => false,
            },
            Data::Boolean { value: v1 } => match other {
                Data::Boolean { value: v2 } => v1 == v2,
                _ => false,
            },
            Data::List { data: v1 } => match other {
                Data::List { data: v2 } => v1 == v2,
                _ => false,
            },
            Data::Bendy { data: v1 } => match other {
                Data::Bendy { data: v2 } => v1 == v2,
                _ => false,
            },
            Data::None => match other {
                Data::None => true,
                _ => false,
            },
            Data::Function {
                args: args1,
                codes: codes1,
            } => match other {
                Data::Function {
                    args: args2,
                    codes: codes2,
                } => args1 == args2 && codes1 == codes2,
                _ => false,
            },
            Data::Native {
                arg_count: a1,
                closure: c1,
            } => match other {
                Data::Native {
                    arg_count: a2,
                    closure: c2,
                } => a1 == a2 && c1 == c2,
                _ => false,
            },
        }
    }
}

impl Data {
    pub fn truthy(&self) -> bool {
        match self {
            Data::Integer { value } => *value != 0,
            Data::Boolean { value } => *value,
            Data::Float { value } => *value != 0.0,
            Data::String { value } => value.len() > 0,
            Data::List { data } => data.len() > 0,
            Data::Bendy { data } => data.len() > 0,
            Data::None => false,
            Data::Function { args: _, codes: _ } => true,
            Data::Native {
                arg_count: _,
                closure: _,
            } => true,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Data::Integer { value } => format!("{}", value),
            Data::Boolean { value } => format!("{}", value),
            Data::Float { value } => format!("{}", value),
            Data::String { value } => value.clone(),
            Data::List { data } => format!(
                "[{}]",
                data.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Data::Bendy { data } => format!(
                "{{{}}}",
                data.iter()
                    .map(|(k, v)| format!("{}: {}", k.to_string(), v.to_string()))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Data::None => String::from("none"),
            Data::Function { args, codes: _ } => format!("function({})", args.join(", ")),
            Data::Native {
                arg_count: _,
                closure,
            } => format!("native({:?})", closure),
        }
    }

    fn operate_int(a: i64, b: i64, operation: &Code) -> i64 {
        match operation {
            Code::Add => a + b,
            Code::Sub => a - b,
            Code::Mod => a % b,
            Code::Mul => a * b,
            Code::BitAnd => a & b,
            Code::BitOr => a | b,
            Code::BitXOr => a ^ b,
            Code::BitLsh => a.checked_shl(b as u32).unwrap_or(0),
            Code::BitRsh => a.checked_shr(b as u32).unwrap_or(0),
            _ => panic!(),
        }
    }

    fn compare_int(a: i64, b: i64, operation: &Code) -> bool {
        match operation {
            Code::LessEquals => a <= b,
            Code::GreaterEquals => a >= b,
            Code::LessThan => a < b,
            Code::GreaterThan => a > b,
            _ => panic!(),
        }
    }

    fn compare_float(a: f64, b: f64, operation: &Code) -> bool {
        match operation {
            Code::LessEquals => a <= b,
            Code::GreaterEquals => a >= b,
            Code::LessThan => a < b,
            Code::GreaterThan => a > b,
            _ => panic!(),
        }
    }

    fn operate_float(a: f64, b: f64, operation: &Code) -> f64 {
        match operation {
            Code::Add => a + b,
            Code::Sub => a - b,
            Code::Mod => a % b,
            Code::Mul => a * b,
            _ => panic!(),
        }
    }

    pub fn operate(
        &self,
        other: &Self,
        position: usize,
        code_pos_table: &HashMap<usize, usize>,
        filename: &str,
        source: Option<&str>,
        operation: &Code,
    ) -> Result<Self, OliveError> {
        match operation {
            Code::Add | Code::Sub | Code::Mod | Code::Mul => match self {
                Data::Integer { value: v1 } => match other {
                    Data::Integer { value: v2 } => {
                        return Ok(Data::Integer {
                            value: Data::operate_int(*v1, *v2, operation),
                        })
                    }
                    Data::Float { value: v2 } => {
                        return Ok(Data::Float {
                            value: Data::operate_float(*v1 as f64, *v2, operation),
                        })
                    }
                    _ => {}
                },
                Data::Float { value: v1 } => match other {
                    Data::Float { value: v2 } => {
                        return Ok(Data::Float {
                            value: Data::operate_float(*v1, *v2, operation),
                        })
                    }
                    Data::Integer { value: v2 } => {
                        return Ok(Data::Float {
                            value: Data::operate_float(*v1, *v2 as f64, operation),
                        })
                    }
                    _ => {}
                },
                _ => {}
            },
            Code::LessEquals | Code::LessThan | Code::GreaterEquals | Code::GreaterThan => {
                match self {
                    Data::Integer { value: v1 } => match other {
                        Data::Integer { value: v2 } => {
                            return Ok(Data::Boolean {
                                value: Data::compare_int(*v1, *v2, operation),
                            })
                        }
                        _ => {}
                    },
                    Data::Float { value: v1 } => match other {
                        Data::Float { value: v2 } => {
                            return Ok(Data::Boolean {
                                value: Data::compare_float(*v1, *v2, operation),
                            })
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Code::BitAnd | Code::BitOr | Code::BitXOr | Code::BitLsh | Code::BitRsh => match self {
                Data::Integer { value: v1 } => match other {
                    Data::Integer { value: v2 } => {
                        return Ok(Data::Integer {
                            value: Data::operate_int(*v1, *v2, operation),
                        })
                    }
                    _ => {}
                },
                _ => {}
            },
            Code::Concat => match self {
                Data::String { value: v1 } => {
                    return Ok(Data::String {
                        value: format!("{}{}", v1, other.to_string()),
                    })
                }
                Data::List { data: d1 } => match other {
                    Data::List { data: d2 } => {
                        let mut result = d1.clone();
                        result.extend(d2.clone());
                        return Ok(Data::List { data: result });
                    }
                    _ => {}
                },
                Data::Bendy { data: d1 } => match other {
                    Data::Bendy { data: d2 } => {
                        let mut result = d1.clone();
                        result.extend(d2.clone());
                        return Ok(Data::Bendy { data: result });
                    }
                    _ => {}
                },
                _ => {}
            },
            Code::FloatDiv => {
                let a: f64 = match self {
                    Data::Float { value } => *value,
                    Data::Integer { value } => *value as f64,
                    _ => {
                        return Err(error::create_binop_type_error(
                            position,
                            code_pos_table,
                            filename,
                            source,
                            self.get_name(),
                            other.get_name(),
                        ))
                    }
                };
                let b: f64 = match other {
                    Data::Float { value } => *value,
                    Data::Integer { value } => *value as f64,
                    _ => {
                        return Err(error::create_binop_type_error(
                            position,
                            code_pos_table,
                            filename,
                            source,
                            self.get_name(),
                            other.get_name(),
                        ))
                    }
                };
                return Ok(Data::Float { value: a / b });
            }
            Code::IntDiv => {
                let a: f64 = match self {
                    Data::Float { value } => *value,
                    Data::Integer { value } => *value as f64,
                    _ => {
                        return Err(error::create_binop_type_error(
                            position,
                            code_pos_table,
                            filename,
                            source,
                            self.get_name(),
                            other.get_name(),
                        ))
                    }
                };
                let b: f64 = match other {
                    Data::Float { value } => *value,
                    Data::Integer { value } => *value as f64,
                    _ => {
                        return Err(error::create_binop_type_error(
                            position,
                            code_pos_table,
                            filename,
                            source,
                            self.get_name(),
                            other.get_name(),
                        ))
                    }
                };
                return Ok(Data::Integer {
                    value: (a / b) as i64,
                });
            }
            Code::Equals => {
                return Ok(Data::Boolean {
                    value: self == other,
                })
            }
            Code::NotEquals => {
                return Ok(Data::Boolean {
                    value: self != other,
                })
            }
            _ => {}
        }
        return Err(error::create_binop_type_error(
            position,
            code_pos_table,
            filename,
            source,
            self.get_name(),
            other.get_name(),
        ));
    }
}
