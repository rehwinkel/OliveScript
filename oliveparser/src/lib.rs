#[macro_use]
extern crate lalrpop_util;

pub mod ast;

pub use lalrpop_util::lexer::Token;
pub use lalrpop_util::ParseError;

lalrpop_mod!(pub olive);

pub fn parse<'a>(
    source: &'a str,
) -> Result<
    Vec<ast::Located<ast::Statement<'a>>>,
    lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token, &str>,
> {
    let parser = olive::FileParser::new();
    parser.parse(source)
}
