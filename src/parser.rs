pub mod util {
    use std::error::Error;
    use std::fmt;
    use std::fmt::Formatter;
    use std::fmt::{Debug, Display};
    #[derive(Debug)]
    pub enum ParserError {
        EOF,
        NoToken(String, char),
        NumberFormat(String, String),
        InvalidEscape(String, String),
        UnexpectedToken(String, String, String),
        NotAccepted,
    }

    impl Error for ParserError {}

    impl Display for ParserError {
        fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
            match self {
                ParserError::EOF => write!(f, "Reached end of file"),
                ParserError::NotAccepted => write!(f, "statement or expression not accepted"),
                ParserError::NoToken(pos, err) => {
                    write!(f, "invalid token found at {}: {}", pos, err)
                }
                ParserError::NumberFormat(pos, err) => {
                    write!(f, "number format error at {}: {}", pos, err)
                }
                ParserError::InvalidEscape(pos, err) => {
                    write!(f, "invalid escape character at {}: {}", pos, err)
                }
                ParserError::UnexpectedToken(pos, exp, err) => write!(
                    f,
                    "unexpected token at {}, expected {} got: {}",
                    pos, exp, err
                ),
            }
        }
    }

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
    use super::util;
    use super::util::ParserError;
    use std::cmp::PartialEq;
    use std::iter::{Enumerate, Peekable};
    use std::str::Chars;

    #[derive(Debug, PartialEq, Clone)]
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

    impl Token {
        pub fn get_position(&self) -> usize {
            match *self {
                Token::Ident(pos, _) => pos,
                Token::Fun(pos) => pos,
                Token::If(pos) => pos,
                Token::Else(pos) => pos,
                Token::Elif(pos) => pos,
                Token::While(pos) => pos,
                Token::Continue(pos) => pos,
                Token::Break(pos) => pos,
                Token::For(pos) => pos,
                Token::In(pos) => pos,
                Token::Return(pos) => pos,
                Token::BoolAnd(pos) => pos,
                Token::BoolOr(pos) => pos,
                Token::ValTrue(pos) => pos,
                Token::ValFalse(pos) => pos,
                Token::ValNone(pos) => pos,
                Token::ValFloat(pos, _) => pos,
                Token::ValInt(pos, _) => pos,
                Token::ValString(pos, _) => pos,
                Token::LPar(pos) => pos,
                Token::RPar(pos) => pos,
                Token::LBrack(pos) => pos,
                Token::RBrack(pos) => pos,
                Token::LBrace(pos) => pos,
                Token::RBrace(pos) => pos,
                Token::Semi(pos) => pos,
                Token::Comma(pos) => pos,
                Token::Add(pos) => pos,
                Token::Minus(pos) => pos,
                Token::Mul(pos) => pos,
                Token::Mod(pos) => pos,
                Token::BitOr(pos) => pos,
                Token::BitXOr(pos) => pos,
                Token::BitAnd(pos) => pos,
                Token::BitLsh(pos) => pos,
                Token::BitRsh(pos) => pos,
                Token::BitURsh(pos) => pos,
                Token::Concat(pos) => pos,
                Token::IntDiv(pos) => pos,
                Token::FloatDiv(pos) => pos,
                Token::Assign(pos) => pos,
                Token::Equals(pos) => pos,
                Token::NotEquals(pos) => pos,
                Token::BoolNot(pos) => pos,
                Token::LessThan(pos) => pos,
                Token::LessEquals(pos) => pos,
                Token::GreaterThan(pos) => pos,
                Token::GreaterEquals(pos) => pos,
            }
        }
    }

    fn get_char(iterator: &mut Peekable<Enumerate<Chars>>) -> Result<(usize, char), ParserError> {
        match iterator.peek() {
            Some(x) => Ok(*x),
            None => Err(ParserError::EOF),
        }
    }

    fn get_keyword_or_ident_token(
        iterator: &mut Peekable<Enumerate<Chars>>,
    ) -> Result<Token, ParserError> {
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

    fn get_number_token(
        iterator: &mut Peekable<Enumerate<Chars>>,
        text: &String,
    ) -> Result<Token, ParserError> {
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
                current_token.parse::<f64>().map_err(|_| {
                    ParserError::NumberFormat(util::get_text_pos(position, text), current_token)
                })?,
            )
        } else {
            Token::ValInt(
                position,
                current_token.parse::<u64>().map_err(|_| {
                    ParserError::NumberFormat(util::get_text_pos(position, text), current_token)
                })?,
            )
        })
    }

    fn get_string_token(
        iterator: &mut Peekable<Enumerate<Chars>>,
        text: &String,
    ) -> Result<Token, ParserError> {
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
                        return Err(ParserError::InvalidEscape(
                            util::get_text_pos(position + current_token.len() + 1, text),
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

    pub fn get_token(
        iterator: &mut Peekable<Enumerate<Chars>>,
        text: &String,
    ) -> Result<Token, ParserError> {
        let (position, mut next) = get_char(iterator)?;
        if next.is_whitespace() {
            loop {
                next = get_char(iterator)?.1;
                if !next.is_whitespace() {
                    break;
                }
                iterator.next();
            }
            get_token(iterator, text)
        } else if next == '#' {
            iterator.next();
            let multiline = get_char(iterator)?.1 == '#';
            if multiline {
                iterator.next();
            }
            loop {
                next = get_char(iterator)?.1;
                if multiline {
                    if next == '#' {
                        break;
                    }
                } else {
                    if next == '\n' {
                        break;
                    }
                }
                iterator.next();
            }
            get_token(iterator, text)
        } else {
            if next.is_alphabetic() {
                get_keyword_or_ident_token(iterator)
            } else if next.is_digit(10) {
                get_number_token(iterator, text)
            } else if next == '"' {
                get_string_token(iterator, text)
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
                    _ => Err(ParserError::NoToken(
                        util::get_text_pos(position, text),
                        next,
                    )),
                }
            }
        }
    }
}

pub mod parser {
    use super::lexer;
    use super::lexer::Token;
    use super::util;
    use super::util::ParserError;
    use std::iter::{Enumerate, Peekable};
    use std::mem;
    use std::str::Chars;

    struct Parser<'a> {
        iterator: &'a mut Peekable<Enumerate<Chars<'a>>>,
        contents: &'a String,
        current: &'a mut Token,
    }

    trait AST {
        fn generate_source(&self) -> &str;
    }

    enum Expression {
        FuncDef(Vec<Token>, Statement),
    }

    enum Statement {
        Block,
    }

    impl AST for Token {
        fn generate_source(&self) -> &str {
            match self {
                Token::ValFalse(_) => "false",
                Token::Ident(_, name) => name.as_str(),
                _ => unimplemented!(),
            }
        }
    }

    impl AST for Expression {
        fn generate_source(&self) -> &str {
            match self {
                Expression::FuncDef(_, _) => "fun(",
                _ => unimplemented!(),
            }
        }
    }

    impl Parser<'_> {
        fn eat(&mut self) -> Result<(), ParserError> {
            *self.current = lexer::get_token(self.iterator, self.contents)?;
            Ok(())
        }

        fn peek(&self) -> Token {
            self.current.clone()
        }

        fn accept(&self, typetoken: &Token) -> bool {
            mem::discriminant(self.current) == mem::discriminant(typetoken)
        }

        fn expect(&self, typetoken: &Token) -> Result<(), ParserError> {
            if self.accept(typetoken) {
                Ok(())
            } else {
                let pos: String = util::get_text_pos(self.current.get_position(), self.contents);
                Err(ParserError::UnexpectedToken(
                    pos,
                    format!("{:?}", typetoken),
                    format!("{:?}", self.current),
                ))
            }
        }
    }

    fn parse_st_block(parser: &mut Parser) -> Result<Statement, ParserError> {
        Ok(Statement::Block)
    }

    fn parse_ex_func_def(parser: &mut Parser) -> Result<Expression, ParserError> {
        if parser.accept(&Token::Fun(0)) {
            parser.eat()?;
            parser.expect(&Token::LPar(0))?;
            parser.eat()?;
            let mut args: Vec<Token> = Vec::new();
            while parser.accept(&Token::Ident(0, String::new())) {
                let tok: Token = parser.peek();
                args.push(tok);
                parser.eat()?;
                if parser.accept(&Token::RPar(0)) {
                    break;
                }
                parser.expect(&Token::Comma(0))?;
                parser.eat()?;
            }
            parser.expect(&Token::RPar(0))?;
            parser.eat()?;
            let block = parse_st_block(parser)?;
            Ok(Expression::FuncDef(args, block))
        } else {
            Err(ParserError::NotAccepted)
        }
    }

    pub fn parse(contents: &String) -> Result<(), ParserError> {
        let mut iterator = contents.chars().enumerate().peekable();
        let mut token = lexer::get_token(&mut iterator, contents)?;
        let mut parser = Parser {
            iterator: &mut iterator,
            contents: contents,
            current: &mut token,
        };
        parse_ex_func_def(&mut parser)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::lexer;
    use super::lexer::Token;
    use super::util;
    use std::fs;
    use std::io;

    fn run_lexer(contents: &str) -> Token {
        let mut iterator = contents.chars().enumerate().peekable();

        match lexer::get_token(&mut iterator, &contents.to_string()) {
            Ok(t) => t,
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn test_util_get_text_pos() {
        let text: String = "fun main(\n) test123".to_string();
        assert_eq!(util::get_text_pos(15, &text), "ln 2 col 6".to_string());
    }

    #[test]
    fn test_lexer_examples_no_panic() -> Result<(), io::Error> {
        for ex in fs::read_dir("examples")? {
            let path = ex?.path();
            println!("{:?}", path);
            let contents = fs::read_to_string(path)?;
            run_lexer(contents.as_str());
        }
        Ok(())
    }

    #[test]
    fn test_lexer_tokens() {
        assert_eq!(
            run_lexer("ßuperĸööl"),
            Token::Ident(0, "ßuperĸööl".to_string()),
            "Ident"
        );
        assert_eq!(
            run_lexer("\"\\\"ĸthis\nis\r\nan interesting \\\"test\\\\ yeäöüöäöĸ\""),
            Token::ValString(
                0,
                "\"ĸthis\nis\r\nan interesting \"test\\ yeäöüöäöĸ".to_string()
            ),
            "ValString"
        );
        assert_eq!(run_lexer("7435971"), Token::ValInt(0, 7435971), "ValInt");
        assert_eq!(run_lexer("24.861"), Token::ValFloat(0, 24.861), "ValFloat");
        assert_eq!(run_lexer("fun"), Token::Fun(0), "Fun");
        assert_eq!(run_lexer("if"), Token::If(0), "If");
        assert_eq!(run_lexer("else"), Token::Else(0), "Else");
        assert_eq!(run_lexer("elif"), Token::Elif(0), "Elif");
        assert_eq!(run_lexer("while"), Token::While(0), "While");
        assert_eq!(run_lexer("continue"), Token::Continue(0), "Continue");
        assert_eq!(run_lexer("break"), Token::Break(0), "Break");
        assert_eq!(run_lexer("for"), Token::For(0), "For");
        assert_eq!(run_lexer("in"), Token::In(0), "In");
        assert_eq!(run_lexer("return"), Token::Return(0), "Return");
        assert_eq!(run_lexer("and"), Token::BoolAnd(0), "BoolAnd");
        assert_eq!(run_lexer("or"), Token::BoolOr(0), "BoolOr");
        assert_eq!(run_lexer("true"), Token::ValTrue(0), "ValTrue");
        assert_eq!(run_lexer("false"), Token::ValFalse(0), "ValFalse");
        assert_eq!(run_lexer("none"), Token::ValNone(0), "None");
        assert_eq!(run_lexer("("), Token::LPar(0), "LPar");
        assert_eq!(run_lexer(")"), Token::RPar(0), "RPar");
        assert_eq!(run_lexer("["), Token::LBrack(0), "LBrack");
        assert_eq!(run_lexer("]"), Token::RBrack(0), "RBrack");
        assert_eq!(run_lexer("{"), Token::LBrace(0), "LBrace");
        assert_eq!(run_lexer("}"), Token::RBrace(0), "RBrace");
        assert_eq!(run_lexer(";"), Token::Semi(0), "Semi");
        assert_eq!(run_lexer(","), Token::Comma(0), "Comma");
        assert_eq!(run_lexer("+"), Token::Add(0), "Add");
        assert_eq!(run_lexer("-"), Token::Minus(0), "Minus");
        assert_eq!(run_lexer("*"), Token::Mul(0), "Mul");
        assert_eq!(run_lexer("%"), Token::Mod(0), "Mod");
        assert_eq!(run_lexer("|"), Token::BitOr(0), "BitOr");
        assert_eq!(run_lexer("^"), Token::BitXOr(0), "BitXOr");
        assert_eq!(run_lexer("&"), Token::BitAnd(0), "BitAnd");
        assert_eq!(run_lexer("<<"), Token::BitLsh(0), "BitLsh");
        assert_eq!(run_lexer(">>"), Token::BitRsh(0), "BitRsh");
        assert_eq!(run_lexer(">>>"), Token::BitURsh(0), "BitURsh");
        assert_eq!(run_lexer("$"), Token::Concat(0), "Concat");
        assert_eq!(run_lexer("//"), Token::IntDiv(0), "IntDiv");
        assert_eq!(run_lexer("/"), Token::FloatDiv(0), "FloatDiv");
        assert_eq!(run_lexer("="), Token::Assign(0), "Assign");
        assert_eq!(run_lexer("=="), Token::Equals(0), "Equals");
        assert_eq!(run_lexer("!="), Token::NotEquals(0), "NotEquals");
        assert_eq!(run_lexer("!"), Token::BoolNot(0), "BoolNot");
        assert_eq!(run_lexer("<"), Token::LessThan(0), "LessThan");
        assert_eq!(run_lexer("<="), Token::LessEquals(0), "LessEquals");
        assert_eq!(run_lexer(">"), Token::GreaterThan(0), "GreaterThan");
        assert_eq!(run_lexer(">="), Token::GreaterEquals(0), "GreaterEquals");
    }
}
