mod codegen;
mod interpreter;
mod parser;

use indexmap::IndexSet;
use interpreter::{bytes_to_u16, bytes_to_u32};
use std::convert::TryInto;
use std::env;
use std::fs;

fn read_from_bytes(bytes: Vec<u8>) -> Result<(Vec<u8>, Vec<String>), String> {
    let mut current = 4;
    let constantlen = bytes_to_u16(bytes[current..current + 2].try_into().expect(""));
    current += 2;
    let mut constants = Vec::with_capacity(constantlen as usize);
    for _ in 0..constantlen {
        let strlen = bytes_to_u16(bytes[current..current + 2].try_into().expect(""));
        current += 2;
        constants.push(
            std::str::from_utf8(&bytes[current..current + strlen as usize])
                .map_err(|err| format!("{}", err))?
                .to_string(),
        );
        current += strlen as usize;
    }
    let codelen = bytes_to_u32(bytes[current..current + 4].try_into().expect(""));
    current += 4;
    let codes: Vec<u8> = (&bytes[current..current + codelen as usize]).to_vec();
    Ok((codes, constants))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let bytes = fs::read(args[1].as_str()).map_err(|err| format!("{}", err))?;
        if &bytes[..4] == [0xCE, 0xDA, 0xFA, 0xBA] {
            let (codes, constants) = read_from_bytes(bytes)?;
            interpreter::run(codes, constants).map_err(|err| format!("{}", err))?;
        } else {
            let contents: String =
                String::from(std::str::from_utf8(&bytes).map_err(|err| format!("{}", err))?);
            let block = parser::parser::parse(&contents).map_err(|err| format!("{}", err))?;
            let mut constants = IndexSet::new();
            let codes =
                codegen::generate(block, &mut constants).map_err(|err| format!("{}", err))?;

            interpreter::run(
                codegen::to_bytes(codes, &mut constants),
                constants.iter().map(|s| s.clone()).collect(),
            )
            .map_err(|err| format!("{}", err))?;
        }
        Ok(())
    } else {
        Err(String::from("argument required"))
    }
}
