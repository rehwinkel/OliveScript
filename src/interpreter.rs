use data::{NativeFunc, Object, RuntimeError, Scope};
use libloading::{RcLibrary, RcSymbol};
use serde_json;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
use std::ffi::CString;
use std::fs;
use std::mem::transmute;
use std::path::PathBuf;
use std::rc::Rc;

pub mod data {
    use libloading::RcSymbol;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fmt::{Display, Formatter, Result as FmtResult};
    use std::rc::Rc;

    pub type NativeFunc =
        fn(Box<Vec<Rc<RefCell<Object>>>>) -> Result<Rc<RefCell<Object>>, RuntimeError>;

    #[derive(Debug)]
    pub enum Object {
        Str(String),
        Bool(bool),
        Int(i64),
        Float(f64),
        Bendy(HashMap<String, Rc<RefCell<Object>>>),
        List(Vec<Rc<RefCell<Object>>>),
        Func(Vec<String>, Vec<u8>),
        Frame(usize, Vec<u8>),
        None,
        BuiltinFunc(usize, NativeFunc),
        NativeFunc(usize, RcSymbol<NativeFunc>),
    }

    impl Object {
        pub fn equals(&self, other: Rc<RefCell<Object>>) -> bool {
            match self {
                Object::None => {
                    if let Object::None = *other.borrow() {
                        true
                    } else {
                        false
                    }
                }
                Object::Bool(b) => {
                    if let Object::Bool(b2) = *other.borrow() {
                        *b == b2
                    } else {
                        false
                    }
                }
                Object::Str(s) => {
                    if let Object::Str(s2) = &*other.borrow() {
                        *s == *s2
                    } else {
                        false
                    }
                }
                Object::Int(i) => match *other.borrow() {
                    Object::Int(j) => *i == j,
                    Object::Float(j) => *i as f64 == j,
                    _ => false,
                },
                Object::Float(i) => match *other.borrow() {
                    Object::Int(j) => *i == j as f64,
                    Object::Float(j) => *i == j,
                    _ => false,
                },
                Object::List(l) => {
                    if let Object::List(l2) = &*other.borrow() {
                        if l.len() == l2.len() {
                            for i in 0..l.len() {
                                if !l[i].borrow().equals(Rc::clone(&l2[i])) {
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
                    if let Object::Bendy(l2) = &*other.borrow() {
                        if l.len() == l2.len() {
                            let lk: Vec<&String> = l.keys().collect();
                            let l2k: Vec<&String> = l2.keys().collect();
                            for i in 0..l.len() {
                                if lk[i] != l2k[i]
                                    || !l[lk[i]].borrow().equals(Rc::clone(&l2[l2k[i]]))
                                {
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
                    if let Object::Func(args2, codes2) = &*other.borrow() {
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

                Object::BuiltinFunc(arglen, fp) => {
                    if let Object::BuiltinFunc(arglen2, fp2) = &*other.borrow() {
                        arglen == arglen2 && fp == fp2
                    } else {
                        false
                    }
                }

                Object::NativeFunc(arglen, _fp) => {
                    if let Object::NativeFunc(arglen2, _fp2) = &*other.borrow() {
                        arglen == arglen2 // TODO && *fp.get() == *fp2.get()
                    } else {
                        false
                    }
                }
                Object::Frame(_, _) => panic!("forbidden type"),
            }
        }

        pub fn to_string(&self) -> String {
            match self {
                Object::Str(s) => s.clone(),
                Object::Bool(b) => b.to_string(),
                Object::Int(i) => i.to_string(),
                Object::Float(f) => format!("{:.5}", f),
                Object::None => String::from("none"),
                Object::List(v) => {
                    let mut string = String::from("[");
                    if v.len() > 0 {
                        string += format!("{}", (*v[0].borrow()).to_string()).as_str();
                        for value in &v[1..] {
                            string += format!(", {}", (*value.borrow()).to_string()).as_str();
                        }
                    }
                    string + "]"
                }
                Object::Bendy(m) => {
                    let mut string = String::from("{");
                    if m.len() > 0 {
                        let keys: Vec<&String> = m.keys().collect();
                        string +=
                            format!("{}: {}", keys[0], (*m[keys[0]].borrow()).to_string()).as_str();
                        for key in &keys[1..] {
                            let value = &m[*key];
                            string +=
                                format!(", {}: {}", key, (*value.borrow()).to_string()).as_str();
                        }
                    }
                    string + "}"
                }
                Object::Func(args, _) => {
                    let mut string = String::from("func(");
                    if args.len() > 0 {
                        string += args[0].as_str();
                        for value in &args[1..] {
                            string += ", ";
                            string += value.as_str();
                        }
                    }
                    string + ")"
                }
                Object::BuiltinFunc(arglen, _) => format!("builtin({})", arglen),
                Object::NativeFunc(arglen, _) => format!("native({})", arglen),
                Object::Frame(_, _) => panic!("forbidden type"),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum RuntimeError {
        TypeError,
        KeyError(String),
        ImportError(String),
    }

    impl Error for RuntimeError {}

    impl Display for RuntimeError {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            match self {
                RuntimeError::TypeError => write!(f, "type error: {:?}", self),
                RuntimeError::KeyError(key) => write!(f, "key error: {}", key),
                RuntimeError::ImportError(err) => write!(f, "import error: {}", err),
            }
        }
    }

    #[derive(Debug)]
    pub struct Scope {
        pub parent: Option<Box<Scope>>,
        locvars: HashMap<String, Rc<RefCell<Object>>>,
    }

    impl Scope {
        pub fn new() -> Self {
            Scope {
                parent: None,
                locvars: HashMap::new(),
            }
        }
        pub fn from(parent: Scope) -> Self {
            Scope {
                parent: Some(Box::new(parent)),
                locvars: HashMap::new(),
            }
        }

        pub fn put(&mut self, name: String, val: Rc<RefCell<Object>>) {
            if self.has(name.clone()) {
                self.locvars.insert(name, val); // write in this
            } else {
                if let Some(ref mut parent_scope) = &mut self.parent {
                    if parent_scope.has(name.clone()) {
                        parent_scope.put(name, val); // write in parent
                    } else {
                        self.locvars.insert(name, val); // write in this
                    }
                } else {
                    self.locvars.insert(name, val); // write in this
                }
            }
        }

        pub fn get(&self, name: String) -> Result<Rc<RefCell<Object>>, RuntimeError> {
            if let Some(val) = self.locvars.get(&name) {
                Ok(Rc::clone(val))
            } else {
                if let Some(parent_scope) = &self.parent {
                    Ok(parent_scope.get(name)?)
                } else {
                    Err(RuntimeError::KeyError(name))
                }
            }
        }

        fn has(&self, name: String) -> bool {
            let keys: Vec<&String> = self.locvars.keys().collect();
            if keys.contains(&&name) {
                true
            } else {
                if let Some(parent_scope) = &self.parent {
                    parent_scope.has(name)
                } else {
                    false
                }
            }
        }
    }
}

pub mod modules {
    use super::bytes_to_u32;
    use crate::codegen;
    use crate::parser::parser;
    use std::fs;
    use std::path::PathBuf;

    fn read_from_bytes(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let mut current = 4;
        let codelen = bytes_to_u32(&bytes, &mut current);
        Ok((&bytes[current..current + codelen as usize]).to_vec())
    }
    fn read_from_source(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
        let contents: String =
            String::from(std::str::from_utf8(&bytes).map_err(|err| format!("{}", err))?);
        let block = parser::parse(&contents).map_err(|err| format!("{}", err))?;
        let codes = codegen::to_bytes(codegen::generate(block).map_err(|err| format!("{}", err))?);
        Ok(codes)
    }
    pub fn read(path: PathBuf) -> Result<Vec<u8>, String> {
        let bytes = fs::read(path).map_err(|err| format!("{}", err))?;
        Ok(if &bytes[..4] == [0xCE, 0xDA, 0xFA, 0xBA] {
            read_from_bytes(bytes)?
        } else {
            read_from_source(bytes)?
        })
    }
}

fn bytes_to_i64(bytes: &Vec<u8>, ip: &mut usize) -> i64 {
    let val = bytes[*ip..*ip + 8].try_into().expect("");
    *ip += 8;
    unsafe { transmute::<[u8; 8], i64>(val) }.to_le()
}

fn bytes_to_f64(bytes: &Vec<u8>, ip: &mut usize) -> f64 {
    let val = bytes[*ip..*ip + 8].try_into().expect("");
    *ip += 8;
    unsafe { transmute::<[u8; 8], f64>(val) }
}

pub fn bytes_to_u32(bytes: &Vec<u8>, ip: &mut usize) -> u32 {
    let val = bytes[*ip..*ip + 4].try_into().expect("");
    *ip += 4;
    unsafe { transmute::<[u8; 4], u32>(val) }.to_le()
}

pub fn bytes_to_u16(bytes: &Vec<u8>, ip: &mut usize) -> u16 {
    let val = bytes[*ip..*ip + 2].try_into().expect("");
    *ip += 2;
    unsafe { transmute::<[u8; 2], u16>(val) }.to_le()
}

pub fn bytes_to_string(bytes: &Vec<u8>, ip: &mut usize) -> String {
    let len = bytes_to_u16(bytes, ip);
    let s = String::from_utf8((&bytes[*ip..*ip + len as usize]).to_vec()).expect("UTF8 error");
    *ip += len as usize;
    s
}

fn pop_int(stack: &mut Vec<Rc<RefCell<Object>>>) -> Result<i64, RuntimeError> {
    if let Object::Int(i) = *stack.pop().unwrap().borrow() {
        Ok(i)
    } else {
        Err(RuntimeError::TypeError)
    }
}

fn pop_string(stack: &mut Vec<Rc<RefCell<Object>>>) -> Result<String, RuntimeError> {
    if let Object::Str(i) = &*stack.pop().unwrap().borrow() {
        Ok(i.clone())
    } else {
        Err(RuntimeError::TypeError)
    }
}

fn pop_stringable(stack: &mut Vec<Rc<RefCell<Object>>>) -> Result<String, RuntimeError> {
    Ok((&*stack.pop().unwrap().borrow()).to_string())
}

fn pop_boolable(stack: &mut Vec<Rc<RefCell<Object>>>) -> Result<bool, RuntimeError> {
    Ok(match &*stack.pop().unwrap().borrow() {
        Object::Str(s) => s.len() > 0,
        Object::Bool(b) => *b,
        Object::Int(i) => *i != 0,
        Object::Float(f) => *f != 0.0,
        Object::None => false,
        Object::List(v) => v.len() > 0,
        Object::Bendy(m) => m.len() > 0,
        Object::Func(_, _) => true,
        Object::BuiltinFunc(_, _) => true,
        Object::NativeFunc(_, _) => true,
        Object::Frame(_, _) => panic!("forbidden type"),
    })
}

fn n_print(args: Box<Vec<Rc<RefCell<Object>>>>) -> Result<Rc<RefCell<Object>>, RuntimeError> {
    println!("{}", args[0].borrow().to_string());
    Ok(Rc::new(RefCell::new(Object::None)))
}

fn n_list_len(args: Box<Vec<Rc<RefCell<Object>>>>) -> Result<Rc<RefCell<Object>>, RuntimeError> {
    Ok(Rc::new(RefCell::new(Object::Int(
        match &*args[0].borrow() {
            Object::List(v) => v.len(),
            Object::Bendy(m) => m.len(),
            Object::Str(s) => s.len(),
            _ => return Err(RuntimeError::TypeError),
        } as i64,
    ))))
}

fn n_import(args: Box<Vec<Rc<RefCell<Object>>>>) -> Result<Rc<RefCell<Object>>, RuntimeError> {
    if let Object::Str(name) = &*args[0].borrow() {
        let current_dir = env::current_dir().expect("no current dir");

        let olv_path: PathBuf = current_dir.join(format!("{}.olv", name));
        let olvc_path: PathBuf = current_dir.join(format!("{}.olvc", name));
        let olvn_path: PathBuf = current_dir.join(format!("{}.olvn", name));

        for file in fs::read_dir(&current_dir).expect("no current dir") {
            let f: PathBuf = file.expect("").path();
            if f == olv_path || f == olvc_path {
                let new_current_dir = f.parent().unwrap().join(".");
                env::set_current_dir(new_current_dir).expect("couldn't set current dir");
                let codes = modules::read(f).map_err(|e| RuntimeError::ImportError(e))?;
                let result = run(codes)?;
                env::set_current_dir(current_dir).expect("couldn't set current dir");
                return Ok(result);
            } else if f == olvn_path {
                let contents = fs::read_to_string(f)
                    .map_err(|err| RuntimeError::ImportError(format!("{}", err)))?;
                let parsed: Value = serde_json::from_str(contents.as_str())
                    .map_err(|err| RuntimeError::ImportError(format!("{}", err)))?;
                return if let Some(libname) = parsed["library"].as_str() {
                    let lib: RcLibrary = RcLibrary::new(current_dir.join(libname))
                        .map_err(|err| RuntimeError::ImportError(format!("{}", err)))?;

                    let format_error = RuntimeError::ImportError(String::from("json format error"));
                    let mut native_funcs = HashMap::new();
                    let funcs = parsed["functions"].as_array().ok_or(format_error.clone())?;
                    for fun in funcs {
                        let native = fun["native"].as_str().ok_or(format_error.clone())?;
                        let name = String::from(fun["name"].as_str().ok_or(format_error.clone())?);
                        let args = fun["args"].as_u64().ok_or(format_error.clone())? as usize;
                        let native_c_str = CString::new(native).expect("nul error");
                        let func: RcSymbol<NativeFunc> = unsafe { lib.get(native_c_str).unwrap() };
                        native_funcs
                            .insert(name, Rc::new(RefCell::new(Object::NativeFunc(args, func))));
                    }
                    Ok(Rc::new(RefCell::new(Object::Bendy(native_funcs))))
                } else {
                    Err(RuntimeError::ImportError(String::from(
                        "no string library field in json",
                    )))
                };
            }
        }
        Err(RuntimeError::ImportError(format!(
            "module not found: {}",
            name
        )))
    } else {
        Err(RuntimeError::TypeError)
    }
}

fn insert_builtin_funcs(scope: &mut Scope) {
    scope.put(
        "print".to_string(),
        Rc::new(RefCell::new(Object::BuiltinFunc(1, n_print))),
    );
    scope.put(
        "len".to_string(),
        Rc::new(RefCell::new(Object::BuiltinFunc(1, n_list_len))),
    );
    scope.put(
        "import".to_string(),
        Rc::new(RefCell::new(Object::BuiltinFunc(1, n_import))),
    );
}

pub fn run(in_codes: Vec<u8>) -> Result<Rc<RefCell<Object>>, RuntimeError> {
    let mut stack: Vec<Rc<RefCell<Object>>> = Vec::new();
    let mut scope = Scope::new();

    macro_rules! push {
        ($e: expr) => {
            stack.push(Rc::new(RefCell::new($e)))
        };
    }

    insert_builtin_funcs(&mut scope);
    push!(Object::Func(Vec::new(), in_codes));
    let mut codes = vec![33_u8];

    let mut ip: usize = 0;
    while ip < codes.len() {
        let code = codes[ip];
        ip += 1;
        match code {
            1 => {
                push!(Object::Str(bytes_to_string(&codes, &mut ip)));
            }
            2 => {
                let val = codes[ip] > 0;
                ip += 1;
                push!(Object::Bool(val));
            }
            3 => {
                let val = bytes_to_f64(&codes, &mut ip);
                push!(Object::Float(val));
            }
            4 => {
                let val = bytes_to_i64(&codes, &mut ip);
                push!(Object::Int(val));
            }
            5 => {
                let arglen = bytes_to_u16(&codes, &mut ip);
                let args: Vec<String> = (0..arglen)
                    .map(|_| bytes_to_string(&codes, &mut ip))
                    .collect();
                let codelen = bytes_to_u32(&codes, &mut ip);
                let codes: Vec<u8> = codes[ip..ip + codelen as usize].to_vec();
                ip += codelen as usize;
                push!(Object::Func(args, codes));
            }
            6 => {
                scope.put(bytes_to_string(&codes, &mut ip), stack.pop().unwrap());
            }
            7 => {
                stack.push(Rc::clone(&scope.get(bytes_to_string(&codes, &mut ip))?));
            }
            10 => {
                let val = bytes_to_u32(&codes, &mut ip);
                if !pop_boolable(&mut stack)? {
                    ip = val as usize;
                }
            }
            11 => {
                let val = bytes_to_u32(&codes, &mut ip);
                ip = val as usize;
            }
            12 => push!(Object::None),
            13 => push!(Object::Bendy(HashMap::new())),
            14 => push!(Object::List(Vec::new())),
            15 => {
                let return_val = stack.pop().unwrap();
                if let Object::Frame(new_ip, new_codes) = &*stack.pop().unwrap().borrow() {
                    ip = *new_ip;
                    codes = new_codes.clone();
                    scope = *scope.parent.unwrap();
                    stack.push(return_val);
                } else {
                    panic!("broken call stack");
                }
            }
            16 => {
                let rhs = stack.pop().unwrap();
                match *rhs.borrow() {
                    Object::Int(x) => {
                        push!(Object::Int(-x));
                    }
                    Object::Float(x) => {
                        push!(Object::Float(-x));
                    }
                    _ => return Err(RuntimeError::TypeError),
                };
            }
            17 | 18 | 19 | 20 | 21 | 22 => {
                let lhs = stack.pop().unwrap();
                let rhs = stack.pop().unwrap();
                match *lhs.borrow() {
                    Object::Int(x) => match *rhs.borrow() {
                        Object::Int(y) => {
                            push!(match code {
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
                            push!(match code {
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
                    Object::Float(x) => match *rhs.borrow() {
                        Object::Int(y) => {
                            push!(match code {
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
                            push!(match code {
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
                };
            }
            23 | 24 | 25 | 26 | 27 | 28 => {
                let x = pop_int(&mut stack)?;
                let y = pop_int(&mut stack)?;
                push!(match code {
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
                push!(Object::Bool(v));
            }
            30 => {
                let mut lhs = pop_stringable(&mut stack)?;
                let rhs = pop_stringable(&mut stack)?;
                lhs.push_str(rhs.as_str());
                push!(Object::Str(lhs));
            }
            31 => match *stack.pop().unwrap().borrow_mut() {
                Object::Bendy(ref mut map) => {
                    let key = pop_string(&mut stack)?;
                    let val = stack.pop().unwrap();
                    map.insert(key, val);
                }
                Object::List(ref mut vec) => {
                    let key = pop_int(&mut stack)?;
                    let val = stack.pop().unwrap();
                    if key >= 0 {
                        let i: usize = key as usize;
                        while i + 1 > vec.len() {
                            vec.push(Rc::new(RefCell::new(Object::None)));
                        }
                        vec[key as usize] = val;
                    } else {
                        return Err(RuntimeError::KeyError(key.to_string()));
                    }
                }
                _ => {
                    return Err(RuntimeError::TypeError);
                }
            },
            32 => match &*stack.pop().unwrap().borrow() {
                Object::Bendy(map) => {
                    let key = pop_string(&mut stack)?;
                    if let Some(val) = map.get(&key) {
                        stack.push(Rc::clone(val));
                    } else {
                        return Err(RuntimeError::KeyError(key));
                    }
                }
                Object::List(vec) => {
                    let key = pop_int(&mut stack)?;
                    if key >= 0 {
                        if let Some(val) = vec.get(key as usize) {
                            stack.push(Rc::clone(&val));
                        } else {
                            return Err(RuntimeError::KeyError(key.to_string()));
                        }
                    } else {
                        return Err(RuntimeError::KeyError(key.to_string()));
                    }
                }
                _ => {
                    return Err(RuntimeError::TypeError);
                }
            },
            33 => match &*stack.pop().unwrap().borrow() {
                Object::Func(f_args, f_codes) => {
                    scope = Scope::from(scope);
                    for arg in f_args {
                        let val = stack.pop().unwrap();
                        scope.put(arg.clone(), val)
                    }
                    push!(Object::Frame(ip, codes));
                    ip = 0;
                    codes = f_codes.clone();
                }
                Object::BuiltinFunc(arglen, func) => {
                    let mut args = Vec::with_capacity(*arglen);
                    for _ in 0..*arglen {
                        let val = stack.pop().unwrap();
                        args.push(val);
                    }
                    stack.push(func(Box::new(args))?);
                }
                Object::NativeFunc(arglen, func) => {
                    let mut args = Vec::with_capacity(*arglen);
                    for _ in 0..*arglen {
                        let val = stack.pop().unwrap();
                        args.push(val);
                    }
                    stack.push(func(Box::new(args))?);
                }
                _ => return Err(RuntimeError::TypeError),
            },
            34 => {
                let a = pop_boolable(&mut stack)?;
                let b = pop_boolable(&mut stack)?;
                push!(Object::Bool(a && b));
            }
            35 => {
                let a = pop_boolable(&mut stack)?;
                let b = pop_boolable(&mut stack)?;
                push!(Object::Bool(a || b));
            }
            36 => {
                let lhs = stack.pop().unwrap();
                let rhs = stack.pop().unwrap();
                push!(Object::Bool(lhs.borrow().equals(rhs)));
            }
            37 => {
                let lhs = stack.pop().unwrap();
                let rhs = stack.pop().unwrap();
                push!(Object::Bool(!lhs.borrow().equals(rhs)));
            }
            38 | 39 | 40 | 41 => {
                let lhs = stack.pop().unwrap();
                let rhs = stack.pop().unwrap();
                match *lhs.borrow() {
                    Object::Int(x) => match *rhs.borrow() {
                        Object::Int(y) => {
                            push!(match code {
                                38 => Object::Bool(x < y),
                                39 => Object::Bool(x <= y),
                                40 => Object::Bool(x > y),
                                41 => Object::Bool(x >= y),
                                _ => panic!(),
                            });
                        }
                        Object::Float(y) => {
                            push!(match code {
                                38 => Object::Bool((x as f64) < y),
                                39 => Object::Bool(x as f64 <= y),
                                40 => Object::Bool(x as f64 > y),
                                41 => Object::Bool(x as f64 >= y),
                                _ => panic!(),
                            });
                        }
                        _ => return Err(RuntimeError::TypeError),
                    },
                    Object::Float(x) => match *rhs.borrow() {
                        Object::Int(y) => {
                            push!(match code {
                                38 => Object::Bool(x < y as f64),
                                39 => Object::Bool(x <= y as f64),
                                40 => Object::Bool(x > y as f64),
                                41 => Object::Bool(x >= y as f64),
                                _ => panic!(),
                            });
                        }
                        Object::Float(y) => {
                            push!(match code {
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
                };
            }
            42 => {
                stack.pop();
            }
            _ => panic!("unexpected code: {:?}", code),
        }
    }
    Ok(stack.pop().unwrap())
}
