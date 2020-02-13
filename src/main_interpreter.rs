mod parser;

use std::fs;

fn main() -> Result<(), String> {
    let contents: String =
        fs::read_to_string("examples/hello_world.olv").map_err(|err| format!("{}", err))?;
    parser::parser::parse(&contents).map_err(|err| format!("{}", err))?;
    Ok(())
}