mod parser;
mod codegen;

use std::env;
use std::fs;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let contents: String =
            fs::read_to_string(args[1].as_str()).map_err(|err| format!("{}", err))?;
        let block = parser::parser::parse(&contents).map_err(|err| format!("{}", err))?;
        println!("{:?}", block);
        let codes = codegen::generate(block).map_err(|err| format!("{}", err))?;
        println!("{:?}", codes);
        Ok(())
    } else {
        Err(String::from("argument required"))
    }
}
