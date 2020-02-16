pub mod util {
    use std::error::Error;
    use std::fmt::Display;
    use std::fmt::Error as FmtError;
    use std::fmt::Formatter;
    #[derive(Debug)]
    pub enum ParserError {
        EOF,
        NoToken(String, char),
        NumberFormat(String, String),
        InvalidEscape(String, String),
        UnexpectedToken(String, String, String),
        NotAccepted(String, String),
        UnmatchedPar,
        TooMuchOutput,
        InvalidValue,
        InvalidExpression,
    }

    impl Error for ParserError {}

    impl Display for ParserError {
        fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
            match self {
                ParserError::EOF => write!(f, "reached end of file"),
                ParserError::NotAccepted(msg, pos) => write!(f, "not accepted at {}: {}", pos, msg),
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
                ParserError::UnmatchedPar => write!(f, "unmatched parenthesis"),
                ParserError::TooMuchOutput => write!(f, "too many expressions on output stack"),
                ParserError::InvalidValue => write!(f, "invalid value"),
                ParserError::InvalidExpression => write!(f, "invalid expression"),
            }
        }
    }

    pub fn get_text_pos(position: usize, text: &str) -> String {
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

    #[cfg(test)]
    mod test {
        use super::get_text_pos;
        #[test]
        fn test_util_get_text_pos() {
            let text: String = "fun main(\n) test123".to_string();
            assert_eq!(get_text_pos(15, &text), "ln 2 col 6".to_string());
        }
    }
}

pub mod lexer {
    use super::util;
    use super::util::ParserError;
    use std::iter::{Enumerate, Peekable};
    use std::str::Chars;

    #[derive(Debug, PartialEq, Clone)]
    pub enum Token {
        // statements
        EOF,
        If(usize),
        Else(usize),
        While(usize),
        Continue(usize),
        Break(usize),
        Return(usize),
        //For(usize),
        //In(usize),
        // values
        New(usize),
        Fun(usize),
        Ident(usize, String),
        ValTrue(usize),
        ValFalse(usize),
        ValNone(usize),
        ValFloat(usize, f64),
        ValInt(usize, u64),
        ValString(usize, String),
        // punctuation
        LPar(usize),
        RPar(usize),
        LBrack(usize),
        RBrack(usize),
        LBrace(usize),
        RBrace(usize),
        Semi(usize),
        Comma(usize),
        Colon(usize),
        // binary/unary operators
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
        IntDiv(usize),
        FloatDiv(usize),
        Concat(usize),
        Assign(usize),
        Equals(usize),
        NotEquals(usize),
        BoolNot(usize),
        LessThan(usize),
        LessEquals(usize),
        GreaterThan(usize),
        GreaterEquals(usize),
        BoolAnd(usize),
        BoolOr(usize),
        Get(usize),
    }

    impl Token {
        pub fn get_position(&self) -> usize {
            match *self {
                Token::EOF => 0,
                Token::Ident(pos, _) => pos,
                Token::Fun(pos) => pos,
                Token::If(pos) => pos,
                Token::Else(pos) => pos,
                Token::While(pos) => pos,
                Token::Continue(pos) => pos,
                Token::Break(pos) => pos,
                //Token::For(pos) => pos,
                //Token::In(pos) => pos,
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
                Token::Colon(pos) => pos,
                Token::New(pos) => pos,
                Token::Get(pos) => pos,
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
            "while" => Token::While(position),
            "continue" => Token::Continue(position),
            "break" => Token::Break(position),
            //"for" => Token::For(position),
            //"in" => Token::In(position),
            "return" => Token::Return(position),
            "true" => Token::ValTrue(position),
            "false" => Token::ValFalse(position),
            "none" => Token::ValNone(position),
            "and" => Token::BoolAnd(position),
            "or" => Token::BoolOr(position),
            "new" => Token::New(position),
            _ => Token::Ident(position, current_token),
        })
    }

    fn get_number_token(
        iterator: &mut Peekable<Enumerate<Chars>>,
        text: &str,
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
        text: &str,
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

    pub fn get_token_eof(
        iterator: &mut Peekable<Enumerate<Chars>>,
        text: &str,
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
            get_token_eof(iterator, text)
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
            get_token_eof(iterator, text)
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
                    ':' => Ok(Token::Colon(position)),
                    '.' => Ok(Token::Get(position)),
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

    pub fn get_token(
        iterator: &mut Peekable<Enumerate<Chars>>,
        text: &str,
    ) -> Result<Token, ParserError> {
        match get_token_eof(iterator, text) {
            Ok(tk) => Ok(tk),
            Err(err) => {
                if let ParserError::EOF = err {
                    Ok(Token::EOF)
                } else {
                    Err(err)
                }
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::get_token;
        use super::Token;
        use std::fs;
        use std::io;

        fn run_lexer(contents: &str) -> Token {
            let mut iterator = contents.chars().enumerate().peekable();

            match get_token(&mut iterator, &contents.to_string()) {
                Ok(t) => t,
                Err(err) => panic!("{}", err),
            }
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
            assert_eq!(run_lexer("while"), Token::While(0), "While");
            assert_eq!(run_lexer("continue"), Token::Continue(0), "Continue");
            assert_eq!(run_lexer("break"), Token::Break(0), "Break");
            //assert_eq!(run_lexer("elif"), Token::Elif(0), "Elif");
            //assert_eq!(run_lexer("for"), Token::For(0), "For");
            //assert_eq!(run_lexer("in"), Token::In(0), "In");
            assert_eq!(run_lexer("return"), Token::Return(0), "Return");
            assert_eq!(run_lexer("and"), Token::BoolAnd(0), "BoolAnd");
            assert_eq!(run_lexer("or"), Token::BoolOr(0), "BoolOr");
            assert_eq!(run_lexer("true"), Token::ValTrue(0), "ValTrue");
            assert_eq!(run_lexer("false"), Token::ValFalse(0), "ValFalse");
            assert_eq!(run_lexer("none"), Token::ValNone(0), "None");
            assert_eq!(run_lexer("new"), Token::New(0), "New");
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
            assert_eq!(run_lexer(":"), Token::Colon(0), "Colon");
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
        contents: &'a str,
        current: &'a mut Token,
    }

    #[derive(Debug, Clone)]
    pub struct BendyPair {
        pub identifier: String,
        pub value: Expression,
    }

    #[derive(Debug, Clone)]
    pub enum Expression {
        NewFunc(Vec<Token>, Box<Statement>),
        NewList(Vec<Expression>),
        NewBendy(Vec<BendyPair>),
        Value(Token),
        Operator(Operator),
        Binary(Box<Expression>, Box<Expression>, Operator),
        Call(Box<Expression>, Vec<Expression>),
        Unary(Box<Expression>, Operator),
    }

    #[derive(Debug, Clone)]
    pub enum Statement {
        If(Box<Expression>, Box<Statement>, Option<Box<Statement>>),
        While(Box<Expression>, Box<Statement>),
        Block(Vec<Statement>),
        Expression(Box<Expression>),
        Return(Box<Expression>),
        Continue,
        Break,
    }

    #[derive(Debug, Clone)]
    pub enum Operator {
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
        Equals,
        NotEquals,
        LessThan,
        LessEquals,
        GreaterThan,
        GreaterEquals,
        BoolNot,
        BoolAnd,
        BoolOr,
        Concat,
        Assign,
        Get,
        LPar,
        RPar,
        ParGet,
        Call,
    }

    impl Operator {
        fn is_binary(&self) -> bool {
            match self {
                Operator::Add
                | Operator::Sub
                | Operator::IntDiv
                | Operator::FloatDiv
                | Operator::Mul
                | Operator::Mod
                | Operator::BitOr
                | Operator::BitXOr
                | Operator::BitAnd
                | Operator::BitLsh
                | Operator::BitRsh
                | Operator::BitURsh
                | Operator::Concat
                | Operator::BoolAnd
                | Operator::BoolOr
                | Operator::Assign
                | Operator::Equals
                | Operator::NotEquals
                | Operator::LessEquals
                | Operator::GreaterEquals
                | Operator::LessThan
                | Operator::GreaterThan
                | Operator::Get
                | Operator::LPar
                | Operator::ParGet
                | Operator::RPar
                | Operator::Call => true,
                Operator::Neg | Operator::BoolNot => false,
            }
        }

        fn precedence(&self) -> usize {
            match self {
                Operator::LPar | Operator::RPar | Operator::ParGet | Operator::Call => 0,
                Operator::Get => 1,
                Operator::Neg | Operator::BoolNot => 2,
                Operator::IntDiv | Operator::FloatDiv | Operator::Mul | Operator::Mod => 3,
                Operator::Add | Operator::Sub => 4,
                Operator::BitLsh | Operator::BitRsh | Operator::BitURsh => 5,
                Operator::LessEquals
                | Operator::LessThan
                | Operator::GreaterEquals
                | Operator::GreaterThan => 6,
                Operator::Concat => 7,
                Operator::Equals | Operator::NotEquals => 8,
                Operator::BitAnd => 9,
                Operator::BitXOr => 10,
                Operator::BitOr => 11,
                Operator::BoolAnd => 12,
                Operator::BoolOr => 13,
                Operator::Assign => 14,
            }
        }

        fn is_left_assoc(&self) -> bool {
            match self {
                Operator::Neg | Operator::BoolNot | Operator::Assign => false,
                _ => true,
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
                let err = ParserError::UnexpectedToken(
                    pos,
                    format!("{:?}", typetoken),
                    format!("{:?}", self.current),
                );
                println!("{}", err);
                Err(err)
            }
        }
    }

    macro_rules! is_accepted {
        ($e: expr) => {
            match $e {
                Ok(ex) => Ok(Some(ex)),
                Err(err) => match err {
                    ParserError::NotAccepted(_, _) => Ok(None),
                    _ => Err(err),
                },
            }
        };
    }

    fn parse_ex_new_func(parser: &mut Parser) -> Result<Expression, ParserError> {
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
            let block = parse_st_block(parser, true)?;
            Ok(Expression::NewFunc(args, Box::from(block)))
        } else {
            Err(ParserError::NotAccepted(
                String::from("new_func"),
                util::get_text_pos(parser.current.get_position(), parser.contents),
            ))
        }
    }

    fn parse_ex_new_list_or_bendy(parser: &mut Parser) -> Result<Expression, ParserError> {
        if parser.accept(&Token::New(0)) {
            parser.eat()?;
            if parser.accept(&Token::LBrack(0)) {
                parser.eat()?;
                let mut exprs = Vec::new();
                while !parser.accept(&Token::RBrack(0)) {
                    exprs.push(parse_ex(parser)?);
                    if !parser.accept(&Token::Comma(0)) {
                        break;
                    } else {
                        parser.eat()?;
                    }
                }
                parser.expect(&Token::RBrack(0))?;
                parser.eat()?;
                Ok(Expression::NewList(exprs))
            } else {
                parser.expect(&Token::LBrace(0))?;
                parser.eat()?;
                let mut pairs = Vec::new();
                while !parser.accept(&Token::RBrace(0)) {
                    parser.expect(&Token::Ident(0, String::new()))?;
                    let name = parser.peek();
                    parser.eat()?;
                    parser.expect(&Token::Colon(0))?;
                    parser.eat()?;
                    let expr = parse_ex(parser)?;
                    pairs.push(BendyPair {
                        identifier: match name {
                            Token::Ident(_, s) => s,
                            _ => panic!(""),
                        },
                        value: expr,
                    });
                    if !parser.accept(&Token::Comma(0)) {
                        break;
                    } else {
                        parser.eat()?;
                    }
                }
                parser.expect(&Token::RBrace(0))?;
                parser.eat()?;
                Ok(Expression::NewBendy(pairs))
            }
        } else {
            Err(ParserError::NotAccepted(
                String::from("list or bendy"),
                util::get_text_pos(parser.current.get_position(), parser.contents),
            ))
        }
    }

    fn parse_ex_primary(parser: &mut Parser) -> Result<Expression, ParserError> {
        if let Some(ex) = is_accepted!(parse_ex_new_list_or_bendy(parser))? {
            Ok(ex)
        } else if let Some(ex) = is_accepted!(parse_ex_new_func(parser))? {
            Ok(ex)
        } else if parser.accept(&Token::ValInt(0, 0))
            || parser.accept(&Token::ValFloat(0, 0.0))
            || parser.accept(&Token::ValNone(0))
            || parser.accept(&Token::ValFalse(0))
            || parser.accept(&Token::ValTrue(0))
            || parser.accept(&Token::ValString(0, String::new()))
            || parser.accept(&Token::Ident(0, String::new()))
        {
            let tok = parser.peek();
            parser.eat()?;
            Ok(Expression::Value(tok))
        } else {
            Err(ParserError::NotAccepted(
                String::from("primary"),
                util::get_text_pos(parser.current.get_position(), parser.contents),
            ))
        }
    }

    fn parse_operator(
        parser: &mut Parser,
        previous: Option<Expression>,
    ) -> Result<Option<Operator>, ParserError> {
        let op = match parser.current {
            Token::Add(_) => Operator::Add,
            Token::Minus(_) => {
                if previous.is_none() {
                    Operator::Neg
                } else {
                    match previous.unwrap() {
                        Expression::Operator(op) => match op {
                            Operator::RPar => Operator::Sub,
                            _ => Operator::Neg,
                        },
                        _ => Operator::Sub,
                    }
                }
            }
            Token::Mul(_) => Operator::Mul,
            Token::IntDiv(_) => Operator::IntDiv,
            Token::FloatDiv(_) => Operator::FloatDiv,
            Token::Mod(_) => Operator::Mod,
            Token::BitOr(_) => Operator::BitOr,
            Token::BitXOr(_) => Operator::BitXOr,
            Token::BitAnd(_) => Operator::BitAnd,
            Token::BitLsh(_) => Operator::BitLsh,
            Token::BitRsh(_) => Operator::BitRsh,
            Token::BitURsh(_) => Operator::BitURsh,
            Token::BoolAnd(_) => Operator::BoolAnd,
            Token::BoolOr(_) => Operator::BoolOr,
            Token::BoolNot(_) => Operator::BoolNot,
            Token::Assign(_) => Operator::Assign,
            Token::LessEquals(_) => Operator::LessEquals,
            Token::LessThan(_) => Operator::LessThan,
            Token::GreaterEquals(_) => Operator::GreaterEquals,
            Token::GreaterThan(_) => Operator::GreaterThan,
            Token::Concat(_) => Operator::Concat,
            Token::Equals(_) => Operator::Equals,
            Token::NotEquals(_) => Operator::NotEquals,
            Token::Get(_) => Operator::Get,
            Token::LPar(_) => {
                if previous.is_none() {
                    Operator::LPar
                } else {
                    match previous.unwrap() {
                        Expression::Operator(op) => match op {
                            Operator::ParGet => Operator::Call,
                            Operator::Call => Operator::Call,
                            _ => Operator::LPar,
                        },
                        _ => Operator::Call,
                    }
                }
            }
            Token::RPar(_) => Operator::RPar,
            Token::LBrack(_) => Operator::ParGet,
            Token::RBrack(_) => return Ok(None),
            _ => {
                return Err(ParserError::NotAccepted(
                    String::from("operator"),
                    util::get_text_pos(parser.current.get_position(), parser.contents),
                ));
            }
        };
        parser.eat()?;
        Ok(Some(op))
    }

    fn parse_element(
        parser: &mut Parser,
        previous: Option<Expression>,
    ) -> Result<Option<Expression>, ParserError> {
        if let Ok(opopt) = parse_operator(parser, previous) {
            if let Some(op) = opopt {
                Ok(Some(Expression::Operator(op)))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(parse_ex_primary(parser)?))
        }
    }

    fn process_op(op: Operator, output: &mut Vec<Expression>) {
        if op.is_binary() {
            let rhs = output.pop().unwrap();
            let lhs = output.pop().unwrap();
            output.push(Expression::Binary(Box::from(lhs), Box::from(rhs), op));
        } else {
            let expr = output.pop().unwrap();
            output.push(Expression::Unary(Box::from(expr), op));
        }
    }

    fn parse_ex(parser: &mut Parser) -> Result<Expression, ParserError> {
        let mut output: Vec<Expression> = Vec::new();
        let mut opstack: Vec<Operator> = Vec::new();
        let mut previous: Option<Expression> = None;

        let mut open_pars: usize = 0;

        let el = parse_element(parser, previous)?.unwrap();
        previous = Some(el.clone());
        match el {
            Expression::Operator(op) => match op {
                Operator::LPar => {
                    open_pars += 1;
                    opstack.push(op)
                }
                _ => opstack.push(op),
            },
            _ => output.push(el),
        }

        while !(parser.accept(&Token::Semi(0))
            || parser.accept(&Token::Comma(0))
            || parser.accept(&Token::RBrace(0))
            || parser.accept(&Token::RBrack(0))
            || (parser.accept(&Token::RPar(0)) && open_pars == 0))
        {
            if let Some(el) = parse_element(parser, previous.clone())? {
                previous = Some(el.clone());
                let op: Operator = match el {
                    Expression::Operator(op) => op,
                    _ => {
                        output.push(el);
                        continue;
                    }
                };
                match op {
                    Operator::Neg | Operator::BoolNot => opstack.push(op),
                    Operator::ParGet => {
                        while !opstack.is_empty() {
                            process_op(opstack.pop().unwrap(), &mut output);
                        }
                        let rhs = parse_ex(parser)?;
                        parser.expect(&Token::RBrack(0))?;
                        parser.eat()?;
                        let lhs = output.pop().unwrap();
                        output.push(Expression::Binary(
                            Box::from(lhs),
                            Box::from(rhs),
                            Operator::Get,
                        ));
                    }
                    Operator::Call => {
                        while !opstack.is_empty() {
                            process_op(opstack.pop().unwrap(), &mut output);
                        }

                        let mut args = Vec::new();
                        if !parser.accept(&Token::RPar(0)) {
                            args.push(parse_ex(parser)?);
                            while parser.accept(&Token::Comma(0)) {
                                parser.eat()?;
                                args.push(parse_ex(parser)?);
                            }
                        }

                        parser.expect(&Token::RPar(0))?;
                        parser.eat()?;
                        let lhs = output.pop().unwrap();
                        output.push(Expression::Call(Box::from(lhs), args));
                    }
                    Operator::LPar => {
                        open_pars += 1;
                        opstack.push(op)
                    }
                    Operator::RPar => {
                        open_pars -= 1;
                        while !opstack.is_empty()
                            && mem::discriminant(opstack.last().unwrap())
                                != mem::discriminant(&Operator::LPar)
                        {
                            process_op(opstack.pop().unwrap(), &mut output);
                        }
                        if !opstack.is_empty()
                            && mem::discriminant(opstack.last().unwrap())
                                == mem::discriminant(&Operator::LPar)
                        {
                            opstack.pop();
                        } else {
                            return Err(ParserError::UnmatchedPar);
                        }
                    }
                    _ => {
                        while !opstack.is_empty()
                            && mem::discriminant(opstack.last().unwrap())
                                != mem::discriminant(&Operator::LPar)
                            && (opstack.last().unwrap().precedence() < op.precedence()
                                || (op.precedence() == opstack.last().unwrap().precedence()
                                    && op.is_left_assoc()))
                        {
                            process_op(opstack.pop().unwrap(), &mut output);
                        }
                        opstack.push(op);
                    }
                }
            } else {
                continue;
            }
        }

        while !opstack.is_empty() {
            let op: Operator = opstack.pop().unwrap();
            process_op(op, &mut output);
        }
        if output.len() == 1 {
            Ok(output[0].clone())
        } else {
            Err(ParserError::TooMuchOutput)
        }
    }

    fn parse_st_block(parser: &mut Parser, braces: bool) -> Result<Statement, ParserError> {
        if braces && !parser.accept(&Token::LBrace(0)) {
            return Err(ParserError::NotAccepted(
                String::from("block"),
                util::get_text_pos(parser.current.get_position(), parser.contents),
            ));
        }
        if braces {
            parser.expect(&Token::LBrace(0))?;
            parser.eat()?;
        }
        let mut statements = Vec::new();
        loop {
            if parser.accept(&Token::EOF) || parser.accept(&Token::RBrace(0)) {
                break;
            } else {
                statements.push(parse_st(parser)?);
            }
        }
        if braces {
            parser.expect(&Token::RBrace(0))?;
            parser.eat()?;
        }
        Ok(Statement::Block(statements))
    }

    fn parse_st(parser: &mut Parser) -> Result<Statement, ParserError> {
        if let Some(st) = is_accepted!(parse_st_block(parser, true))? {
            Ok(st)
        } else if parser.accept(&Token::Continue(0)) {
            parser.eat()?;
            parser.expect(&Token::Semi(0))?;
            parser.eat()?;
            Ok(Statement::Continue)
        } else if parser.accept(&Token::Break(0)) {
            parser.eat()?;
            parser.expect(&Token::Semi(0))?;
            parser.eat()?;
            Ok(Statement::Break)
        } else if parser.accept(&Token::Return(0)) {
            parser.eat()?;
            let value = parse_ex(parser)?;
            parser.expect(&Token::Semi(0))?;
            parser.eat()?;
            Ok(Statement::Return(Box::from(value)))
        } else if parser.accept(&Token::While(0)) {
            parser.eat()?;
            parser.expect(&Token::LPar(0))?;
            parser.eat()?;
            let condition = parse_ex(parser)?;
            parser.expect(&Token::RPar(0))?;
            parser.eat()?;
            let block = parse_st_block(parser, true)?;
            Ok(Statement::While(Box::from(condition), Box::from(block)))
        } else if parser.accept(&Token::If(0)) {
            parser.eat()?;
            parser.expect(&Token::LPar(0))?;
            parser.eat()?;
            let condition = parse_ex(parser)?;
            parser.expect(&Token::RPar(0))?;
            parser.eat()?;
            let block = parse_st_block(parser, true)?;
            Ok(if parser.accept(&Token::Else(0)) {
                parser.eat()?;
                let else_st = if parser.accept(&Token::If(0)) {
                    parse_st(parser)
                } else {
                    parse_st_block(parser, true)
                }?;
                Statement::If(
                    Box::from(condition),
                    Box::from(block),
                    Some(Box::from(else_st)),
                )
            } else {
                Statement::If(Box::from(condition), Box::from(block), None)
            })
        } else {
            let expr = parse_ex(parser)?;
            parser.expect(&Token::Semi(0))?;
            parser.eat()?;
            let valid = match &expr {
                Expression::Call(_, _) => true,
                Expression::Binary(_, _, op) => match op {
                    Operator::Assign => true,
                    _ => false,
                },
                _ => false,
            };
            if valid {
                Ok(Statement::Expression(Box::from(expr)))
            } else {
                Err(ParserError::InvalidExpression)
            }
        }
    }

    pub fn parse(contents: &str) -> Result<Statement, ParserError> {
        let mut iterator = contents.chars().enumerate().peekable();
        let mut token = lexer::get_token(&mut iterator, contents)?;
        let mut parser = Parser {
            iterator: &mut iterator,
            contents: contents,
            current: &mut token,
        };
        Ok(parse_st_block(&mut parser, false)?)
    }

    #[cfg(test)]
    mod test {
        use super::parse_ex_primary;
        use super::Expression;
        use super::Parser;
        use crate::parser::lexer;
        use crate::parser::util::ParserError;

        fn run_parser(
            contents: &str,
            fun: &dyn Fn(&mut Parser) -> Result<Expression, ParserError>,
        ) {
            let mut iterator = contents.chars().enumerate().peekable();
            let mut token =
                lexer::get_token(&mut iterator, contents).expect("couldnt read first token");
            let mut parser = Parser {
                iterator: &mut iterator,
                contents: contents,
                current: &mut token,
            };
            fun(&mut parser).expect("failed to parse");
        }

        #[test]
        fn test_parse_ex_value() {
            run_parser("3.1415926535", &parse_ex_primary);
            run_parser("50981237", &parse_ex_primary);
            run_parser("\"this is a test string \\n oh yeah\"", &parse_ex_primary);
            run_parser("true", &parse_ex_primary);
            run_parser("false", &parse_ex_primary);
            run_parser("none", &parse_ex_primary);
            run_parser("test", &parse_ex_primary);
            run_parser("bla", &parse_ex_primary);
            run_parser("öpöelßöüĸĸĸ", &parse_ex_primary);
            run_parser("fun(a){}", &parse_ex_primary);
            run_parser("fun(a432){}", &parse_ex_primary);
            run_parser("fun(a,e,r,e,re,e,){}", &parse_ex_primary);
            run_parser("new []", &parse_ex_primary);
            run_parser("new [a]", &parse_ex_primary);
            run_parser("new [a, \"te\", 342.537]", &parse_ex_primary);
            run_parser("new {}", &parse_ex_primary);
            run_parser("new {a:3}", &parse_ex_primary);
            run_parser("new {a:3,öäü:3453}", &parse_ex_primary);
        }
    }
}
