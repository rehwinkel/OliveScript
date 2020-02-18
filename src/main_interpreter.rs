mod codegen;
mod interpreter;
mod parser;

use indexmap::IndexSet;
use std::env;
use std::fs;

fn main() -> Result<(), String> {
    //TODO olvc -> run, olv -> parse, run
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let contents: String =
            fs::read_to_string(args[1].as_str()).map_err(|err| format!("{}", err))?;

        let block = parser::parser::parse(&contents).map_err(|err| format!("{}", err))?;
        let mut constants = IndexSet::new();
        let codes = codegen::generate(block, &mut constants).map_err(|err| format!("{}", err))?;

        interpreter::run(&codes, &constants.iter().map(|s| s.clone()).collect());
        Ok(())
    } else {
        Err(String::from("argument required"))
    }
}
