use crate::parser::lexer::Token;
use crate::parser::parser::{Expression, Operator, Statement};
use std::fmt::Debug;
use std::fmt::Formatter;
use std::mem::transmute;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod util {
    use std::error::Error;
    use std::fmt::Display;
    use std::fmt::Error as FmtError;
    use std::fmt::Formatter;
    #[derive(Debug)]
    pub enum CodeGenError {
        InvalidValue,
    }

    impl Error for CodeGenError {}

    impl Display for CodeGenError {
        fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
            match self {
                CodeGenError::InvalidValue => write!(f, "invalid value"),
            }
        }
    }
}

use util::CodeGenError;

#[derive(Debug, Clone)]
pub enum Code {
    PushString(String),
    PushBoolean(bool),
    PushFloat(f64),
    PushInt(i64),
    PushNone,
    NewFun(Vec<String>, Vec<u8>),
    NewBendy,
    NewList,
    Return,
    Neg,
    Add,
    Sub,
    Mul,
    IntDiv,
    FloatDiv,
    Mod,
    BitLsh,
    BitRsh,
    BitAnd,
    BitOr,
    BitXOr,
    BoolNot,
    Concat,
    Store(String),
    Load(String),
    Put,
    Get,
    Call,
    BoolAnd,
    BoolOr,
    Equals,
    NotEquals,
    LessThan,
    LessEquals,
    GreaterThan,
    GreaterEquals,
    JumpNot(u32),
    Goto(u32),
    Break(u32),
    Pop,
}

#[derive(Clone)]
pub struct NumberedCode {
    pos: u32,
    code: Code,
}

impl Debug for NumberedCode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.pos, self.code)
    }
}

impl Code {
    pub fn u16_to_bytes(val: u16) -> Vec<u8> {
        let bytes: [u8; 2] = unsafe { transmute(val.to_le()) };
        bytes.to_vec()
    }

    pub fn u32_to_bytes(val: u32) -> Vec<u8> {
        let bytes: [u8; 4] = unsafe { transmute(val.to_le()) };
        bytes.to_vec()
    }

    fn i64_to_bytes(val: i64) -> Vec<u8> {
        let bytes: [u8; std::mem::size_of::<i64>()] = unsafe { transmute(val.to_le()) };
        bytes.to_vec()
    }

    fn f64_to_bytes(val: f64) -> Vec<u8> {
        let bytes: [u8; std::mem::size_of::<f64>()] = unsafe { transmute(val) };
        bytes.to_vec()
    }

    fn string_to_bytes(val: String) -> Vec<u8> {
        [
            Code::u16_to_bytes(val.len() as u16),
            val.as_bytes().to_vec(),
        ]
        .concat()
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Code::PushString(s) => [vec![1], Code::string_to_bytes(s.clone())].concat(),
            Code::PushBoolean(b) => vec![2, *b as u8],
            Code::PushFloat(f) => [vec![3], Code::f64_to_bytes(*f)].concat(),
            Code::PushInt(i) => [vec![4], Code::i64_to_bytes(*i)].concat(),
            Code::Store(name) => [vec![6], Code::string_to_bytes(name.clone())].concat(),
            Code::Load(name) => [vec![7], Code::string_to_bytes(name.clone())].concat(),
            Code::JumpNot(p) => [vec![10], Code::u32_to_bytes(*p)].concat(),
            Code::Goto(p) | Code::Break(p) => [vec![11], Code::u32_to_bytes(*p)].concat(),
            Code::NewFun(args, codes) => {
                let arglen = Code::u16_to_bytes(args.len() as u16);
                let argindices: Vec<u8> = args
                    .iter()
                    .map(|arg| Code::string_to_bytes(arg.clone()))
                    .flat_map(|bytes| bytes)
                    .collect();
                let codeslen = Code::u32_to_bytes(codes.len() as u32);
                [vec![5], arglen, argindices, codeslen, codes.clone()].concat()
            }
            Code::PushNone => vec![12],
            Code::NewBendy => vec![13],
            Code::NewList => vec![14],
            Code::Return => vec![15],
            Code::Neg => vec![16],
            Code::Add => vec![17],
            Code::Sub => vec![18],
            Code::Mul => vec![19],
            Code::IntDiv => vec![20],
            Code::FloatDiv => vec![21],
            Code::Mod => vec![22],
            Code::BitLsh => vec![23],
            Code::BitRsh => vec![24],
            Code::BitAnd => vec![26],
            Code::BitOr => vec![27],
            Code::BitXOr => vec![28],
            Code::BoolNot => vec![29],
            Code::Concat => vec![30],
            Code::Put => vec![31],
            Code::Get => vec![32],
            Code::Call => vec![33],
            Code::BoolAnd => vec![34],
            Code::BoolOr => vec![35],
            Code::Equals => vec![36],
            Code::NotEquals => vec![37],
            Code::LessThan => vec![38],
            Code::LessEquals => vec![39],
            Code::GreaterThan => vec![40],
            Code::GreaterEquals => vec![41],
            Code::Pop => vec![42],
        }
    }

    fn get_code(&self, counter: &mut AtomicUsize) -> NumberedCode {
        NumberedCode {
            pos: counter.fetch_add(self.len(), Ordering::SeqCst) as u32,
            code: self.clone(),
        }
    }

    fn push_code(&self, codes: &mut Vec<NumberedCode>, counter: &mut AtomicUsize) {
        codes.push(self.get_code(counter))
    }
    fn string_len(s: &String) -> usize {
        s.len() + 2
    }

    fn len(&self) -> usize {
        match self {
            Code::PushString(s) => 1 + Code::string_len(s),
            Code::PushBoolean(_) => 2,
            Code::PushFloat(_) => 9,
            Code::PushInt(_) => 9,
            Code::NewFun(args, codes) => {
                let strargs: usize = args.iter().map(|arg| Code::string_len(arg)).sum();
                1 + 2 + strargs + 4 + codes.len()
            }
            Code::Store(s) => 1 + Code::string_len(s),
            Code::Load(s) => 1 + Code::string_len(s),
            Code::JumpNot(_) => 5,
            Code::Goto(_) | Code::Break(_) => 5,
            Code::PushNone => 1,
            Code::NewBendy => 1,
            Code::NewList => 1,
            Code::Return => 1,
            Code::Neg => 1,
            Code::Add => 1,
            Code::Sub => 1,
            Code::Mul => 1,
            Code::IntDiv => 1,
            Code::FloatDiv => 1,
            Code::Mod => 1,
            Code::BitLsh => 1,
            Code::BitRsh => 1,
            Code::BitAnd => 1,
            Code::BitOr => 1,
            Code::BitXOr => 1,
            Code::BoolNot => 1,
            Code::Concat => 1,
            Code::Put => 1,
            Code::Get => 1,
            Code::Call => 1,
            Code::BoolAnd => 1,
            Code::BoolOr => 1,
            Code::Equals => 1,
            Code::NotEquals => 1,
            Code::LessThan => 1,
            Code::LessEquals => 1,
            Code::GreaterThan => 1,
            Code::GreaterEquals => 1,
            Code::Pop => 1,
        }
    }
}

static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl Expression {
    fn generate(
        &self,
        codes: &mut Vec<NumberedCode>,
        counter: &mut AtomicUsize,
        is_load: bool,
        is_set: bool,
    ) -> Result<(), CodeGenError> {
        match self {
            Expression::Value(val) => match val {
                Token::ValString(_, string) => {
                    Code::PushString(string.clone()).push_code(codes, counter)
                }
                Token::ValTrue(_) => Code::PushBoolean(true).push_code(codes, counter),
                Token::ValFalse(_) => Code::PushBoolean(false).push_code(codes, counter),
                Token::ValNone(_) => Code::PushNone.push_code(codes, counter),
                Token::ValFloat(_, f) => Code::PushFloat(*f).push_code(codes, counter),
                Token::ValInt(_, i) => Code::PushInt(*i).push_code(codes, counter),
                Token::Ident(_, name) => {
                    if is_load {
                        Code::PushString(name.clone()).push_code(codes, counter)
                    } else {
                        if is_set {
                            Code::Store(name.clone()).push_code(codes, counter)
                        } else {
                            Code::Load(name.clone()).push_code(codes, counter)
                        }
                    }
                }
                _ => return Err(CodeGenError::InvalidValue),
            },
            Expression::Unary(expr, op) => {
                expr.generate(codes, counter, false, false)?;
                match op {
                    Operator::Neg => Code::Neg,
                    Operator::BoolNot => Code::BoolNot,
                    _ => return Err(CodeGenError::InvalidValue),
                }
                .push_code(codes, counter);
            }
            Expression::Binary(lhs, rhs, op) => {
                let code = match op {
                    Operator::Add => Code::Add,
                    Operator::Sub => Code::Sub,
                    Operator::Mul => Code::Mul,
                    Operator::IntDiv => Code::IntDiv,
                    Operator::FloatDiv => Code::FloatDiv,
                    Operator::Mod => Code::Mod,
                    Operator::BitLsh => Code::BitLsh,
                    Operator::BitRsh => Code::BitRsh,
                    Operator::BitAnd => Code::BitAnd,
                    Operator::BitOr => Code::BitOr,
                    Operator::BitXOr => Code::BitXOr,
                    Operator::Concat => Code::Concat,
                    Operator::BoolAnd => Code::BoolAnd,
                    Operator::BoolOr => Code::BoolOr,
                    Operator::Equals => Code::Equals,
                    Operator::NotEquals => Code::NotEquals,
                    Operator::LessThan => Code::LessThan,
                    Operator::LessEquals => Code::LessEquals,
                    Operator::GreaterThan => Code::GreaterThan,
                    Operator::GreaterEquals => Code::GreaterEquals,
                    Operator::Get => {
                        rhs.generate(codes, counter, true, false)?;
                        lhs.generate(codes, counter, false, false)?;
                        if is_set {
                            Code::Put.push_code(codes, counter);
                        } else {
                            Code::Get.push_code(codes, counter);
                        }
                        return Ok(());
                    }
                    Operator::ParGet => {
                        rhs.generate(codes, counter, false, false)?;
                        lhs.generate(codes, counter, false, false)?;
                        if is_set {
                            Code::Put.push_code(codes, counter);
                        } else {
                            Code::Get.push_code(codes, counter);
                        }
                        return Ok(());
                    }
                    Operator::Assign => {
                        rhs.generate(codes, counter, false, false)?;
                        lhs.generate(codes, counter, false, true)?;
                        return Ok(());
                    }
                    _ => return Err(CodeGenError::InvalidValue),
                };
                rhs.generate(codes, counter, false, false)?;
                lhs.generate(codes, counter, false, false)?;
                code.push_code(codes, counter);
            }
            Expression::Call(func, args) => {
                for arg in args.iter().rev() {
                    arg.generate(codes, counter, false, false)?;
                }
                func.generate(codes, counter, false, false)?;
                Code::Call.push_code(codes, counter);
            }
            Expression::NewList(args) => {
                let cnt = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
                Code::NewList.push_code(codes, counter);
                if !args.is_empty() {
                    Code::Store(format!("<{}>", cnt)).push_code(codes, counter);
                    for (i, arg) in args.iter().enumerate() {
                        arg.generate(codes, counter, false, false)?;
                        Code::PushInt(i as i64).push_code(codes, counter);
                        Code::Load(format!("<{}>", cnt)).push_code(codes, counter);
                        Code::Put.push_code(codes, counter);
                    }
                    Code::Load(format!("<{}>", cnt)).push_code(codes, counter);
                }
            }
            Expression::NewBendy(args) => {
                let cnt = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
                Code::NewBendy.push_code(codes, counter);
                if !args.is_empty() {
                    Code::Store(format!("<{}>", cnt)).push_code(codes, counter);
                    for pair in args {
                        pair.value.generate(codes, counter, false, false)?;
                        Code::PushString(pair.identifier.clone()).push_code(codes, counter);
                        Code::Load(format!("<{}>", cnt)).push_code(codes, counter);
                        Code::Put.push_code(codes, counter);
                    }
                    Code::Load(format!("<{}>", cnt)).push_code(codes, counter);
                }
            }
            Expression::NewFunc(args, block) => {
                Code::NewFun(args.clone(), to_bytes(generate(*block.clone())?))
                    .push_code(codes, counter);
            }
            _ => panic!(),
        }
        Ok(())
    }
}

impl Statement {
    fn generate(
        &self,
        codes: &mut Vec<NumberedCode>,
        counter: &mut AtomicUsize,
        while_start_index: u32,
        while_end_index: u32,
    ) -> Result<(), CodeGenError> {
        match self {
            Statement::Block(sts) => {
                for st in sts {
                    st.generate(codes, counter, while_start_index, while_end_index)?;
                }
            }
            Statement::Return(expr) => {
                expr.generate(codes, counter, false, false)?;
                Code::Return.push_code(codes, counter)
            }
            Statement::Expression(expr) => {
                expr.generate(codes, counter, false, false)?;
                if let Expression::Call(_, _) = &**expr {
                    Code::Pop.push_code(codes, counter);
                }
            }
            Statement::If(cond, ifblock, elseblock) => {
                cond.generate(codes, counter, false, false)?;
                let jumpindex = codes.len();
                Code::JumpNot(0).push_code(codes, counter);
                ifblock.generate(codes, counter, while_start_index, while_end_index)?;
                if let Some(block) = elseblock {
                    let gotoindex = codes.len();
                    Code::Goto(0).push_code(codes, counter);
                    codes[jumpindex] = NumberedCode {
                        pos: codes[jumpindex].pos,
                        code: Code::JumpNot(codes.len() as u32),
                    };
                    block.generate(codes, counter, while_start_index, while_end_index)?;
                    codes[gotoindex] = NumberedCode {
                        pos: codes[gotoindex].pos,
                        code: Code::Goto(codes.len() as u32),
                    };
                } else {
                    codes[jumpindex] = NumberedCode {
                        pos: codes[jumpindex].pos,
                        code: Code::JumpNot(codes.len() as u32),
                    };
                }
            }
            Statement::While(cond, block) => {
                let repeat_index = codes.len() as u32;
                cond.generate(codes, counter, false, false)?;
                let end_index = codes.len() as u32;
                Code::JumpNot(0).push_code(codes, counter);
                block.generate(codes, counter, repeat_index, end_index)?;
                Code::Goto(repeat_index).push_code(codes, counter);
                codes[end_index as usize] = NumberedCode {
                    pos: codes[end_index as usize].pos,
                    code: Code::JumpNot(codes.len() as u32),
                };
            }
            Statement::Continue => {
                Code::Goto(while_start_index).push_code(codes, counter);
            }
            Statement::Break => {
                Code::Break(while_end_index).push_code(codes, counter);
            }
        }
        Ok(())
    }
}

pub fn generate(block: Statement) -> Result<Vec<NumberedCode>, CodeGenError> {
    let mut codes = Vec::new();
    let mut counter = AtomicUsize::new(0);
    block.generate(&mut codes, &mut counter, 0, 0)?;
    Code::PushNone.push_code(&mut codes, &mut counter);
    Code::Return.push_code(&mut codes, &mut counter);
    for (i, code) in codes.clone().into_iter().enumerate() {
        match code.code {
            Code::JumpNot(index) => codes[i].code = Code::JumpNot(codes[index as usize].pos as u32),
            Code::Goto(index) => codes[i].code = Code::Goto(codes[index as usize].pos as u32),
            Code::Break(index) => {
                if let Code::JumpNot(j) = codes[index as usize].code {
                    codes[i].code = Code::Goto(j)
                }
            }
            _ => {}
        }
    }
    Ok(codes)
}

pub fn to_bytes(codes: Vec<NumberedCode>) -> Vec<u8> {
    codes
        .iter()
        .map(|code| code.code.to_bytes())
        .flat_map(|bytes| bytes)
        .collect()
}

#[cfg(test)]
mod test {
    #[test]
    fn test_generate() {
        unimplemented!();
    }
}
