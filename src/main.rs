use clap::{App, Arg};
use oliveparser::parse;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[macro_use]
extern crate mistake;
use mistake::Mistake::{self, Fail, Fine};

mod codegen;
mod errors;
mod interpreter;
use errors::{OliveError, OliveIoError};

fn get_codes(
    contents: Vec<u8>,
    compile: bool,
    in_path_str: &str,
) -> Mistake<
    (
        bool,
        Vec<codegen::Code>,
        HashMap<usize, usize>,
        Option<String>,
    ),
    OliveError,
> {
    let mut errors = Vec::new();
    let in_path = Path::new(in_path_str);
    match in_path.extension() {
        Some(x) if x == "olv" => {
            let str_contents: &str = attempt_res!(
                std::str::from_utf8(&contents).map_err(|_| OliveError::Io {
                    file: String::from(in_path_str),
                    kind: OliveIoError::UTF,
                }),
                errors
            );
            let ast = parse(str_contents);
            let (codes, code_pos) = attempt!(
                codegen::generate_codes(
                    attempt_res!(
                        ast.map_err(|err| OliveError::from_parse_err(
                            err,
                            in_path_str,
                            str_contents
                        )),
                        errors
                    ),
                    in_path_str,
                    str_contents
                ),
                errors
            );
            Fine(
                (
                    !compile,
                    vec![
                        codegen::Code::PushFun(Vec::new(), codes),
                        codegen::Code::Call,
                        codegen::Code::Return,
                    ],
                    code_pos,
                    Some(String::from(str_contents)),
                ),
                errors,
            )
        }
        Some(x) if x == "olvc" => {
            if !compile {
                let codes = attempt_res!(
                    bincode::deserialize(&contents).map_err(|_| {
                        OliveError::Io {
                            file: String::from(in_path_str),
                            kind: OliveIoError::Deserialize,
                        }
                    }),
                    errors
                );
                Fine((true, codes, HashMap::new(), None), errors)
            } else {
                errors.push(OliveError::Io {
                    kind: OliveIoError::CompileCompiled,
                    file: String::from(in_path_str),
                });
                Fail(errors)
            }
        }
        _ => {
            errors.push(OliveError::Io {
                file: String::from(in_path_str),
                kind: OliveIoError::Extension,
            });
            Fail(errors)
        }
    }
}

fn run<'a>() -> Mistake<(), OliveError> {
    let mut errors = Vec::new();
    let matches = App::new("olv")
        .about("OliveScript interpreter and compiler")
        .author("Ian Rehwinkel")
        .version("0.2.0")
        .arg(Arg::with_name("INPUT").required(true))
        .arg(
            Arg::with_name("compile")
                .short("c")
                .long("compile")
                .help("produce binary instead of running file"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .value_name("output")
                .long("output")
                .help("output file path"),
        )
        .get_matches();
    let in_path_str: &str = matches.value_of("INPUT").unwrap();
    let in_path = Path::new(in_path_str);
    let mut file = attempt_res!(
        File::open(in_path).map_err(|_| OliveError::Io {
            file: String::from(in_path_str),
            kind: OliveIoError::OpenRead,
        }),
        errors
    );
    let mut contents: Vec<u8> = Vec::new();
    attempt_res!(
        file.read_to_end(&mut contents).map_err(|_| OliveError::Io {
            file: String::from(in_path_str),
            kind: OliveIoError::Read,
        }),
        errors
    );
    let (should_run, codes, code_pos_table, source) = attempt!(
        get_codes(contents, matches.is_present("compile"), in_path_str),
        errors
    );
    if should_run {
        attempt!(
            interpreter::start(&codes, &code_pos_table, in_path_str, source.as_deref()),
            errors
        );
    } else {
        let out_path = match matches.value_of("output") {
            Some(val) => val.to_string(),
            None => format!(
                "{}c",
                Path::new(in_path_str)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
            ),
        };
        let mut out_file = attempt_res!(
            File::create(&out_path).map_err(|_| OliveError::Io {
                file: String::from(&out_path),
                kind: OliveIoError::OpenWrite,
            }),
            errors
        );
        attempt_res!(
            out_file
                .write(&attempt_res!(
                    bincode::serialize(&codes).map_err(|_| OliveError::Io {
                        file: String::from(&out_path),
                        kind: OliveIoError::Serialize,
                    }),
                    errors
                ))
                .map_err(|_| OliveError::Io {
                    file: String::from(&out_path),
                    kind: OliveIoError::Write,
                }),
            errors
        );
    }
    Fine((), errors)
}

fn main() {
    match run() {
        Fine(_, errors) => {
            for err in errors {
                println!("{}", err);
            }
        }
        Fail(errors) => {
            for err in errors {
                println!("{}", err);
            }
        }
    }
}
