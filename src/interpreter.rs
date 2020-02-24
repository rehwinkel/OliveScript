use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::mem::transmute;
use std::collections::HashMap;

#[derive(Debug)]
enum Object {
    Str(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Bendy(HashMap<String, Object>),
    List(Vec<Object>),
    None,
}

#[derive(Debug)]
pub enum RuntimeError {
    TypeError,
}

impl Error for RuntimeError {}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "type error: {:?}", self)
    }
}

fn bytes_to_i64(val: [u8; 8]) -> i64 {
    unsafe { transmute::<[u8; 8], i64>(val) }
}

fn bytes_to_f64(val: [u8; 8]) -> f64 {
    unsafe { transmute::<[u8; 8], f64>(val) }
}

fn bytes_to_usize(val: [u8; std::mem::size_of::<usize>()]) -> usize {
    unsafe { transmute::<[u8; std::mem::size_of::<usize>()], usize>(val) }
}

impl Object {
    fn to_string(&self) -> String {
        match self {
            Object::Str(s) => s.clone(),
            Object::Bool(b) => b.to_string(),
            Object::Int(i) => i.to_string(),
            Object::Float(f) => format!("{:.5}", f),
            Object::None => String::from("none"),
            Object::List(v) => format!("{:?}", v),
            Object::Bendy(m) => format!("{:?}", m),
        }
    }
}

fn pop_int(stack: &mut Vec<Object>) -> Result<i64, RuntimeError> {
    if let Object::Int(i) = stack.pop().unwrap() {
        Ok(i)
    } else {
        Err(RuntimeError::TypeError)
    }
}

fn pop_float(stack: &mut Vec<Object>) -> Result<f64, RuntimeError> {
    if let Object::Float(f) = stack.pop().unwrap() {
        Ok(f)
    } else {
        Err(RuntimeError::TypeError)
    }
}

fn pop_bool(stack: &mut Vec<Object>) -> Result<bool, RuntimeError> {
    if let Object::Bool(b) = stack.pop().unwrap() {
        Ok(b)
    } else {
        Err(RuntimeError::TypeError)
    }
}

fn pop_string(stack: &mut Vec<Object>) -> Result<String, RuntimeError> {
    if let Object::Str(s) = stack.pop().unwrap() {
        Ok(s)
    } else {
        Err(RuntimeError::TypeError)
    }
}

pub fn run(codes: &Vec<u8>, constants: &Vec<String>) -> Result<(), RuntimeError> {
    let mut stack: Vec<Object> = Vec::new();

    let mut ip = 0;
    loop {
        let code = codes[ip];
        ip += 1;
        match code {
            1 => {
                let s = std::mem::size_of::<usize>();
                let val = bytes_to_usize(codes[ip..ip + s].try_into().expect(""));
                ip += s;
                stack.push(Object::Str(constants[val].clone()));
            }
            2 => {
                let val = codes[ip] > 0;
                ip += 1;
                stack.push(Object::Bool(val));
            }
            3 => {
                let val = bytes_to_f64(codes[ip..ip + 8].try_into().expect(""));
                ip += 8;
                stack.push(Object::Float(val));
            }
            4 => {
                let val = bytes_to_i64(codes[ip..ip + 8].try_into().expect(""));
                ip += 8;
                stack.push(Object::Int(val));
            }
            12 => stack.push(Object::None),
            13 => stack.push(Object::Bendy(HashMap::new())),
            14 => stack.push(Object::List(Vec::new())),
            15 => {
                println!("{:?}", stack.pop().unwrap());
            }
            16 => {
                let rhs = stack.pop().unwrap();
                match rhs {
                    Object::Int(x) => {
                        stack.push(Object::Int(-x));
                    }
                    Object::Float(x) => {
                        stack.push(Object::Float(-x));
                    }
                    _ => panic!("type error"),
                }
            }
            17 | 18 | 19 | 20 | 21 | 22 => {
                let lhs = stack.pop().unwrap();
                let rhs = stack.pop().unwrap();
                match lhs {
                    Object::Int(x) => match rhs {
                        Object::Int(y) => {
                            stack.push(match code {
                                17 => Object::Int(x + y),
                                18 => Object::Int(x - y),
                                19 => Object::Int(x * y),
                                20 => Object::Int(x / y),
                                21 => Object::Float(x as f64 / y as f64),
                                22 => Object::Int(x % y),
                                _ => panic!(),
                            });
                        }
                        Object::Float(y) => {
                            stack.push(match code {
                                17 => Object::Float(x as f64 + y),
                                18 => Object::Float(x as f64 - y),
                                19 => Object::Float(x as f64 * y),
                                20 => Object::Float((x / y as i64) as f64),
                                21 => Object::Float(x as f64 / y),
                                22 => Object::Float(x as f64 % y),
                                _ => panic!(),
                            });
                        }
                        _ => panic!("type error"),
                    },
                    Object::Float(x) => match rhs {
                        Object::Int(y) => {
                            stack.push(match code {
                                17 => Object::Float(x + y as f64),
                                18 => Object::Float(x - y as f64),
                                19 => Object::Float(x * y as f64),
                                20 => Object::Float((x as i64 / y) as f64),
                                21 => Object::Float(x / y as f64),
                                22 => Object::Float(x % y as f64),
                                _ => panic!(),
                            });
                        }
                        Object::Float(y) => {
                            stack.push(match code {
                                17 => Object::Float(x + y),
                                18 => Object::Float(x - y),
                                19 => Object::Float(x * y),
                                20 => Object::Float((x as i64 / y as i64) as f64),
                                21 => Object::Float(x / y),
                                22 => Object::Float(x % y),
                                _ => panic!(),
                            });
                        }
                        _ => panic!("type error"),
                    },
                    _ => panic!("type error"),
                }
            }
            23 | 24 | 25 | 26 | 27 | 28 => {
                let x = pop_int(&mut stack)?;
                let y = pop_int(&mut stack)?;
                stack.push(match code {
                    23 => Object::Int(x << y),
                    24 => Object::Int(x >> y),
                    26 => Object::Int(x & y),
                    27 => Object::Int(x | y),
                    28 => Object::Int(x ^ y),
                    _ => panic!(),
                });
            }
            29 => {
                let v = !pop_bool(&mut stack)?;
                stack.push(Object::Bool(v));
            }
            30 => {
                let lhs = stack.pop().unwrap();
                let rhs = stack.pop().unwrap();
                let mut s1 = lhs.to_string();
                s1.push_str(rhs.to_string().as_str());
                stack.push(Object::Str(s1));
            }
            34 => {
                println!("stack: {:?}", stack);
                let a = pop_bool(&mut stack)?;
                let b = pop_bool(&mut stack)?;
                stack.push(Object::Bool(a && b));
            }
            35 => {
                let a = pop_bool(&mut stack)?;
                let b = pop_bool(&mut stack)?;
                stack.push(Object::Bool(a || b));
            }
            _ => println!("Error: {:?}", code),
        }
    }
    /*
    Code::NewFun(args, codes) => {
        let arglen = Code::usize_to_bytes(args.len());
        let argindices: Vec<u8> = args
            .iter()
            .map(|arg| Code::usize_to_bytes(constants.insert_full(arg.clone()).0))
            .flat_map(|bytes| bytes)
            .collect();
        let codeslen = Code::usize_to_bytes(codes.len());
        [vec![5], arglen, argindices, codeslen, codes.clone()].concat()
    }
    Code::Store(name) => {
        let index = constants.insert_full(name.clone()).0;
        [vec![6], Code::usize_to_bytes(index)].concat()
    }
    Code::Load(name) => {
        let index = constants.insert_full(name.clone()).0;
        [vec![7], Code::usize_to_bytes(index)].concat()
    }
    Code::TStore(i) => {
        let index = constants.insert_full(format!("<{}>", i)).0;
        [vec![6], Code::usize_to_bytes(index)].concat()
    }
    Code::TLoad(i) => {
        let index = constants.insert_full(format!("<{}>", i)).0;
        [vec![7], Code::usize_to_bytes(index)].concat()
    }
    Code::JumpNot(p) => [vec![10], Code::usize_to_bytes(*p)].concat(),
    Code::Goto(p) | Code::Break(p) => [vec![11], Code::usize_to_bytes(*p)].concat(),
    Code::Put => vec![31],
    Code::Get => vec![32],
    Code::Call => vec![33],
    Code::Equals => vec![36],
    Code::NotEquals => vec![37],
    Code::LessThan => vec![38],
    Code::LessEquals => vec![39],
    Code::GreaterThan => vec![40],
    Code::GreaterEquals => vec![41],
    */
}
