use super::super::codegen::Code;
use super::super::errors::OliveError;
use super::error;
use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub enum RefObject {
    Function {
        args: Vec<String>,
        codes: Vec<Code>,
    },
    String {
        value: String,
    },
    List {
        data: Vec<Object>,
    },
    Bendy {
        data: HashMap<String, Object>,
    },
    Native {
        arg_count: u32,
        closure: fn(Vec<Object>) -> Object,
    },
}

#[derive(Clone)]
pub enum Object {
    Integer { value: i64 },
    Float { value: f64 },
    Boolean { value: bool },
    None,
    Pointer { value: Garbage<RefObject> },
}

impl From<Garbage<RefObject>> for Object {
    fn from(value: Garbage<RefObject>) -> Self {
        Object::Pointer { value }
    }
}

impl RefObject {
    pub fn get_type_name(&self) -> &str {
        match self {
            RefObject::Function { args: _, codes: _ } => "function",
            RefObject::String { value: _ } => "string",
            RefObject::List { data: _ } => "list",
            RefObject::Bendy { data: _ } => "bendy",
            RefObject::Native {
                arg_count: _,
                closure: _,
            } => "native",
        }
    }
}

impl ToString for Object {
    fn to_string(&self) -> String {
        match self {
            Object::Integer { value } => format!("{}", value),
            Object::Boolean { value } => format!("{}", value),
            Object::Float { value } => format!("{}", value),
            Object::None => String::from("none"),
            Object::Pointer { value: v } => match &**v {
                RefObject::String { value } => value.clone(),
                RefObject::List { data } => format!(
                    "[{}]",
                    data.iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                RefObject::Bendy { data } => format!(
                    "{{{}}}",
                    data.iter()
                        .map(|(k, v)| format!("{}: {}", k.to_string(), v.to_string()))
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                RefObject::Function { args, codes: _ } => format!("function({})", args.join(", ")),
                RefObject::Native {
                    arg_count: _,
                    closure,
                } => format!("native({:?})", closure),
            },
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Object::Integer { value: v1 } => match other {
                Object::Integer { value: v2 } => v1 == v2,
                _ => false,
            },
            Object::Float { value: v1 } => match other {
                Object::Float { value: v2 } => v1 == v2,
                _ => false,
            },
            Object::Boolean { value: v1 } => match other {
                Object::Boolean { value: v2 } => v1 == v2,
                _ => false,
            },
            Object::None => match other {
                Object::None => true,
                _ => false,
            },
            Object::Pointer { value: v } => match &**v {
                RefObject::String { value: v1 } => match other {
                    Object::Pointer { value: v } => match &**v {
                        RefObject::String { value: v2 } => v1 == v2,
                        _ => false,
                    },
                    _ => false,
                },
                RefObject::List { data: v1 } => match other {
                    Object::Pointer { value: v } => match &**v {
                        RefObject::List { data: v2 } => v1 == v2,
                        _ => false,
                    },
                    _ => false,
                },
                RefObject::Bendy { data: v1 } => match other {
                    Object::Pointer { value: v } => match &**v {
                        RefObject::Bendy { data: v2 } => v1 == v2,
                        _ => false,
                    },
                    _ => false,
                },
                RefObject::Function {
                    args: args1,
                    codes: codes1,
                } => match other {
                    Object::Pointer { value: v } => match &**v {
                        RefObject::Function {
                            args: args2,
                            codes: codes2,
                        } => args1 == args2 && codes1 == codes2,
                        _ => false,
                    },
                    _ => false,
                },
                RefObject::Native {
                    arg_count: a1,
                    closure: c1,
                } => match other {
                    Object::Pointer { value: v } => match &**v {
                        RefObject::Native {
                            arg_count: a2,
                            closure: c2,
                        } => a1 == a2 && c1 == c2,
                        _ => false,
                    },
                    _ => false,
                },
            },
        }
    }
}

impl Object {
    pub fn new_none() -> Self {
        Object::None
    }
    pub fn new_integer(value: i64) -> Self {
        Object::Integer { value }
    }
    pub fn new_float(value: f64) -> Self {
        Object::Float { value }
    }
    pub fn new_boolean(value: bool) -> Self {
        Object::Boolean { value }
    }
    pub fn new_function(args: Vec<String>, codes: Vec<Code>) -> Self {
        Object::Pointer {
            value: Garbage::new(RefObject::Function { args, codes }),
        }
    }
    pub fn new_native(arg_count: u32, closure: fn(Vec<Object>) -> Object) -> Self {
        Object::Pointer {
            value: Garbage::new(RefObject::Native { arg_count, closure }),
        }
    }
    pub fn new_bendy() -> Self {
        Object::Pointer {
            value: Garbage::new(RefObject::Bendy {
                data: HashMap::new(),
            }),
        }
    }
    pub fn new_list() -> Self {
        Object::Pointer {
            value: Garbage::new(RefObject::List { data: Vec::new() }),
        }
    }
    pub fn new_filled_list(data: Vec<Object>) -> Self {
        Object::Pointer {
            value: Garbage::new(RefObject::List { data }),
        }
    }
    pub fn new_filled_bendy(data: HashMap<String, Object>) -> Self {
        Object::Pointer {
            value: Garbage::new(RefObject::Bendy { data }),
        }
    }
    pub fn new_string(value: String) -> Self {
        Object::Pointer {
            value: Garbage::new(RefObject::String { value }),
        }
    }

    pub fn get_type_name(&self) -> &str {
        match self {
            Object::Integer { value: _ } => "integer",
            Object::Float { value: _ } => "float",
            Object::None => "none",
            Object::Boolean { value: _ } => "boolean",
            Object::Pointer { value } => value.get_type_name(),
        }
    }
    pub fn truthy(&self) -> bool {
        match self {
            Object::Integer { value } => *value != 0,
            Object::Boolean { value } => *value,
            Object::Float { value } => *value != 0.0,
            Object::None => false,
            Object::Pointer { value } => match &**value {
                RefObject::String { value } => value.len() > 0,
                RefObject::List { data } => data.len() > 0,
                RefObject::Bendy { data } => data.len() > 0,
                RefObject::Function { args: _, codes: _ } => true,
                RefObject::Native {
                    arg_count: _,
                    closure: _,
                } => true,
            },
        }
    }
    pub fn as_integer(
        &self,
        position: usize,
        code_pos_table: &HashMap<usize, usize>,
        filename: &str,
        source: Option<&str>,
    ) -> Result<i64, OliveError> {
        match self {
            Object::Integer { value } => Ok(*value),
            t => Err(error::create_type_error(
                position,
                code_pos_table,
                filename,
                source,
                vec!["integer"],
                t.get_type_name(),
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
            Object::Pointer { value } => match &**value {
                RefObject::String { value } => Ok(value),
                t => Err(error::create_type_error(
                    position,
                    code_pos_table,
                    filename,
                    source,
                    vec!["string"],
                    t.get_type_name(),
                )),
            },
            t => Err(error::create_type_error(
                position,
                code_pos_table,
                filename,
                source,
                vec!["string"],
                t.get_type_name(),
            )),
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
                Object::Integer { value: v1 } => match other {
                    Object::Integer { value: v2 } => {
                        return Ok(Object::Integer {
                            value: Object::operate_int(*v1, *v2, operation),
                        })
                    }
                    Object::Float { value: v2 } => {
                        return Ok(Object::Float {
                            value: Object::operate_float(*v1 as f64, *v2, operation),
                        })
                    }
                    _ => {}
                },
                Object::Float { value: v1 } => match other {
                    Object::Float { value: v2 } => {
                        return Ok(Object::Float {
                            value: Object::operate_float(*v1, *v2, operation),
                        })
                    }
                    Object::Integer { value: v2 } => {
                        return Ok(Object::Float {
                            value: Object::operate_float(*v1, *v2 as f64, operation),
                        })
                    }
                    _ => {}
                },
                _ => {}
            },
            Code::LessEquals | Code::LessThan | Code::GreaterEquals | Code::GreaterThan => {
                match self {
                    Object::Integer { value: v1 } => match other {
                        Object::Integer { value: v2 } => {
                            return Ok(Object::Boolean {
                                value: Object::compare_int(*v1, *v2, operation),
                            })
                        }
                        _ => {}
                    },
                    Object::Float { value: v1 } => match other {
                        Object::Float { value: v2 } => {
                            return Ok(Object::Boolean {
                                value: Object::compare_float(*v1, *v2, operation),
                            })
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            Code::BitAnd | Code::BitOr | Code::BitXOr | Code::BitLsh | Code::BitRsh => match self {
                Object::Integer { value: v1 } => match other {
                    Object::Integer { value: v2 } => {
                        return Ok(Object::Integer {
                            value: Object::operate_int(*v1, *v2, operation),
                        })
                    }
                    _ => {}
                },
                _ => {}
            },
            Code::Concat => match self {
                Object::Pointer { value: v } => match &**v {
                    RefObject::String { value: v1 } => {
                        return Ok(Object::new_string(format!("{}{}", v1, other.to_string())))
                    }
                    RefObject::List { data: d1 } => match other {
                        Object::Pointer { value: v } => match &**v {
                            RefObject::List { data: d2 } => {
                                let mut result = d1.clone();
                                result.extend(d2.clone());
                                return Ok(Object::new_filled_list(result));
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    RefObject::Bendy { data: d1 } => match other {
                        Object::Pointer { value: v } => match &**v {
                            RefObject::Bendy { data: d2 } => {
                                let mut result = d1.clone();
                                result.extend(d2.clone());
                                return Ok(Object::new_filled_bendy(result));
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            },
            Code::FloatDiv => {
                let a: f64 = match self {
                    Object::Float { value } => *value,
                    Object::Integer { value } => *value as f64,
                    _ => {
                        return Err(error::create_binop_type_error(
                            position,
                            code_pos_table,
                            filename,
                            source,
                            self.get_type_name(),
                            other.get_type_name(),
                        ))
                    }
                };
                let b: f64 = match other {
                    Object::Float { value } => *value,
                    Object::Integer { value } => *value as f64,
                    _ => {
                        return Err(error::create_binop_type_error(
                            position,
                            code_pos_table,
                            filename,
                            source,
                            self.get_type_name(),
                            other.get_type_name(),
                        ))
                    }
                };
                return Ok(Object::Float { value: a / b });
            }
            Code::IntDiv => {
                let a: f64 = match self {
                    Object::Float { value } => *value,
                    Object::Integer { value } => *value as f64,
                    _ => {
                        return Err(error::create_binop_type_error(
                            position,
                            code_pos_table,
                            filename,
                            source,
                            self.get_type_name(),
                            other.get_type_name(),
                        ))
                    }
                };
                let b: f64 = match other {
                    Object::Float { value } => *value,
                    Object::Integer { value } => *value as f64,
                    _ => {
                        return Err(error::create_binop_type_error(
                            position,
                            code_pos_table,
                            filename,
                            source,
                            self.get_type_name(),
                            other.get_type_name(),
                        ))
                    }
                };
                return Ok(Object::Integer {
                    value: (a / b) as i64,
                });
            }
            Code::Equals => {
                return Ok(Object::Boolean {
                    value: self == other,
                })
            }
            Code::NotEquals => {
                return Ok(Object::Boolean {
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
            self.get_type_name(),
            other.get_type_name(),
        ));
    }
}

pub struct Garbage<T> {
    data: *mut T,
    refcount: *mut usize,
}

impl<T: Sized> Garbage<T> {
    pub fn new(value: T) -> Self {
        let layout = Layout::new::<T>();
        let data;
        unsafe {
            data = alloc(layout) as *mut T;
            *data = value;
        }
        Garbage {
            data,
            refcount: Box::into_raw(Box::new(1)),
        }
    }
}

impl<T> Drop for Garbage<T> {
    fn drop(&mut self) {
        unsafe {
            *self.refcount -= 1;
            if *self.refcount == 0 {
                let layout = Layout::new::<T>();
                dealloc(self.data as *mut u8, layout);
            }
        }
    }
}

impl<T> Deref for Garbage<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.data.as_ref().unwrap() }
    }
}

impl<T> DerefMut for Garbage<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut().unwrap() }
    }
}

impl<T> Clone for Garbage<T> {
    fn clone(&self) -> Self {
        unsafe {
            *self.refcount += 1;
        }
        Garbage {
            data: self.data,
            refcount: self.refcount,
        }
    }
}
