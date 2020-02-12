pub mod util {
    use std::fmt::{Debug, Display};
    use std::fmt::{Error, Formatter};

    #[derive(Debug)]
    pub struct TextPos {
        line: usize,
        col: usize,
    }

    impl Display for TextPos {
        fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
            write!(f, "ln {} col {}", self.line, self.col)
        }
    }

    pub fn get_text_pos(position: usize) -> TextPos {
        // TODO
        return TextPos {
            line: 0,
            col: position,
        };
    }
}

pub mod lexer {
    use crate::parser::util;
    use crate::parser::util::TextPos;
    use std::error::Error;
    use std::fmt;
    use std::fmt::Formatter;
    use std::fmt::{Debug, Display};
    use std::iter::{Enumerate, Peekable};
    use std::str::Chars;

    #[derive(Debug)]
    pub enum LexerError {
        EOF,
        NoToken(TextPos, char),
        NumberFormat(TextPos, String),
        InvalidEscape(TextPos, String),
    }

    impl Error for LexerError {}

    impl Display for LexerError {
        fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
            match self {
                LexerError::EOF => write!(f, "Reached end of file"),
                LexerError::NoToken(pos, err) => {
                    write!(f, "invalid token found at {}: {}", pos, err)
                }
                LexerError::NumberFormat(pos, err) => {
                    write!(f, "number format error at {}: {}", pos, err)
                }
                LexerError::InvalidEscape(pos, err) => {
                    write!(f, "invalid escape character at {}: {}", pos, err)
                }
            }
        }
    }

    #[derive(Debug)]
    pub enum Token {
        Ident(usize, String),
        Fun(usize),
        If(usize),
        Else(usize),
        Elif(usize),
        While(usize),
        Continue(usize),
        Break(usize),
        For(usize),
        In(usize),
        Return(usize),
        BoolAnd(usize),
        BoolOr(usize),
        ValTrue(usize),
        ValFalse(usize),
        ValNone(usize),
        ValFloat(usize, f64),
        ValInt(usize, u64),
        ValString(usize, String),
        LPar(usize),
        RPar(usize),
        LBrack(usize),
        RBrack(usize),
        LBrace(usize),
        RBrace(usize),
        Semi(usize),
        Comma(usize),
        Add(usize),
        Minus(usize),
        Mul(usize),
        Mod(usize),
        BitOr(usize),
        BitXOr(usize),
        BitAnd(usize),
        BitLsh(usize),
        BitRsh(usize),
        BitURsh(usize),
        Concat(usize),
        IntDiv(usize),
        FloatDiv(usize),
        Assign(usize),
        Equals(usize),
        NotEquals(usize),
        BoolNot(usize),
        LessThan(usize),
        LessEquals(usize),
        GreaterThan(usize),
        GreaterEquals(usize),
    }

    fn get_char(iterator: &mut Peekable<Enumerate<Chars>>) -> Result<(usize, char), LexerError> {
        match iterator.peek() {
            Some(x) => Ok(*x),
            None => Err(LexerError::EOF),
        }
    }

    fn get_keyword_or_ident_token(
        iterator: &mut Peekable<Enumerate<Chars>>,
    ) -> Result<Token, LexerError> {
        let position = get_char(iterator)?.0;
        let mut current_token: String = String::new();
        loop {
            let next = get_char(iterator)?.1;
            if !(next.is_alphabetic() || next.is_digit(10) || next == '_') {
                break;
            }
            current_token.push(next);
            iterator.next();
        }
        Ok(match current_token.as_str() {
            "fun" => Token::Fun(position),
            "if" => Token::If(position),
            "else" => Token::Else(position),
            "elif" => Token::Elif(position),
            "while" => Token::While(position),
            "continue" => Token::Continue(position),
            "break" => Token::Break(position),
            "for" => Token::For(position),
            "in" => Token::In(position),
            "return" => Token::Return(position),
            "true" => Token::ValTrue(position),
            "false" => Token::ValFalse(position),
            "none" => Token::ValNone(position),
            "and" => Token::BoolAnd(position),
            "or" => Token::BoolOr(position),
            _ => Token::Ident(position, current_token),
        })
    }

    fn get_number_token(iterator: &mut Peekable<Enumerate<Chars>>) -> Result<Token, LexerError> {
        let position = get_char(iterator)?.0;
        let mut current_token: String = String::new();
        loop {
            let next = get_char(iterator)?.1;
            if !(next.is_digit(10) || next == '.') {
                break;
            }
            current_token.push(next);
            iterator.next();
        }
        Ok(if current_token.contains('.') {
            Token::ValFloat(
                position,
                current_token.parse::<f64>().map_err(|_| {
                    LexerError::NumberFormat(util::get_text_pos(position), current_token)
                })?,
            )
        } else {
            Token::ValInt(
                position,
                current_token.parse::<u64>().map_err(|_| {
                    LexerError::NumberFormat(util::get_text_pos(position), current_token)
                })?,
            )
        })
    }

    fn get_string_token(iterator: &mut Peekable<Enumerate<Chars>>) -> Result<Token, LexerError> {
        let position = get_char(iterator)?.0;
        iterator.next();
        let mut current_token: String = String::new();
        loop {
            let next = get_char(iterator)?.1;
            let next_char = if next == '\\' {
                iterator.next();
                let escaped = get_char(iterator)?.1;
                match escaped {
                    '\\' => '\\',
                    '"' => '"',
                    'n' => '\n',
                    'r' => '\r',
                    _ => {
                        return Err(LexerError::InvalidEscape(
                            util::get_text_pos(position + current_token.len() + 1),
                            next.to_string() + &escaped.to_string(),
                        ));
                    }
                }
            } else if next == '"' {
                iterator.next();
                break;
            } else {
                next
            };
            current_token.push(next_char);
            iterator.next();
        }
        Ok(Token::ValString(position, current_token))
    }

    pub fn get_token(iterator: &mut Peekable<Enumerate<Chars>>) -> Result<Token, LexerError> {
        let (position, mut next) = get_char(iterator)?;
        if next.is_whitespace() {
            loop {
                next = get_char(iterator)?.1;
                if !next.is_whitespace() {
                    break;
                }
                iterator.next();
            }
            get_token(iterator)
        } else {
            if next.is_alphabetic() {
                get_keyword_or_ident_token(iterator)
            } else if next.is_digit(10) {
                get_number_token(iterator)
            } else if next == '"' {
                get_string_token(iterator)
            } else {
                iterator.next();
                match next {
                    '(' => Ok(Token::LPar(position)),
                    ')' => Ok(Token::RPar(position)),
                    '[' => Ok(Token::LBrack(position)),
                    ']' => Ok(Token::RBrack(position)),
                    '{' => Ok(Token::LBrace(position)),
                    '}' => Ok(Token::RBrace(position)),
                    ';' => Ok(Token::Semi(position)),
                    ',' => Ok(Token::Comma(position)),
                    '+' => Ok(Token::Add(position)),
                    '-' => Ok(Token::Minus(position)),
                    '*' => Ok(Token::Mul(position)),
                    '%' => Ok(Token::Mod(position)),
                    '|' => Ok(Token::BitOr(position)),
                    '^' => Ok(Token::BitXOr(position)),
                    '&' => Ok(Token::BitAnd(position)),
                    '$' => Ok(Token::Concat(position)),
                    '/' => Ok(if get_char(iterator)?.1 == '/' {
                        iterator.next();
                        Token::IntDiv(position)
                    } else {
                        Token::FloatDiv(position)
                    }),
                    '=' => Ok(if get_char(iterator)?.1 == '=' {
                        iterator.next();
                        Token::Equals(position)
                    } else {
                        Token::Assign(position)
                    }),
                    '!' => Ok(if get_char(iterator)?.1 == '=' {
                        iterator.next();
                        Token::NotEquals(position)
                    } else {
                        Token::BoolNot(position)
                    }),
                    '<' => Ok(if get_char(iterator)?.1 == '=' {
                        iterator.next();
                        Token::LessEquals(position)
                    } else if get_char(iterator)?.1 == '<' {
                        iterator.next();
                        Token::BitLsh(position)
                    } else {
                        Token::LessThan(position)
                    }),
                    '>' => Ok(if get_char(iterator)?.1 == '=' {
                        iterator.next();
                        Token::GreaterEquals(position)
                    } else if get_char(iterator)?.1 == '>' {
                        iterator.next();
                        if get_char(iterator)?.1 == '>' {
                            iterator.next();
                            Token::BitURsh(position)
                        } else {
                            Token::BitRsh(position)
                        }
                    } else {
                        Token::GreaterThan(position)
                    }),
                    _ => Err(LexerError::NoToken(util::get_text_pos(position), next)),
                }
            }
        }
    }
}
