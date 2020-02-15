mod parser;

use std::fs;

fn main() -> Result<(), String> {
    let contents: String =
        fs::read_to_string("examples/test.olv").map_err(|err| format!("{}", err))?;
    let block = parser::parser::parse(&contents).map_err(|err| format!("{}", err))?;
    println!("{:?}", block);
    Ok(())
}
