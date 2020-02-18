use crate::parser::lexer::Token;
use crate::parser::parser::{Expression, Operator, Statement};
use crate::parser::util::ParserError;
use indexmap::IndexSet;
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
    Break(usize),
}

#[derive(Clone)]
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
    fn to_bytes(&self, constants: &mut IndexSet<String>) -> Vec<u8> {
        match self {
            Code::PushString(_) => vec![1],
            Code::PushBoolean(_) => vec![2],
            Code::PushFloat(_) => vec![3],
            Code::PushInt(_) => vec![4],
            Code::NewFun(_, _) => vec![5],
            Code::Store(_) => vec![6],
            Code::Load(_) => vec![7],
            Code::TStore(_) => vec![8],
            Code::TLoad(_) => vec![9],
            Code::JumpNot(_) => vec![10],
            Code::Goto(_) | Code::Break(_) => vec![11],
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
            Code::BitURsh => vec![25],
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
        }
    }

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
        let intsize = std::mem::size_of::<usize>();
        match self {
            Code::PushString(_) => 1 + intsize,
            Code::PushBoolean(_) => 2,
            Code::PushFloat(_) => 9,
            Code::PushInt(_) => 9,
            Code::NewFun(_, _) => 1, //TODO
            Code::Store(_) => 1,
            Code::Load(_) => 1,
            Code::TStore(_) => 1,
            Code::TLoad(_) => 1,
            Code::JumpNot(_) => 1 + intsize,
            Code::Goto(_) | Code::Break(_) => 1 + intsize,
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

static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl Expression {
    fn generate(
        &self,
        codes: &mut Vec<NumberedCode>,
        counter: &mut AtomicUsize,
        is_load: bool,
        is_set: bool,
        constants: &mut IndexSet<String>,
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
                expr.generate(codes, counter, false, false, constants)?;
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
                        rhs.generate(codes, counter, true, false, constants)?;
                        lhs.generate(codes, counter, false, false, constants)?;
                        if is_set {
                            Code::Put.push_code(codes, counter);
                        } else {
                            Code::Get.push_code(codes, counter);
                        }
                        return Ok(());
                    }
                    Operator::Assign => {
                        rhs.generate(codes, counter, false, false, constants)?;
                        lhs.generate(codes, counter, false, true, constants)?;
                        return Ok(());
                    }
                    _ => return Err(ParserError::InvalidValue),
                };
                rhs.generate(codes, counter, false, false, constants)?;
                lhs.generate(codes, counter, false, false, constants)?;
                code.push_code(codes, counter);
            }
            Expression::Call(func, args) => {
                for arg in args.iter().rev() {
                    arg.generate(codes, counter, false, false, constants)?;
                }
                func.generate(codes, counter, false, false, constants)?;
                Code::Call.push_code(codes, counter);
            }
            Expression::NewList(args) => {
                let cnt = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
                Code::NewList.push_code(codes, counter);
                if !args.is_empty() {
                    Code::TStore(cnt).push_code(codes, counter);
                    for (i, arg) in args.iter().enumerate() {
                        arg.generate(codes, counter, false, false, constants)?;
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
                        pair.value.generate(codes, counter, false, false, constants)?;
                        Code::PushString(pair.identifier.clone()).push_code(codes, counter);
                        Code::TLoad(cnt).push_code(codes, counter);
                        Code::Put.push_code(codes, counter);
                    }
                    Code::TLoad(cnt).push_code(codes, counter);
                }
            }
            Expression::NewFunc(args, block) => {
                Code::NewFun(args.clone(), generate(*block.clone(), constants)?).push_code(codes, counter);
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
        while_start_index: usize,
        while_end_index: usize,
        constants: &mut IndexSet<String>,
    ) -> Result<(), ParserError> {
        match self {
            Statement::Block(sts) => {
                for st in sts {
                    st.generate(codes, counter, while_start_index, while_end_index, constants)?;
                }
            }
            Statement::Return(expr) => {
                expr.generate(codes, counter, false, false, constants)?;
                Code::Return.push_code(codes, counter)
            }
            Statement::Expression(expr) => {
                expr.generate(codes, counter, false, false, constants)?;
            }
            Statement::If(cond, ifblock, elseblock) => {
                cond.generate(codes, counter, false, false, constants)?;
                let jumpindex = codes.len();
                Code::JumpNot(0).push_code(codes, counter);
                ifblock.generate(codes, counter, while_start_index, while_end_index, constants)?;
                if let Some(block) = elseblock {
                    let gotoindex = codes.len();
                    Code::Goto(0).push_code(codes, counter);
                    codes[jumpindex] = NumberedCode {
                        pos: codes[jumpindex].pos,
                        code: Code::JumpNot(codes.len()),
                    };
                    block.generate(codes, counter, while_start_index, while_end_index, constants)?;
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
            Statement::While(cond, block) => {
                let repeat_index = codes.len();
                cond.generate(codes, counter, false, false, constants)?;
                let end_index = codes.len();
                Code::JumpNot(0).push_code(codes, counter);
                block.generate(codes, counter, repeat_index, end_index, constants)?;
                Code::Goto(repeat_index).push_code(codes, counter);
                codes[end_index] = NumberedCode {
                    pos: codes[end_index].pos,
                    code: Code::JumpNot(codes.len()),
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

pub fn generate_codes(
    block: Statement,
    constants: &mut IndexSet<String>,
) -> Result<Vec<NumberedCode>, ParserError> {
    let mut codes = Vec::new();
    let mut counter = AtomicUsize::new(0);
    block.generate(&mut codes, &mut counter, 0, 0, constants)?;
    Code::PushNone.push_code(&mut codes, &mut counter);
    Code::Return.push_code(&mut codes, &mut counter);
    for (i, code) in codes.clone().into_iter().enumerate() {
        match code.code {
            Code::JumpNot(index) => codes[i].code = Code::JumpNot(codes[index].pos),
            Code::Goto(index) => codes[i].code = Code::Goto(codes[index].pos),
            Code::Break(index) => {
                if let Code::JumpNot(i) = codes[index].code {
                    codes[i].code = Code::Goto(i)
                }
            }
            _ => {}
        }
    }
    Ok(codes)
}

pub fn generate(
    block: Statement,
    constants: &mut IndexSet<String>,
) -> Result<Vec<u8>, ParserError> {
    Ok(generate_codes(block, constants)?
        .iter()
        .map(|code| code.code.to_bytes(constants))
        .flat_map(|bytes| bytes)
        .collect())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_generate() {
        unimplemented!();
    }
}
