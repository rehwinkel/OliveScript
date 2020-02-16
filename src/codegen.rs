use crate::parser::lexer::Token;
use crate::parser::parser::{Expression, Operator, Statement};
use crate::parser::util::ParserError;

#[derive(Debug, Clone)]
pub enum Code {
    PushString(String),
    PushBoolean(bool),
    PushNone,
    PushFloat(f64),
    PushInt(u64),
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
    Put,
    Get,

    BoolAnd,
    BoolOr,
    Equals,
    NotEquals,
    LessThan,
    LessEquals,
    GreaterThan,
    GreaterEquals,
    Assign,
    Call,
}

trait Generate {
    fn generate(
        &self,
        codes: &mut Vec<Code>,
        is_load: bool,
        is_set: bool,
    ) -> Result<(), ParserError>;
}

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
                        if is_set {
                            codes.push(Code::Store(name.clone()))
                        } else {
                            codes.push(Code::Load(name.clone()))
                        }
                    } else {
                        codes.push(Code::PushString(name.clone()))
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
                    Operator::Get => {
                        rhs.generate(codes, false, false)?;
                        lhs.generate(codes, true, false)?;
                        if is_set {
                            codes.push(Code::Put);
                        } else {
                            codes.push(Code::Get);
                        }
                        return Ok(());
                    }
                    Operator::Assign => {
                        rhs.generate(codes, false, false)?;
                        lhs.generate(codes, true, true)?;
                        return Ok(());
                    }
                    _ => return Err(ParserError::InvalidValue),
                    /*Operator::Equals => Code::Equals,
                    Operator::NotEquals => Code::NotEquals,
                    Operator::LessThan => Code::LessThan,
                    Operator::LessEquals => Code::LessEquals,
                    Operator::GreaterThan => Code::GreaterThan,
                    Operator::GreaterEquals => Code::GreaterEquals,
                    Operator::Get => Code::Get,
                    Operator::ParGet => Code::ParGet,
                    Operator::Call => Code::Call,*/
                };
                rhs.generate(codes, false, false)?;
                lhs.generate(codes, false, false)?;
                codes.push(code);
            }
            _ => unimplemented!(),
            /*
            Expression::NewFunc(Vec<Token>, Box<Statement>),
            Expression::NewList(Vec<Expression>),
            Expression::NewBendy(Vec<BendyPair>),
            Expression::Call(Box<Expression>, Vec<Expression>),
            */
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
            _ => unimplemented!(),
        }
        Ok(())
    }
}

pub fn generate(block: Statement) -> Result<Vec<Code>, ParserError> {
    let mut codes = Vec::new();
    block.generate(&mut codes, false, false)?;
    Ok(codes)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_generate() {
        unimplemented!();
    }
}
