use crate::parser::lexer::Token;
use crate::parser::parser::{Expression, Operator, Statement};
use crate::parser::util::ParserError;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone)]
pub enum Code {
    PushString(String),
    PushBoolean(bool),
    PushFloat(f64),
    PushInt(u64),
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
    BitURsh,
    BitAnd,
    BitOr,
    BitXOr,
    BoolNot,
    Concat,
    Store(String),
    Load(String),
    TStore(usize),
    TLoad(usize),
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
    JumpNot(usize),
    Goto(usize),
}

pub struct NumberedCode {
    pos: usize,
    code: Code,
}

impl Debug for NumberedCode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.pos, self.code)
    }
}

impl Code {
    fn get_code(&self, counter: &mut AtomicUsize) -> NumberedCode {
        NumberedCode {
            pos: counter.fetch_add(self.len(), Ordering::SeqCst),
            code: self.clone(),
        }
    }

    fn push_code(&self, codes: &mut Vec<NumberedCode>, counter: &mut AtomicUsize) {
        codes.push(self.get_code(counter))
    }

    fn len(&self) -> usize {
        match self {
            Code::PushString(_) => 1,
            Code::PushBoolean(_) => 1,
            Code::PushFloat(_) => 1,
            Code::PushInt(_) => 1,
            Code::NewFun(_, _) => 1,
            Code::Store(_) => 1,
            Code::Load(_) => 1,
            Code::TStore(_) => 1,
            Code::TLoad(_) => 1,
            Code::JumpNot(_) => 1,
            Code::Goto(_) => 1,
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
            Code::BitURsh => 1,
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
        }
    }
}

trait Generate {
    fn generate(
        &self,
        codes: &mut Vec<NumberedCode>,
        counter: &mut AtomicUsize,
        is_load: bool,
        is_set: bool,
    ) -> Result<(), ParserError>;
}

static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl Generate for Expression {
    fn generate(
        &self,
        codes: &mut Vec<NumberedCode>,
        counter: &mut AtomicUsize,
        is_load: bool,
        is_set: bool,
    ) -> Result<(), ParserError> {
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
                _ => return Err(ParserError::InvalidValue),
            },
            Expression::Unary(expr, op) => {
                expr.generate(codes, counter, false, false)?;
                match op {
                    Operator::Neg => Code::Neg,
                    Operator::BoolNot => Code::BoolNot,
                    _ => return Err(ParserError::InvalidValue),
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
                    Operator::BitURsh => Code::BitURsh,
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
                    Operator::Assign => {
                        rhs.generate(codes, counter, false, false)?;
                        lhs.generate(codes, counter, false, true)?;
                        return Ok(());
                    }
                    _ => return Err(ParserError::InvalidValue),
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
                    Code::TStore(cnt).push_code(codes, counter);
                    for (i, arg) in args.iter().enumerate() {
                        arg.generate(codes, counter, false, false)?;
                        Code::PushInt(i as u64).push_code(codes, counter);
                        Code::TLoad(cnt).push_code(codes, counter);
                        Code::Put.push_code(codes, counter);
                    }
                    Code::TLoad(cnt).push_code(codes, counter);
                }
            }
            Expression::NewBendy(args) => {
                let cnt = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
                Code::NewBendy.push_code(codes, counter);
                if !args.is_empty() {
                    Code::TStore(cnt).push_code(codes, counter);
                    for pair in args {
                        pair.value.generate(codes, counter, false, false)?;
                        Code::PushString(pair.identifier.clone()).push_code(codes, counter);
                        Code::TLoad(cnt).push_code(codes, counter);
                        Code::Put.push_code(codes, counter);
                    }
                    Code::TLoad(cnt).push_code(codes, counter);
                }
            }
            Expression::NewFunc(args, block) => {
                Code::NewFun(args.clone(), generate(*block.clone())?).push_code(codes, counter);
            }
            _ => panic!(),
        }
        Ok(())
    }
}

impl Generate for Statement {
    fn generate(
        &self,
        codes: &mut Vec<NumberedCode>,
        counter: &mut AtomicUsize,
        _is_load: bool,
        _is_set: bool,
    ) -> Result<(), ParserError> {
        match self {
            Statement::Block(sts) => {
                for st in sts {
                    st.generate(codes, counter, false, false)?;
                }
            }
            Statement::Return(expr) => {
                expr.generate(codes, counter, false, false)?;
                Code::Return.push_code(codes, counter)
            }
            Statement::Expression(expr) => {
                expr.generate(codes, counter, false, false)?;
            }
            Statement::If(cond, ifblock, elseblock) => {
                cond.generate(codes, counter, false, false)?;
                let jumpindex = codes.len();
                Code::JumpNot(0).push_code(codes, counter);
                ifblock.generate(codes, counter, false, false)?;
                if let Some(block) = elseblock {
                    let gotoindex = codes.len();
                    Code::Goto(0).push_code(codes, counter);
                    codes[jumpindex] = NumberedCode {
                        pos: codes[jumpindex].pos,
                        code: Code::JumpNot(codes.len()),
                    };
                    block.generate(codes, counter, false, false)?;
                    codes[gotoindex] = NumberedCode {
                        pos: codes[gotoindex].pos,
                        code: Code::Goto(codes.len()),
                    };
                } else {
                    codes[jumpindex] = NumberedCode {
                        pos: codes[jumpindex].pos,
                        code: Code::JumpNot(codes.len()),
                    };
                }
            }
            /*
            While(Box<Expression>, Box<Statement>),
            Continue,
            Break,
            */
            _ => unimplemented!(),
        }
        Ok(())
    }
}

pub fn generate_codes(block: Statement) -> Result<Vec<NumberedCode>, ParserError> {
    let mut codes = Vec::new();
    let mut counter = AtomicUsize::new(0);
    block.generate(&mut codes, &mut counter, false, false)?;
    /*
    if codes.is_empty() {
        Code::PushNone);
        Code::Return);
    } else {
        match codes.last().unwrap() {
            Code::Return => {}
            _ => {
                Code::PushNone);
                Code::Return);
            }
        }
    }
    */
    Ok(codes)
}

pub fn generate(block: Statement) -> Result<Vec<u8>, ParserError> {
    let codes = generate_codes(block)?;
    for code in codes {
        println!("{:?}", code);
    }
    let bytes: Vec<u8> = vec![0];
    //codes.iter().map(|code| code.to_bytes(&mut constants)).flat_map(|bytes| bytes).collect();
    Ok(bytes)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_generate() {
        unimplemented!();
    }
}
