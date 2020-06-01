use super::codegen::Code;
use super::errors::{OliveError, OliveRuntimeError};
use mistake::Mistake::{self, Fail, Fine};
use std::collections::HashMap;

mod builtins;
mod data;
mod error;
pub mod garbage;
use data::Data;
use garbage::{Garbage, GarbageCollector};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Scope {
    variables: HashMap<String, Garbage<Data>>,
    parent: Option<Rc<RefCell<Scope>>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
            parent: None,
        }
    }

    fn from_parent(parent: Rc<RefCell<Scope>>) -> Self {
        Scope {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }

    fn has(&self, name: String) -> bool {
        let keys: Vec<&String> = self.variables.keys().collect();
        if keys.contains(&&name) {
            true
        } else {
            if let Some(parent_scope) = &self.parent {
                parent_scope.borrow().has(name)
            } else {
                false
            }
        }
    }

    fn load(&self, varname: &String) -> Option<Garbage<Data>> {
        if let Some(result) = self.variables.get(varname) {
            Some(result.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.borrow().load(varname)
            } else {
                None
            }
        }
    }

    fn store(&mut self, name: String, val: Garbage<Data>) {
        if self.has(name.clone()) {
            self.variables.insert(name, val); // write in this
        } else {
            if let Some(parent_scope) = &mut self.parent {
                if parent_scope.borrow().has(name.clone()) {
                    parent_scope.borrow_mut().store(name, val); // write in parent
                } else {
                    self.variables.insert(name, val); // write in this
                }
            } else {
                self.variables.insert(name, val); // write in this
            }
        }
    }
}

pub fn run(
    codes: &Vec<Code>,
    code_pos_table: &HashMap<usize, usize>,
    filename: &str,
    source: Option<&str>,
    gc: &mut GarbageCollector<Data>,
    scope: Rc<RefCell<Scope>>,
) -> Mistake<Garbage<Data>, OliveError> {
    let mut errors = Vec::new();
    let mut stack = Vec::new();

    let mut ip = 0;
    loop {
        let code = &codes[ip];
        match code {
            Code::PushFun(args, codes) => {
                let fun_obj = gc.alloc(Data::Function {
                    args: args.clone(),
                    codes: codes.clone(),
                });
                stack.push(fun_obj);
            }
            Code::Call => {
                let function = stack.pop().unwrap();
                match &*function {
                    Data::Function { args, codes } => {
                        let new_scope = Rc::new(RefCell::new(Scope::from_parent(scope.clone())));
                        for (i, arg) in args.iter().rev().enumerate() {
                            if let Some(value) = stack.pop() {
                                new_scope.borrow_mut().store(arg.clone(), value);
                            } else {
                                println!("{}, {:?}", ip, code_pos_table);
                                errors.push(error::create_call_error(
                                    ip,
                                    code_pos_table,
                                    filename,
                                    source,
                                    i,
                                    args.len(),
                                ));
                                return Fail(errors);
                            }
                        }
                        let return_val = attempt!(
                            run(&codes, &code_pos_table, filename, source, gc, new_scope,),
                            errors
                        );
                        stack.push(return_val);
                    }
                    Data::Native { arg_count, closure } => {
                        let mut args = Vec::new();
                        for _ in 0..*arg_count {
                            let value = stack.pop().unwrap();
                            args.push(value);
                        }
                        let return_val = closure(args);
                        stack.push(gc.alloc(return_val));
                    }
                    t => {
                        println!("{}, {:?} {:?}", ip, code_pos_table, codes);
                        errors.push(error::create_type_error(
                            ip,
                            code_pos_table,
                            filename,
                            source,
                            vec!["function"],
                            t.get_name(),
                        ));
                        return Fail(errors);
                    }
                }
            }
            Code::PushByte(data) => {
                stack.push(gc.alloc(Data::Integer {
                    value: *data as i64,
                }));
            }
            Code::PushShort(data) => {
                stack.push(gc.alloc(Data::Integer {
                    value: *data as i64,
                }));
            }
            Code::PushInt(data) => {
                stack.push(gc.alloc(Data::Integer {
                    value: *data as i64,
                }));
            }
            Code::PushLong(data) => {
                stack.push(gc.alloc(Data::Integer {
                    value: *data as i64,
                }));
            }
            Code::PushDouble(data) => {
                stack.push(gc.alloc(Data::Float { value: *data }));
            }
            Code::PushBoolean(data) => {
                stack.push(gc.alloc(Data::Boolean { value: *data }));
            }
            Code::PushString(data) => {
                stack.push(gc.alloc(Data::String {
                    value: data.clone(),
                }));
            }
            Code::PushBendy => stack.push(gc.alloc(Data::Bendy {
                data: HashMap::new(),
            })),
            Code::PushList => stack.push(gc.alloc(Data::List { data: Vec::new() })),
            Code::PushNone => {
                stack.push(gc.alloc(Data::None));
            }
            Code::Return => {
                return Fine(stack.pop().unwrap(), errors);
            }
            Code::Dup => {
                let val = stack.last().unwrap().clone();
                stack.push(val);
            }
            Code::Pop => {
                stack.pop();
            }
            Code::Goto(offset) => {
                if *offset > 0 {
                    ip += *offset as usize;
                } else {
                    ip -= (-*offset) as usize;
                }
                continue;
            }
            Code::JumpNot(offset) => {
                if !stack.pop().unwrap().truthy() {
                    if *offset > 0 {
                        ip += *offset as usize;
                    } else {
                        ip -= (-*offset) as usize;
                    }
                    continue;
                }
            }
            Code::Jump(offset) => {
                if stack.pop().unwrap().truthy() {
                    if *offset > 0 {
                        ip += *offset as usize;
                    } else {
                        ip -= (-*offset) as usize;
                    }
                    continue;
                }
            }
            Code::Neg => match &*stack.pop().unwrap() {
                Data::Integer { value } => stack.push(gc.alloc(Data::Integer { value: -value })),
                Data::Float { value } => stack.push(gc.alloc(Data::Float { value: -value })),
                t => {
                    errors.push(error::create_type_error(
                        ip,
                        code_pos_table,
                        filename,
                        source,
                        vec!["integer", "float"],
                        t.get_name(),
                    ));
                    return Fail(errors);
                }
            },
            Code::BoolNot => {
                let value = !stack.pop().unwrap().truthy();
                stack.push(gc.alloc(Data::Boolean { value }))
            }
            Code::Add
            | Code::Sub
            | Code::Mod
            | Code::Mul
            | Code::FloatDiv
            | Code::IntDiv
            | Code::BitAnd
            | Code::BitOr
            | Code::BitXOr
            | Code::BitLsh
            | Code::BitRsh
            | Code::Concat
            | Code::Equals
            | Code::NotEquals
            | Code::LessThan
            | Code::LessEquals
            | Code::GreaterThan
            | Code::GreaterEquals => {
                let b = &*stack.pop().unwrap();
                let a = &*stack.pop().unwrap();
                stack.push(gc.alloc(attempt_res!(
                    a.operate(b, ip, code_pos_table, filename, source, code),
                    errors
                )))
            }
            Code::Put => {
                let value = stack.pop().unwrap();
                let index = stack.pop().unwrap();
                let mut object = stack.pop().unwrap();
                match &mut *object {
                    Data::List { data } => {
                        let int_index: i64 = attempt_res!(
                            index.as_integer(ip, code_pos_table, filename, source),
                            errors
                        );
                        while data.len() < int_index as usize + 1 {
                            data.push(gc.alloc(Data::None));
                        }
                        data[int_index as usize] = value;
                    }
                    Data::Bendy { data } => {
                        let str_index: &str = attempt_res!(
                            index.as_string(ip, code_pos_table, filename, source),
                            errors
                        );
                        data.insert(String::from(str_index), value);
                    }
                    _ => unimplemented!(),
                }
            }
            Code::Get => {
                let index = stack.pop().unwrap();
                let mut object = stack.pop().unwrap();
                match &mut *object {
                    Data::List { data } => {
                        let int_index: i64 = attempt_res!(
                            index.as_integer(ip, code_pos_table, filename, source),
                            errors
                        );
                        if let Some(v) = data.get(int_index as usize) {
                            stack.push(v.clone());
                        } else {
                            errors.push(error::create_runtime_error(
                                ip,
                                code_pos_table,
                                filename,
                                source,
                                OliveRuntimeError::IndexOutOfBounds,
                            ));
                            return Fail(errors);
                        }
                    }
                    Data::String { value } => {
                        let int_index: i64 = attempt_res!(
                            index.as_integer(ip, code_pos_table, filename, source),
                            errors
                        );
                        if let Some(v) = value.chars().skip(int_index as usize).next() {
                            stack.push(gc.alloc(Data::String {
                                value: v.to_string(),
                            }));
                        } else {
                            errors.push(error::create_runtime_error(
                                ip,
                                code_pos_table,
                                filename,
                                source,
                                OliveRuntimeError::IndexOutOfBounds,
                            ));
                            return Fail(errors);
                        }
                    }
                    Data::Bendy { data } => {
                        let str_index: &str = attempt_res!(
                            index.as_string(ip, code_pos_table, filename, source),
                            errors
                        );
                        if let Some(v) = data.get(str_index) {
                            stack.push(v.clone());
                        } else {
                            errors.push(error::create_runtime_error(
                                ip,
                                code_pos_table,
                                filename,
                                source,
                                OliveRuntimeError::IndexOutOfBounds,
                            ));
                            return Fail(errors);
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            Code::Load(varname) => {
                if let Some(value) = scope.borrow().load(varname) {
                    stack.push(value);
                } else {
                    errors.push(error::create_variable_error(
                        ip,
                        code_pos_table,
                        filename,
                        source,
                        varname,
                    ));
                    return Fail(errors);
                }
            }
            Code::Store(varname) => {
                let value = stack.pop().unwrap();
                scope.borrow_mut().store(varname.clone(), value.clone());
            }
        }
        ip += 1;
    }
}

pub fn start(
    codes: &Vec<Code>,
    code_pos_table: &HashMap<usize, usize>,
    filename: &str,
    source: Option<&str>,
) -> Mistake<(), OliveError> {
    let mut errors = Vec::new();
    let mut gc = garbage::GarbageCollector::new();
    let global_scope = Rc::new(RefCell::new(Scope::new()));
    for (name, function) in builtins::get_functions() {
        global_scope.borrow_mut().store(name, gc.alloc(function));
    }
    attempt!(
        run(
            codes,
            code_pos_table,
            filename,
            source,
            &mut gc,
            global_scope
        ),
        errors
    );
    gc.run(Vec::new());
    return Fine((), errors);
}
