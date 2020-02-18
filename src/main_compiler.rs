mod codegen;
mod parser;

use codegen::Code;
use indexmap::IndexSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn write_to_file(
    pathname: &String,
    constants: &IndexSet<String>,
    codes: &Vec<u8>,
) -> Result<(), String> {
    let outpath = Path::new(pathname);
    let mut outfile = File::create(&outpath).map_err(|err| format!("{}", err))?;

    outfile
        .write_all(&Code::usize_to_bytes(constants.len()))
        .map_err(|err| format!("{}", err))?;
    for constant in constants {
        outfile
            .write_all(&Code::usize_to_bytes(constant.len()))
            .map_err(|err| format!("{}", err))?;
        outfile
            .write_all(constant.as_bytes())
            .map_err(|err| format!("{}", err))?;
    }
    outfile
        .write_all(&Code::usize_to_bytes(codes.len()))
        .map_err(|err| format!("{}", err))?;
    outfile
        .write_all(&codes)
        .map_err(|err| format!("{}", err))?;
    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let path = Path::new(args[1].as_str());
        let outpath = format!("{}c", path.file_name().unwrap().to_str().unwrap());
        let contents: String = fs::read_to_string(path).map_err(|err| format!("{}", err))?;

        let block = parser::parser::parse(&contents).map_err(|err| format!("{}", err))?;
        let mut constants = IndexSet::new();
        let codes = codegen::generate(block, &mut constants).map_err(|err| format!("{}", err))?;

        write_to_file(&outpath, &constants, &codes)?;
        Ok(())
    } else {
        Err(String::from("argument required"))
    }
}
