pub mod util {
    pub fn get_text_pos(position: usize, text: &String) -> String {
        let mut line = 1;
        let mut col = 1;
        let mut count = 0;
        for ch in text.chars() {
            if count == position {
                break;
            }
            if ch == '\n' {
                col = 1;
                line += 1;
            } else {
                col += 1;
            }
            count += 1;
        }
        format!("ln {} col {}", line, col)
    }
}

pub mod lexer {
    use std::cmp::PartialEq;
    use std::error::Error;
    use std::fmt;
    use std::fmt::Formatter;
    use std::fmt::{Debug, Display};
    use std::iter::{Enumerate, Peekable};
    use std::str::Chars;

    #[derive(Debug)]
    pub enum LexerError {
        EOF,
        NoToken(usize, char),
        NumberFormat(usize, String),
        InvalidEscape(usize, String),
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

    #[derive(Debug, PartialEq)]
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
            let next = match get_char(iterator) {
                Ok((_, ch)) => ch,
                Err(_) => {
                    break;
                }
            };
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
            let next = match get_char(iterator) {
                Ok((_, ch)) => ch,
                Err(_) => {
                    break;
                }
            };
            if !(next.is_digit(10) || next == '.') {
                break;
            }
            current_token.push(next);
            iterator.next();
        }
        Ok(if current_token.contains('.') {
            Token::ValFloat(
                position,
                current_token
                    .parse::<f64>()
                    .map_err(|_| LexerError::NumberFormat(position, current_token))?,
            )
        } else {
            Token::ValInt(
                position,
                current_token
                    .parse::<u64>()
                    .map_err(|_| LexerError::NumberFormat(position, current_token))?,
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
                            position + current_token.len() + 1,
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
                    '/' => Ok(
                        if match get_char(iterator) {
                            Ok((_, ch)) => ch,
                            Err(_) => {
                                return Ok(Token::FloatDiv(position));
                            }
                        } == '/'
                        {
                            iterator.next();
                            Token::IntDiv(position)
                        } else {
                            Token::FloatDiv(position)
                        },
                    ),
                    '=' => Ok(
                        if match get_char(iterator) {
                            Ok((_, ch)) => ch,
                            Err(_) => {
                                return Ok(Token::Assign(position));
                            }
                        } == '='
                        {
                            iterator.next();
                            Token::Equals(position)
                        } else {
                            Token::Assign(position)
                        },
                    ),
                    '!' => Ok(
                        if match get_char(iterator) {
                            Ok((_, ch)) => ch,
                            Err(_) => {
                                return Ok(Token::BoolNot(position));
                            }
                        } == '='
                        {
                            iterator.next();
                            Token::NotEquals(position)
                        } else {
                            Token::BoolNot(position)
                        },
                    ),
                    '<' => Ok(
                        if match get_char(iterator) {
                            Ok((_, ch)) => ch,
                            Err(_) => {
                                return Ok(Token::LessThan(position));
                            }
                        } == '='
                        {
                            iterator.next();
                            Token::LessEquals(position)
                        } else if get_char(iterator)?.1 == '<' {
                            iterator.next();
                            Token::BitLsh(position)
                        } else {
                            Token::LessThan(position)
                        },
                    ),
                    '>' => Ok(
                        if match get_char(iterator) {
                            Ok((_, ch)) => ch,
                            Err(_) => {
                                return Ok(Token::GreaterThan(position));
                            }
                        } == '='
                        {
                            iterator.next();
                            Token::GreaterEquals(position)
                        } else if match get_char(iterator) {
                            Ok((_, ch)) => ch,
                            Err(_) => {
                                return Ok(Token::GreaterThan(position));
                            }
                        } == '>'
                        {
                            iterator.next();
                            if match get_char(iterator) {
                                Ok((_, ch)) => ch,
                                Err(_) => {
                                    return Ok(Token::BitRsh(position));
                                }
                            } == '>'
                            {
                                iterator.next();
                                Token::BitURsh(position)
                            } else {
                                Token::BitRsh(position)
                            }
                        } else {
                            Token::GreaterThan(position)
                        },
                    ),
                    _ => Err(LexerError::NoToken(position, next)),
                }
            }
        }
    }
}

pub mod parser {}

#[cfg(test)]
mod tests {
    use super::lexer;
    use super::lexer::{LexerError, Token};
    use super::util;

    fn run_lexer(contents: &str) -> Option<Token> {
        let mut iterator = contents.chars().enumerate().peekable();

        match lexer::get_token(&mut iterator) {
            Ok(t) => Some(t),
            Err(err) => match err {
                LexerError::EOF => None,
                _ => {
                    panic!("{}", err);
                }
            },
        }
    }

    #[test]
    fn test_util_get_text_pos() {
        let text: String = "fun main(\n) test123".to_string();
        assert_eq!(util::get_text_pos(15, &text), "ln 2 col 6".to_string());
    }

    #[test]
    fn test_lexer_tokens() {
        let result = run_lexer("ßuperĸööl");
        assert!(
            result.unwrap() == Token::Ident(0, "ßuperĸööl".to_string()),
            "reading Ident token failed"
        );
        let result = run_lexer("24.861");
        assert!(
            result.unwrap() == Token::ValFloat(0, 24.861),
            "reading ValFloat token failed"
        );
        let result = run_lexer("74057135971");
        assert!(
            result.unwrap() == Token::ValInt(0, 74057135971),
            "reading ValInt token failed"
        );
        let result = run_lexer("\"\\\"ĸthis\nis\r\nan interesting \\\"test\\\\ yeäöüöäöĸ\"");
        assert_eq!(
            result.unwrap(),
            Token::ValString(
                0,
                "\"ĸthis\nis\r\nan interesting \"test\\ yeäöüöäöĸ".to_string()
            ),
            "reading ValString token failed"
        );
        let result = run_lexer("fun");
        assert!(result.unwrap() == Token::Fun(0), "reading Fun token failed");
        let result = run_lexer("if");
        assert!(result.unwrap() == Token::If(0), "reading If token failed");
        let result = run_lexer("else");
        assert!(
            result.unwrap() == Token::Else(0),
            "reading Else token failed"
        );
        let result = run_lexer("elif");
        assert!(
            result.unwrap() == Token::Elif(0),
            "reading Elif token failed"
        );
        let result = run_lexer("while");
        assert!(
            result.unwrap() == Token::While(0),
            "reading While token failed"
        );
        let result = run_lexer("continue");
        assert!(
            result.unwrap() == Token::Continue(0),
            "reading Continue token failed"
        );
        let result = run_lexer("break");
        assert!(
            result.unwrap() == Token::Break(0),
            "reading Break token failed"
        );
        let result = run_lexer("for");
        assert!(result.unwrap() == Token::For(0), "reading For token failed");
        let result = run_lexer("in");
        assert!(result.unwrap() == Token::In(0), "reading In token failed");
        let result = run_lexer("return");
        assert!(
            result.unwrap() == Token::Return(0),
            "reading Return token failed"
        );
        let result = run_lexer("and");
        assert!(
            result.unwrap() == Token::BoolAnd(0),
            "reading BoolAnd token failed"
        );
        let result = run_lexer("or");
        assert!(
            result.unwrap() == Token::BoolOr(0),
            "reading BoolOr token failed"
        );
        let result = run_lexer("true");
        assert!(
            result.unwrap() == Token::ValTrue(0),
            "reading ValTrue token failed"
        );
        let result = run_lexer("false");
        assert!(
            result.unwrap() == Token::ValFalse(0),
            "reading ValFalse token failed"
        );
        let result = run_lexer("none");
        assert!(
            result.unwrap() == Token::ValNone(0),
            "reading None token failed"
        );
        let result = run_lexer("(");
        assert!(
            result.unwrap() == Token::LPar(0),
            "reading LPar token failed"
        );
        let result = run_lexer(")");
        assert!(
            result.unwrap() == Token::RPar(0),
            "reading RPar token failed"
        );
        let result = run_lexer("[");
        assert!(
            result.unwrap() == Token::LBrack(0),
            "reading LBrack token failed"
        );
        let result = run_lexer("]");
        assert!(
            result.unwrap() == Token::RBrack(0),
            "reading RBrack token failed"
        );
        let result = run_lexer("{");
        assert!(
            result.unwrap() == Token::LBrace(0),
            "reading LBrace token failed"
        );
        let result = run_lexer("}");
        assert!(
            result.unwrap() == Token::RBrace(0),
            "reading RBrace token failed"
        );
        let result = run_lexer(";");
        assert!(
            result.unwrap() == Token::Semi(0),
            "reading Semi token failed"
        );
        let result = run_lexer(",");
        assert!(
            result.unwrap() == Token::Comma(0),
            "reading Comma token failed"
        );
        let result = run_lexer("+");
        assert!(result.unwrap() == Token::Add(0), "reading Add token failed");
        let result = run_lexer("-");
        assert!(
            result.unwrap() == Token::Minus(0),
            "reading Minus token failed"
        );
        let result = run_lexer("*");
        assert!(result.unwrap() == Token::Mul(0), "reading Mul token failed");
        let result = run_lexer("%");
        assert!(result.unwrap() == Token::Mod(0), "reading Mod token failed");
        let result = run_lexer("|");
        assert!(
            result.unwrap() == Token::BitOr(0),
            "reading BitOr token failed"
        );
        let result = run_lexer("^");
        assert!(
            result.unwrap() == Token::BitXOr(0),
            "reading BitXOr token failed"
        );
        let result = run_lexer("&");
        assert!(
            result.unwrap() == Token::BitAnd(0),
            "reading BitAnd token failed"
        );
        let result = run_lexer("<<");
        assert!(
            result.unwrap() == Token::BitLsh(0),
            "reading BitLsh token failed"
        );
        let result = run_lexer(">>");
        assert!(
            result.unwrap() == Token::BitRsh(0),
            "reading BitRsh token failed"
        );
        let result = run_lexer(">>>");
        assert!(
            result.unwrap() == Token::BitURsh(0),
            "reading BitURsh token failed"
        );
        let result = run_lexer("$");
        assert!(
            result.unwrap() == Token::Concat(0),
            "reading Concat token failed"
        );
        let result = run_lexer("//");
        assert!(
            result.unwrap() == Token::IntDiv(0),
            "reading IntDiv token failed"
        );
        let result = run_lexer("/");
        assert!(
            result.unwrap() == Token::FloatDiv(0),
            "reading FloatDiv token failed"
        );
        let result = run_lexer("=");
        assert!(
            result.unwrap() == Token::Assign(0),
            "reading Assign token failed"
        );
        let result = run_lexer("==");
        assert!(
            result.unwrap() == Token::Equals(0),
            "reading Equals token failed"
        );
        let result = run_lexer("!=");
        assert!(
            result.unwrap() == Token::NotEquals(0),
            "reading NotEquals token failed"
        );
        let result = run_lexer("!");
        assert!(
            result.unwrap() == Token::BoolNot(0),
            "reading BoolNot token failed"
        );
        let result = run_lexer("<");
        assert!(
            result.unwrap() == Token::LessThan(0),
            "reading LessThan token failed"
        );
        let result = run_lexer("<=");
        assert!(
            result.unwrap() == Token::LessEquals(0),
            "reading LessEquals token failed"
        );
        let result = run_lexer(">");
        assert!(
            result.unwrap() == Token::GreaterThan(0),
            "reading GreaterThan token failed"
        );
        let result = run_lexer(">=");
        assert!(
            result.unwrap() == Token::GreaterEquals(0),
            "reading GreaterEquals token failed"
        );
    }
}
