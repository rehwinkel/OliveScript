use super::errors::{OliveCodeError, OliveError};
use mistake::Mistake::{self, Fail, Fine};
use oliveparser::ast::{BinaryOperator, Expression, Located, Statement, UnaryOperator};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Code {
    PushString(String),
    PushBoolean(bool),
    PushDouble(f64),
    PushLong(i64),
    PushInt(i32),
    PushShort(i16),
    PushByte(i8),
    PushBendy,
    PushList,
    PushNone,
    Pop,
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
    Put,
    Get,
    Call,
    Equals,
    NotEquals,
    LessThan,
    LessEquals,
    GreaterThan,
    GreaterEquals,
    Dup,
    JumpNot(i32),
    Jump(i32),
    Goto(i32),
    Store(String),
    Load(String),
    PushFun(Vec<String>, Vec<Code>),
}

trait Generatable {
    fn generate(
        self,
        codes: &mut Vec<Code>,
        filename: &str,
        source: &str,
        code_pos_table: &mut HashMap<usize, usize>,
    ) -> Mistake<(u32, Vec<usize>), OliveError>;
    fn generate_lhs(
        self,
        codes: &mut Vec<Code>,
        filename: &str,
        source: &str,
        code_pos_table: &mut HashMap<usize, usize>,
    ) -> Mistake<u32, OliveError>;
}

fn push_integer(codes: &mut Vec<Code>, value: usize) -> bool {
    if value < 0x80 {
        codes.push(Code::PushByte(value as i8));
    } else if value < 0x8000 {
        codes.push(Code::PushShort(value as i16));
    } else if value < 0x80000000 {
        codes.push(Code::PushInt(value as i32));
    } else if value < 0x8000000000000000 {
        codes.push(Code::PushLong(value as i64));
    } else {
        return false;
    }
    true
}

impl<'a> Generatable for Located<Expression<'a>> {
    fn generate(
        self,
        codes: &mut Vec<Code>,
        filename: &str,
        source: &str,
        code_pos_table: &mut HashMap<usize, usize>,
    ) -> Mistake<(u32, Vec<usize>), OliveError> {
        let mut errors = Vec::new();

        Fine(
            match self.inner {
                Expression::Integer { value } => {
                    codes.push(if let Ok(ival) = value.parse::<i8>() {
                        Code::PushByte(ival)
                    } else if let Ok(ival) = value.parse::<i16>() {
                        Code::PushShort(ival)
                    } else if let Ok(ival) = value.parse::<i32>() {
                        Code::PushInt(ival)
                    } else if let Ok(ival) = value.parse::<i64>() {
                        Code::PushLong(ival)
                    } else {
                        errors.push(OliveError::new_code_error(
                            self.start,
                            filename,
                            source,
                            OliveCodeError::ParseInteger {
                                value: String::from(value),
                            },
                        ));
                        return Fail(errors);
                    });
                    (1, Vec::new())
                }
                Expression::Float { value } => {
                    codes.push(if let Ok(ival) = value.parse::<f64>() {
                        Code::PushDouble(ival)
                    } else {
                        errors.push(OliveError::new_code_error(
                            self.start,
                            filename,
                            source,
                            OliveCodeError::ParseFloat {
                                value: String::from(value),
                            },
                        ));
                        return Fail(errors);
                    });
                    (1, Vec::new())
                }
                Expression::Boolean { value } => {
                    codes.push(Code::PushBoolean(value));
                    (1, Vec::new())
                }
                Expression::None => {
                    codes.push(Code::PushNone);
                    (1, Vec::new())
                }
                Expression::Unary {
                    expression,
                    operator,
                } => {
                    let expression_size = attempt!(
                        expression.generate(codes, filename, source, code_pos_table),
                        errors
                    );
                    code_pos_table.insert(codes.len(), self.start);
                    match operator {
                        UnaryOperator::Neg => codes.push(Code::Neg),
                        UnaryOperator::BoolNot => codes.push(Code::BoolNot),
                    }
                    (expression_size.0 + 1, Vec::new())
                }
                Expression::Binary {
                    left,
                    right,
                    operator,
                } => match operator {
                    BinaryOperator::BoolAnd => {
                        let left_opt = left
                            .generate(codes, filename, source, code_pos_table)
                            .to_option(&mut errors);
                        let first_jump_index = codes.len();
                        codes.push(Code::JumpNot(0));
                        let right_opt = right
                            .generate(codes, filename, source, code_pos_table)
                            .to_option(&mut errors);
                        if let None = right_opt {
                            return Fail(errors);
                        }
                        if let None = left_opt {
                            return Fail(errors);
                        }
                        match &mut codes[first_jump_index] {
                            Code::JumpNot(pos) => *pos = (right_opt.as_ref().unwrap().0 + 2) as i32,
                            _ => panic!(),
                        }
                        codes.push(Code::Goto(2));
                        codes.push(Code::PushBoolean(false));
                        (3 + left_opt.unwrap().0 + right_opt.unwrap().0, Vec::new())
                    }
                    BinaryOperator::BoolOr => {
                        let left_opt = left
                            .generate(codes, filename, source, code_pos_table)
                            .to_option(&mut errors);
                        let first_jump_index = codes.len();
                        codes.push(Code::Jump(0));
                        let right_opt = right
                            .generate(codes, filename, source, code_pos_table)
                            .to_option(&mut errors);
                        if let None = right_opt {
                            return Fail(errors);
                        }
                        if let None = left_opt {
                            return Fail(errors);
                        }
                        match &mut codes[first_jump_index] {
                            Code::Jump(pos) => *pos = (right_opt.as_ref().unwrap().0 + 2) as i32,
                            _ => panic!(),
                        }
                        codes.push(Code::Goto(2));
                        codes.push(Code::PushBoolean(true));
                        (3 + left_opt.unwrap().0 + right_opt.unwrap().0, Vec::new())
                    }
                    BinaryOperator::Access => {
                        let left_opt = left
                            .generate(codes, filename, source, code_pos_table)
                            .to_option(&mut errors);
                        let name = match right.inner {
                            Expression::Variable { name } => name,
                            _ => {
                                errors.push(OliveError::new_code_error(
                                    right.start,
                                    filename,
                                    source,
                                    OliveCodeError::Access,
                                ));
                                return Fail(errors);
                            }
                        };
                        if let None = left_opt {
                            return Fail(errors);
                        }
                        codes.push(Code::PushString(String::from(name)));
                        codes.push(Code::Get);
                        (left_opt.unwrap().0 + 2, Vec::new())
                    }
                    _ => {
                        let left_opt = left
                            .generate(codes, filename, source, code_pos_table)
                            .to_option(&mut errors);
                        let right_opt = right
                            .generate(codes, filename, source, code_pos_table)
                            .to_option(&mut errors);
                        if let None = left_opt {
                            return Fail(errors);
                        }
                        if let None = right_opt {
                            return Fail(errors);
                        }
                        code_pos_table.insert(codes.len(), self.start);
                        match operator {
                            BinaryOperator::Add => codes.push(Code::Add),
                            BinaryOperator::Sub => codes.push(Code::Sub),
                            BinaryOperator::Mul => codes.push(Code::Mul),
                            BinaryOperator::Mod => codes.push(Code::Mod),
                            BinaryOperator::FloatDiv => codes.push(Code::FloatDiv),
                            BinaryOperator::IntDiv => codes.push(Code::IntDiv),
                            BinaryOperator::BitAnd => codes.push(Code::BitAnd),
                            BinaryOperator::BitOr => codes.push(Code::BitOr),
                            BinaryOperator::BitXOr => codes.push(Code::BitXOr),
                            BinaryOperator::BitLsh => codes.push(Code::BitLsh),
                            BinaryOperator::BitRsh => codes.push(Code::BitRsh),
                            BinaryOperator::Concat => codes.push(Code::Concat),
                            BinaryOperator::Equals => codes.push(Code::Equals),
                            BinaryOperator::NotEquals => codes.push(Code::NotEquals),
                            BinaryOperator::LessEquals => codes.push(Code::LessEquals),
                            BinaryOperator::LessThan => codes.push(Code::LessThan),
                            BinaryOperator::GreaterEquals => codes.push(Code::GreaterEquals),
                            BinaryOperator::GreaterThan => codes.push(Code::GreaterThan),
                            _ => panic!(),
                        }
                        (left_opt.unwrap().0 + right_opt.unwrap().0 + 1, Vec::new())
                    }
                },
                Expression::Index { expression, index } => {
                    let left_opt = expression
                        .generate(codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    let right_opt = index
                        .generate(codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    if let None = left_opt {
                        return Fail(errors);
                    }
                    if let None = right_opt {
                        return Fail(errors);
                    }
                    code_pos_table.insert(codes.len(), self.start);
                    codes.push(Code::Get);
                    (left_opt.unwrap().0 + right_opt.unwrap().0 + 1, Vec::new())
                }
                Expression::String { value } => {
                    codes.push(Code::PushString(value));
                    (1, Vec::new())
                }
                Expression::Call { expression, args } => {
                    let results: Vec<Option<u32>> = args
                        .into_iter()
                        .map(|arg| {
                            match arg
                                .generate(codes, filename, source, code_pos_table)
                                .to_option(&mut errors)
                            {
                                Some((i, _)) => Some(i),
                                None => None,
                            }
                        })
                        .collect();
                    let expression_opt = expression
                        .generate(codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    let mut size = 0;
                    for res in results {
                        if let Some(l) = res {
                            size += l;
                        } else {
                            return Fail(errors);
                        }
                    }
                    if let None = expression_opt {
                        return Fail(errors);
                    }
                    code_pos_table.insert(codes.len(), self.start);
                    codes.push(Code::Call);
                    (1 + expression_opt.unwrap().0 + size, Vec::new())
                }
                Expression::List { elements } => {
                    codes.push(Code::PushList);
                    if elements.len() > 0 {
                        let results: Vec<Option<u32>> = elements
                            .into_iter()
                            .enumerate()
                            .map(|(i, arg)| {
                                codes.push(Code::Dup);
                                push_integer(codes, i);
                                let opt: Option<u32> = match arg
                                    .generate(codes, filename, source, code_pos_table)
                                    .to_option(&mut errors)
                                {
                                    Some((i, _)) => Some(i),
                                    None => None,
                                };
                                codes.push(Code::Put);
                                opt
                            })
                            .collect();
                        for res in &results {
                            if let None = res {
                                return Fail(errors);
                            }
                        }
                        (
                            1 + results.into_iter().map(|x| x.unwrap() + 3).sum::<u32>(),
                            Vec::new(),
                        )
                    } else {
                        (1, Vec::new())
                    }
                }
                Expression::Bendy { elements } => {
                    codes.push(Code::PushBendy);
                    if elements.len() > 0 {
                        let results: Vec<Option<u32>> = elements
                            .into_iter()
                            .map(|(name, arg)| {
                                codes.push(Code::Dup);
                                codes.push(Code::PushString(String::from(name.inner)));
                                let opt = match arg
                                    .generate(codes, filename, source, code_pos_table)
                                    .to_option(&mut errors)
                                {
                                    Some((i, _)) => Some(i),
                                    None => None,
                                };
                                codes.push(Code::Put);
                                opt
                            })
                            .collect();
                        for res in &results {
                            if let None = res {
                                return Fail(errors);
                            }
                        }
                        (
                            1 + results.into_iter().map(|x| x.unwrap() + 3).sum::<u32>(),
                            Vec::new(),
                        )
                    } else {
                        (1, Vec::new())
                    }
                }
                Expression::Variable { name } => {
                    code_pos_table.insert(codes.len(), self.start);
                    codes.push(Code::Load(String::from(name)));
                    (1, Vec::new())
                }
                Expression::Function { parameters, block } => {
                    let (inner_codes, code_pos) =
                        attempt!(generate_codes(block, filename, source), errors);
                    code_pos_table.extend(code_pos);
                    codes.push(Code::PushFun(
                        parameters.iter().map(|s| String::from(s.inner)).collect(),
                        inner_codes,
                    ));
                    (1, Vec::new())
                }
            },
            errors,
        )
    }

    fn generate_lhs(
        self,
        codes: &mut Vec<Code>,
        filename: &str,
        source: &str,
        code_pos_table: &mut HashMap<usize, usize>,
    ) -> Mistake<u32, OliveError> {
        let mut errors = Vec::new();
        Fine(
            match self.inner {
                Expression::Binary {
                    left,
                    right,
                    operator,
                } => match operator {
                    BinaryOperator::Access => {
                        let left_opt = left
                            .generate(codes, filename, source, code_pos_table)
                            .to_option(&mut errors);
                        let name = match right.inner {
                            Expression::Variable { name } => name,
                            _ => {
                                errors.push(OliveError::new_code_error(
                                    right.start,
                                    filename,
                                    source,
                                    OliveCodeError::Access,
                                ));
                                return Fail(errors);
                            }
                        };
                        if let None = left_opt {
                            return Fail(errors);
                        }
                        codes.push(Code::PushString(String::from(name)));
                        left_opt.unwrap().0 + 1
                    }
                    _ => {
                        errors.push(OliveError::new_code_error(
                            self.start,
                            filename,
                            source,
                            OliveCodeError::Assign {
                                expression_type: String::from(&source[self.start..self.end]),
                            },
                        ));
                        return Fail(errors);
                    }
                },
                Expression::Index { expression, index } => {
                    let left_opt = expression
                        .generate(codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    let right_opt = index
                        .generate(codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    if let None = left_opt {
                        return Fail(errors);
                    }
                    if let None = right_opt {
                        return Fail(errors);
                    }
                    left_opt.unwrap().0 + right_opt.unwrap().0
                }
                Expression::Variable { name: _ } => 0,
                _ => {
                    errors.push(OliveError::new_code_error(
                        self.start,
                        filename,
                        source,
                        OliveCodeError::Assign {
                            expression_type: String::from(&source[self.start..self.end]),
                        },
                    ));
                    return Fail(errors);
                }
            },
            errors,
        )
    }
}

impl<'a> Generatable for Located<Statement<'a>> {
    fn generate(
        self,
        codes: &mut Vec<Code>,
        filename: &str,
        source: &str,
        code_pos_table: &mut HashMap<usize, usize>,
    ) -> Mistake<(u32, Vec<usize>), OliveError> {
        let mut errors = Vec::new();

        Fine(
            match self.inner {
                Statement::Return { value } => {
                    let value_size = attempt!(
                        value.generate(codes, filename, source, code_pos_table),
                        errors
                    )
                    .0;
                    codes.push(Code::Return);
                    (value_size + 1, Vec::new())
                }
                Statement::If {
                    condition,
                    block,
                    elseblock,
                } => {
                    let mut break_positions = Vec::new();
                    let condition_opt = condition
                        .generate(codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    let first_jump_index = codes.len();
                    codes.push(Code::JumpNot(0));
                    let block_opt = generate_block(block, codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    let else_bonus = if elseblock.is_some() { 1 } else { 0 };
                    let else_size = if let Some(elseblock) = elseblock {
                        let second_jump_index = codes.len();
                        codes.push(Code::Goto(0));
                        let elseblock_opt =
                            generate_block(elseblock, codes, filename, source, code_pos_table)
                                .to_option(&mut errors);
                        if let None = elseblock_opt {
                            return Fail(errors);
                        }
                        break_positions.extend(&elseblock_opt.as_ref().unwrap().1);
                        match &mut codes[second_jump_index] {
                            Code::Goto(pos) => {
                                *pos = (elseblock_opt.as_ref().unwrap().0 + 1) as i32
                            }
                            _ => panic!(),
                        }
                        elseblock_opt.unwrap().0 + 1
                    } else {
                        0
                    };
                    if let None = condition_opt {
                        return Fail(errors);
                    }
                    if let None = block_opt {
                        return Fail(errors);
                    }
                    break_positions.extend(&block_opt.as_ref().unwrap().1);
                    match &mut codes[first_jump_index] {
                        Code::JumpNot(pos) => {
                            *pos = (block_opt.as_ref().unwrap().0 + 1 + else_bonus) as i32
                        }
                        _ => panic!(),
                    }
                    (
                        1 + condition_opt.unwrap().0 + block_opt.as_ref().unwrap().0 + else_size,
                        break_positions,
                    )
                }
                Statement::Call { expression, args } => {
                    let results: Vec<Option<u32>> = args
                        .into_iter()
                        .map(|arg| {
                            match arg
                                .generate(codes, filename, source, code_pos_table)
                                .to_option(&mut errors)
                            {
                                Some((i, _)) => Some(i),
                                None => None,
                            }
                        })
                        .collect();
                    let expression_opt = expression
                        .generate(codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    let mut size = 0;
                    for res in results {
                        if let Some(l) = res {
                            size += l;
                        } else {
                            return Fail(errors);
                        }
                    }
                    if let None = expression_opt {
                        return Fail(errors);
                    }
                    code_pos_table.insert(codes.len(), self.start);
                    codes.push(Code::Call);
                    codes.push(Code::Pop);
                    (2 + expression_opt.unwrap().0 + size, Vec::new())
                }
                Statement::Block { statements } => attempt!(
                    generate_block(statements, codes, filename, source, code_pos_table),
                    errors
                ),
                Statement::Assign { left, right } => {
                    let var_name = match left.inner {
                        Expression::Variable { name } => Some(String::from(name)),
                        _ => None,
                    };
                    let left_opt = left
                        .generate_lhs(codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    let right_opt = right
                        .generate(codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    if let None = left_opt {
                        return Fail(errors);
                    }
                    if let None = right_opt {
                        return Fail(errors);
                    }
                    if let Some(name) = var_name {
                        codes.push(Code::Store(name));
                    } else {
                        codes.push(Code::Put);
                    }
                    (1 + left_opt.unwrap() + right_opt.unwrap().0, Vec::new())
                }
                Statement::While { condition, block } => {
                    let condition_opt = condition
                        .generate(codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    let first_jump_index = codes.len();
                    codes.push(Code::JumpNot(0));
                    let block_opt = generate_block(block, codes, filename, source, code_pos_table)
                        .to_option(&mut errors);
                    if let None = condition_opt {
                        return Fail(errors);
                    }
                    if let None = block_opt {
                        return Fail(errors);
                    }
                    codes.push(Code::Goto(
                        -((block_opt.as_ref().unwrap().0 + condition_opt.as_ref().unwrap().0 + 1)
                            as i32),
                    ));
                    match &mut codes[first_jump_index] {
                        Code::JumpNot(pos) => *pos = (block_opt.as_ref().unwrap().0 + 2) as i32,
                        _ => panic!(),
                    }
                    for position in &block_opt.as_ref().unwrap().1 {
                        match &mut codes[*position] {
                            Code::Goto(pos) if *pos == 0 => {
                                *pos = block_opt.as_ref().unwrap().0 as i32
                                    - (position - first_jump_index) as i32
                                    + 2
                            }
                            Code::Goto(pos) if *pos == 1 => {
                                *pos = -((position - first_jump_index) as i32
                                    + condition_opt.as_ref().unwrap().0 as i32)
                            }
                            _ => panic!(),
                        }
                    }
                    (
                        2 + block_opt.unwrap().0 + condition_opt.unwrap().0,
                        Vec::new(),
                    )
                }
                Statement::Break => {
                    let pos = codes.len();
                    code_pos_table.insert(pos, self.start);
                    codes.push(Code::Goto(0));
                    (1, vec![pos])
                }
                Statement::Continue => {
                    let pos = codes.len();
                    code_pos_table.insert(pos, self.start);
                    codes.push(Code::Goto(1));
                    (1, vec![pos])
                }
            },
            errors,
        )
    }

    fn generate_lhs(
        self,
        _codes: &mut Vec<Code>,
        _filename: &str,
        _source: &str,
        _code_pos_table: &mut HashMap<usize, usize>,
    ) -> Mistake<u32, OliveError> {
        panic!()
    }
}

fn generate_block(
    block: Vec<Located<Statement>>,
    codes: &mut Vec<Code>,
    filename: &str,
    source: &str,
    code_pos_table: &mut HashMap<usize, usize>,
) -> Mistake<(u32, Vec<usize>), OliveError> {
    let mut break_positions = Vec::new();
    let mut errors = Vec::new();
    let mut fine = true;
    let mut size = 0;
    for st in block {
        let st_opt = st
            .generate(codes, filename, source, code_pos_table)
            .to_option(&mut errors);
        if let Some((l, break_pos)) = st_opt {
            size += l;
            break_positions.extend(break_pos);
        } else {
            fine = false;
        }
    }
    if fine {
        Fine((size, break_positions), errors)
    } else {
        Fail(errors)
    }
}

pub fn generate_codes<'a>(
    tree: Vec<Located<Statement<'a>>>,
    filename: &str,
    source: &str,
) -> Mistake<(Vec<Code>, HashMap<usize, usize>), OliveError> {
    let mut code_pos_table = HashMap::new();
    let mut errors = Vec::new();
    let mut codes: Vec<Code> = Vec::new();
    let (total_len, break_positions) = attempt!(
        generate_block(tree, &mut codes, filename, source, &mut code_pos_table),
        errors
    );
    assert_eq!(codes.len() as u32, total_len);
    codes.push(Code::PushNone);
    codes.push(Code::Return);
    for bp in &break_positions {
        errors.push(OliveError::new_code_error(
            *code_pos_table.get(bp).unwrap(),
            filename,
            source,
            OliveCodeError::BreakOutsideWhile,
        ));
    }
    if break_positions.len() != 0 {
        return Fail(errors);
    }
    Fine((codes, code_pos_table), errors)
}
