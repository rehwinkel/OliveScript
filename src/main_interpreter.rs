mod parser;
mod codegen;
mod interpreter;

use std::env;
use std::path::Path;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let path = Path::new(args[1].as_str());
        let codes = interpreter::modules::read(path.to_path_buf())?;
        env::set_current_dir(path.parent().unwrap().join(".")).map_err(|err| format!("{}", err))?;
        interpreter::run(codes).map_err(|err| format!("{}", err))?;
        Ok(())
    } else {
        Err(String::from("argument required"))
    }
}
