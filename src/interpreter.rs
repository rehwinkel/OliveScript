use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::mem::transmute;

#[derive(Debug, Clone)]
enum Object {
    Str(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Bendy(HashMap<String, Object>),
    List(Vec<Object>),
    Func(Vec<String>, Vec<u8>),
    None,
}

impl Object {
    fn equals(&self, other: &Object) -> bool {
        match self {
            Object::None => {
                if let Object::None = other {
                    true
                } else {
                    false
                }
            }
            Object::Bool(b) => {
                if let Object::Bool(b2) = other {
                    b == b2
                } else {
                    false
                }
            }
            Object::Str(s) => {
                if let Object::Str(s2) = other {
                    s == s2
                } else {
                    false
                }
            }
            Object::Int(i) => match other {
                Object::Int(j) => i == j,
                Object::Float(j) => *i as f64 == *j,
                _ => false,
            },
            Object::Float(i) => match other {
                Object::Int(j) => *i == *j as f64,
                Object::Float(j) => i == j,
                _ => false,
            },
            Object::List(l) => {
                if let Object::List(l2) = other {
                    if l.len() == l2.len() {
                        for i in 0..l.len() {
                            if !l[i].equals(&l2[i]) {
                                return false;
                            }
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Object::Bendy(l) => {
                if let Object::Bendy(l2) = other {
                    if l.len() == l2.len() {
                        let lk: Vec<&String> = l.keys().collect();
                        let l2k: Vec<&String> = l2.keys().collect();
                        for i in 0..l.len() {
                            if lk[i] != l2k[i] || !l[lk[i]].equals(&l2[l2k[i]]) {
                                return false;
                            }
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Object::Func(args, codes) => {
                if let Object::Func(args2, codes2) = other {
                    if args.len() == args.len() && codes.len() == codes2.len() {
                        for i in 0..args.len() {
                            if args[i] != args2[i] {
                                return false;
                            }
                        }
                        for i in 0..codes.len() {
                            if codes[i] != codes2[i] {
                                return false;
                            }
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    TypeError,
    KeyError(String),
}

impl Error for RuntimeError {}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            RuntimeError::TypeError => write!(f, "type error: {:?}", self),
            RuntimeError::KeyError(key) => write!(f, "key error: {}", key),
        }
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

fn pop_int(stack: &mut Vec<Object>) -> Result<i64, RuntimeError> {
    if let Object::Int(i) = stack.pop().unwrap() {
        Ok(i)
    } else {
        Err(RuntimeError::TypeError)
    }
}

fn pop_string(stack: &mut Vec<Object>) -> Result<String, RuntimeError> {
    if let Object::Str(i) = stack.pop().unwrap() {
        Ok(i)
    } else {
        Err(RuntimeError::TypeError)
    }
}

fn pop_stringable(stack: &mut Vec<Object>) -> Result<String, RuntimeError> {
    Ok(match stack.pop().unwrap() {
        Object::Str(s) => s.clone(),
        Object::Bool(b) => b.to_string(),
        Object::Int(i) => i.to_string(),
        Object::Float(f) => format!("{:.5}", f),
        Object::None => String::from("none"),
        Object::List(v) => format!("{:?}", v),
        Object::Bendy(m) => format!("{:?}", m),
        Object::Func(args, _) => format!("func({:?})", args),
    })
}

fn pop_boolable(stack: &mut Vec<Object>) -> Result<bool, RuntimeError> {
    Ok(match stack.pop().unwrap() {
        Object::Str(s) => s.len() > 0,
        Object::Bool(b) => b,
        Object::Int(i) => i != 0,
        Object::Float(f) => f != 0.0,
        Object::None => false,
        Object::List(v) => v.len() > 0,
        Object::Bendy(m) => m.len() > 0,
        Object::Func(_, _) => true,
    })
}

pub fn run(codes: &Vec<u8>, constants: &Vec<String>) -> Result<(), RuntimeError> {
    let mut stack: Vec<Object> = Vec::new();
    let mut locvars: HashMap<String, Object> = HashMap::new();

    let mut ip: usize = 0;
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
            5 => {
                let s = std::mem::size_of::<usize>();
                let arglen = bytes_to_usize(codes[ip..ip + s].try_into().expect(""));
                ip += s;
                let args: Vec<String> = (0..arglen)
                    .map(|_| {
                        let val = bytes_to_usize(codes[ip..ip + s].try_into().expect(""));
                        ip += s;
                        constants[val].clone()
                    })
                    .collect();
                let codelen = bytes_to_usize(codes[ip..ip + s].try_into().expect(""));
                ip += s;
                let codes: Vec<u8> = codes[ip..ip + codelen].to_vec();
                ip += codelen;
                stack.push(Object::Func(args, codes));
            }
            6 => {
                let s = std::mem::size_of::<usize>();
                let val = bytes_to_usize(codes[ip..ip + s].try_into().expect(""));
                ip += s;
                let name = constants[val].clone();
                locvars.insert(name, stack.pop().unwrap());
            }
            7 => {
                let s = std::mem::size_of::<usize>();
                let val = bytes_to_usize(codes[ip..ip + s].try_into().expect(""));
                ip += s;
                let name = constants[val].clone();
                if let Some(val) = locvars.get(&name) {
                    stack.push(val.clone()); //TODO clone ok?
                } else {
                    return Err(RuntimeError::KeyError(name));
                }
            }
            10 => {
                let s = std::mem::size_of::<usize>();
                let val = bytes_to_usize(codes[ip..ip + s].try_into().expect(""));
                ip += s;
                if !pop_boolable(&mut stack)? {
                    ip = val;
                }
            }
            11 => {
                let s = std::mem::size_of::<usize>();
                let val = bytes_to_usize(codes[ip..ip + s].try_into().expect(""));
                ip = val;
            }
            12 => stack.push(Object::None),
            13 => stack.push(Object::Bendy(HashMap::new())),
            14 => stack.push(Object::List(Vec::new())),
            15 => {
                println!("{:?}", stack.pop().unwrap()); //TODO
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
                    _ => return Err(RuntimeError::TypeError),
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
                        _ => return Err(RuntimeError::TypeError),
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
                        _ => return Err(RuntimeError::TypeError),
                    },
                    _ => return Err(RuntimeError::TypeError),
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
                let v = !pop_boolable(&mut stack)?;
                stack.push(Object::Bool(v));
            }
            30 => {
                let mut lhs = pop_stringable(&mut stack)?;
                let rhs = pop_stringable(&mut stack)?;
                lhs.push_str(rhs.as_str());
                stack.push(Object::Str(lhs));
            }
            31 => {
                if let Object::Bendy(ref mut map) = stack.pop().unwrap() {
                    let key = pop_string(&mut stack)?;
                    let val = stack.pop().unwrap();
                    map.insert(key, val);
                }
            }
            32 => {
                if let Object::Bendy(map) = stack.pop().unwrap() {
                    let key = pop_string(&mut stack)?;
                    if let Some(val) = map.get(&key) {
                        stack.push(val.clone()); //TODO Clone ok?
                    } else {
                        return Err(RuntimeError::KeyError(key));
                    }
                }
            }
            34 => {
                let a = pop_boolable(&mut stack)?;
                let b = pop_boolable(&mut stack)?;
                stack.push(Object::Bool(a && b));
            }
            35 => {
                let a = pop_boolable(&mut stack)?;
                let b = pop_boolable(&mut stack)?;
                stack.push(Object::Bool(a || b));
            }
            36 => {
                let lhs = stack.pop().unwrap();
                let rhs = stack.pop().unwrap();
                stack.push(Object::Bool(lhs.equals(&rhs)));
            }
            37 => {
                let lhs = stack.pop().unwrap();
                let rhs = stack.pop().unwrap();
                stack.push(Object::Bool(!lhs.equals(&rhs)));
            }
            38 | 39 | 40 | 41 => {
                let lhs = stack.pop().unwrap();
                let rhs = stack.pop().unwrap();
                match lhs {
                    Object::Int(x) => match rhs {
                        Object::Int(y) => {
                            stack.push(match code {
                                38 => Object::Bool(x < y),
                                39 => Object::Bool(x <= y),
                                40 => Object::Bool(x > y),
                                41 => Object::Bool(x >= y),
                                _ => panic!(),
                            });
                        }
                        Object::Float(y) => {
                            stack.push(match code {
                                38 => Object::Bool((x as f64) < y),
                                39 => Object::Bool(x as f64 <= y),
                                40 => Object::Bool(x as f64 > y),
                                41 => Object::Bool(x as f64 >= y),
                                _ => panic!(),
                            });
                        }
                        _ => return Err(RuntimeError::TypeError),
                    },
                    Object::Float(x) => match rhs {
                        Object::Int(y) => {
                            stack.push(match code {
                                38 => Object::Bool(x < y as f64),
                                39 => Object::Bool(x <= y as f64),
                                40 => Object::Bool(x > y as f64),
                                41 => Object::Bool(x >= y as f64),
                                _ => panic!(),
                            });
                        }
                        Object::Float(y) => {
                            stack.push(match code {
                                38 => Object::Bool(x < y),
                                39 => Object::Bool(x <= y),
                                40 => Object::Bool(x > y),
                                41 => Object::Bool(x >= y),
                                _ => panic!(),
                            });
                        }
                        _ => return Err(RuntimeError::TypeError),
                    },
                    _ => return Err(RuntimeError::TypeError),
                }
            }
            _ => panic!("unexpected code: {:?}", code),
        }
    }
    /*
    Code::Call => vec![33],
    */
}
