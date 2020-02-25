mod codegen;
mod parser;

use codegen::Code;
use indexmap::IndexSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Result as IoResult;
use std::io::Write;
use std::path::Path;

fn write_to_file(pathname: &String, constants: &IndexSet<String>, codes: &Vec<u8>) -> IoResult<()> {
    let outpath = Path::new(pathname);
    let mut outfile = File::create(&outpath)?;

    outfile.write_all(&Code::u32_to_bytes(0xBAFADACE))?;
    outfile.write_all(&Code::u16_to_bytes(constants.len() as u16))?;
    for constant in constants {
        outfile.write_all(&Code::u16_to_bytes(constant.len() as u16))?;
        outfile.write_all(constant.as_bytes())?;
    }
    outfile.write_all(&Code::u32_to_bytes(codes.len() as u32))?;
    outfile.write_all(&codes)?;
    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let path = Path::new(args[1].as_str());
        env::set_current_dir(path.parent().unwrap().join(".")).map_err(|err| format!("{}", err))?;
        let outpath = format!("{}.olvc", path.file_stem().unwrap().to_str().unwrap());
        let contents: String = fs::read_to_string(path.file_name().unwrap()).map_err(|err| format!("{}", err))?;

        let block = parser::parser::parse(&contents).map_err(|err| format!("{}", err))?;
        let mut constants = IndexSet::new();
        let codes = codegen::to_bytes(
            codegen::generate(block, &mut constants).map_err(|err| format!("{}", err))?,
            &mut constants,
        );

        write_to_file(&outpath, &constants, &codes).map_err(|err| format!("{}", err))?;
        Ok(())
    } else {
        Err(String::from("argument required"))
    }
}
