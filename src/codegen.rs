use crate::parser::lexer::Token;
use crate::parser::parser::{Expression, Operator, Statement};
use crate::parser::util::ParserError;
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
}

trait Generate {
    fn generate(
        &self,
        codes: &mut Vec<Code>,
        is_load: bool,
        is_set: bool,
    ) -> Result<(), ParserError>;
}

static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl Generate for Expression {
    fn generate(
        &self,
        codes: &mut Vec<Code>,
        is_load: bool,
        is_set: bool,
    ) -> Result<(), ParserError> {
        match self {
            Expression::Value(val) => match val {
                Token::ValString(_, string) => codes.push(Code::PushString(string.clone())),
                Token::ValTrue(_) => codes.push(Code::PushBoolean(true)),
                Token::ValFalse(_) => codes.push(Code::PushBoolean(false)),
                Token::ValNone(_) => codes.push(Code::PushNone),
                Token::ValFloat(_, f) => codes.push(Code::PushFloat(*f)),
                Token::ValInt(_, i) => codes.push(Code::PushInt(*i)),
                Token::Ident(_, name) => {
                    if is_load {
                        codes.push(Code::PushString(name.clone()))
                    } else {
                        if is_set {
                            codes.push(Code::Store(name.clone()))
                        } else {
                            codes.push(Code::Load(name.clone()))
                        }
                    }
                }
                _ => return Err(ParserError::InvalidValue),
            },
            Expression::Unary(expr, op) => {
                expr.generate(codes, false, false)?;
                codes.push(match op {
                    Operator::Neg => Code::Neg,
                    Operator::BoolNot => Code::BoolNot,
                    _ => return Err(ParserError::InvalidValue),
                });
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
                        rhs.generate(codes, true, false)?;
                        lhs.generate(codes, false, false)?;
                        if is_set {
                            codes.push(Code::Put);
                        } else {
                            codes.push(Code::Get);
                        }
                        return Ok(());
                    }
                    Operator::Assign => {
                        rhs.generate(codes, false, false)?;
                        lhs.generate(codes, false, true)?;
                        return Ok(());
                    }
                    _ => return Err(ParserError::InvalidValue),
                };
                rhs.generate(codes, false, false)?;
                lhs.generate(codes, false, false)?;
                codes.push(code);
            }
            Expression::Call(func, args) => {
                for arg in args.iter().rev() {
                    arg.generate(codes, false, false)?;
                }
                func.generate(codes, false, false)?;
                codes.push(Code::Call);
            }
            Expression::NewList(args) => {
                let cnt = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
                codes.push(Code::NewList);
                if !args.is_empty() {
                    codes.push(Code::TStore(cnt));
                    for (i, arg) in args.iter().enumerate() {
                        arg.generate(codes, false, false)?;
                        codes.push(Code::PushInt(i as u64));
                        codes.push(Code::TLoad(cnt));
                        codes.push(Code::Put);
                    }
                    codes.push(Code::TLoad(cnt));
                }
            }
            Expression::NewBendy(args) => {
                let cnt = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
                codes.push(Code::NewBendy);
                if !args.is_empty() {
                    codes.push(Code::TStore(cnt));
                    for pair in args {
                        pair.value.generate(codes, false, false)?;
                        codes.push(Code::PushString(pair.identifier.clone()));
                        codes.push(Code::TLoad(cnt));
                        codes.push(Code::Put);
                    }
                    codes.push(Code::TLoad(cnt));
                }
            }
            Expression::NewFunc(args, block) => {
                codes.push(Code::NewFun(args.clone(), generate(*block.clone())?));
            }
            _ => panic!(),
        }
        Ok(())
    }
}

impl Generate for Statement {
    fn generate(
        &self,
        codes: &mut Vec<Code>,
        _is_load: bool,
        _is_set: bool,
    ) -> Result<(), ParserError> {
        match self {
            Statement::Block(sts) => {
                for st in sts {
                    st.generate(codes, false, false)?;
                }
            }
            Statement::Return(expr) => {
                expr.generate(codes, false, false)?;
                codes.push(Code::Return)
            }
            Statement::Expression(expr) => {
                expr.generate(codes, false, false)?;
            }
            Statement::If(cond, ifblock, elseblock) => {
                cond.generate(codes, false, false)?;
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

pub fn generate_codes(block: Statement) -> Result<Vec<Code>, ParserError> {
    let mut codes = Vec::new();
    block.generate(&mut codes, false, false)?;
    /*
    if codes.is_empty() {
        codes.push(Code::PushNone);
        codes.push(Code::Return);
    } else {
        match codes.last().unwrap() {
            Code::Return => {}
            _ => {
                codes.push(Code::PushNone);
                codes.push(Code::Return);
            }
        }
    }
    */
    Ok(codes)
}

pub fn generate(block: Statement) -> Result<Vec<u8>, ParserError> {
    let codes = generate_codes(block)?;
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
