mod parser;

use parser::lexer;
use parser::lexer::LexerError;
use std::fs;

fn main() -> Result<(), String> {
    let contents: String =
        fs::read_to_string("examples/hello_world.olv").map_err(|err| format!("{}", err))?;
    let mut iterator = contents.chars().enumerate().peekable();
    loop {
        let tk = match lexer::get_token(&mut iterator) {
            Ok(t) => t,
            Err(err) => match err {
                LexerError::EOF => {
                    break;
                },
                _ => {
                    return Err(format!("{}", err));
                }
            },
        };
        println!("{:?}", tk);
    }
    Ok(())
}
