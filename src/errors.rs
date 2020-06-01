use colored::Colorize;
use oliveparser::{ParseError, Token};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub enum OliveIoError {
    OpenRead,
    OpenWrite,
    Read,
    Write,
    UTF,
    Serialize,
    Deserialize,
    Extension,
    CompileCompiled,
}

#[derive(Debug)]
pub enum OliveCodeError {
    Parse {
        found: String,
        expected: Vec<String>,
    },
    InvalidToken,
    ParseInteger {
        value: String,
    },
    ParseFloat {
        value: String,
    },
    Assign {
        expression_type: String,
    },
    Access,
    BreakOutsideWhile,
}

#[derive(Debug)]
pub enum OliveRuntimeError {
    IncorrectType { got: String, expected: Vec<String> },
    UnmatchingTypes { left: String, right: String },
    IndexOutOfBounds,
    CallArgs { expected: usize, got: usize },
    VariableNotFound { name: String },
}

#[derive(Debug)]
pub enum OliveError {
    Io {
        file: String,
        kind: OliveIoError,
    },
    Code {
        file: String,
        line: usize,
        col: usize,
        data: OliveCodeError,
    },
    Runtime {
        file: String,
        line: Option<usize>,
        col: Option<usize>,
        data: OliveRuntimeError,
    },
}

impl Display for OliveError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            OliveError::Code {
                line,
                col,
                file,
                data,
            } => {
                let message = match data {
                    OliveCodeError::Parse { found, expected } => format!(
                        "got unexpected token '{}', expected one of [{}]",
                        found,
                        expected
                            .iter()
                            .map(|s| format!("'{}'", &s[1..s.len() - 1]))
                            .collect::<Vec<String>>()
                            .join(", ")
                    ),
                    OliveCodeError::InvalidToken => format!("invalid token (probalby unclosed multi-line comment"),
                    OliveCodeError::ParseInteger { value } => format!(
                        "couldn't convert literal '{}' to integer (might be too large)",
                        value
                    ),
                    OliveCodeError::ParseFloat { value } => format!(
                        "couldn't convert literal '{}' to float (might be too large)",
                        value
                    ),
                    OliveCodeError::Access => {
                        String::from("can't use access operator with any right hand expression (must be identifier)")
                    }
                    OliveCodeError::Assign {expression_type} => {
                        format!("can't use '{}' as left hand of assignment", expression_type)
                    }
                    OliveCodeError::BreakOutsideWhile => String::from("tried to break or continue outside of a while loop")
                };
                write!(
                    f,
                    "{} {} {}",
                    "error".red().bold(),
                    format!("(in '{}'):", file).bold(),
                    format!("at ln {} col {}: {}", line, col, message)
                )
            }
            OliveError::Runtime {
                line,
                col,
                file,
                data,
            } => {
                let message = match data {
                    OliveRuntimeError::IncorrectType { expected, got } => {
                        if expected.len() == 1 {
                            format!("expected type {}, got type {}", &expected[0], got)
                        } else {
                            format!(
                                "expected one of types [{}], got type {}",
                                expected.join(", "),
                                got
                            )
                        }
                    }
                    OliveRuntimeError::UnmatchingTypes { left, right } => format!(
                        "operation not supported for type {} and type {}",
                        left, right
                    ),
                    OliveRuntimeError::IndexOutOfBounds => {
                        String::from("index not found in object")
                    }
                    OliveRuntimeError::VariableNotFound { name } => {
                        format!("couldn't find variable '{}' in scope", name)
                    }
                    OliveRuntimeError::CallArgs { expected, got } => format!(
                        "expected {} arguments to function call, got {}",
                        expected, got
                    ),
                };
                write!(
                    f,
                    "{} {} {}",
                    "error".red().bold(),
                    format!("(in '{}'):", file).bold(),
                    if let Some(line) = line {
                        format!("at ln {} col {}: {}", line, col.unwrap(), message)
                    } else {
                        message
                    }
                )
            }
            OliveError::Io { kind, file } => {
                let message: &str = match kind {
                    OliveIoError::OpenRead => {
                        "failed to open file for reading (file might not exist)"
                    }
                    OliveIoError::OpenWrite => "failed to open file for writing",
                    OliveIoError::Read => "failed to read from file",
                    OliveIoError::Write => "failed to write to file",
                    OliveIoError::UTF => "failed to convert file to utf-8",
                    OliveIoError::Serialize => "failed to serialize codes",
                    OliveIoError::Deserialize => "failed to deserialize file",
                    OliveIoError::Extension => "unrecognized file extension",
                    OliveIoError::CompileCompiled => "tried to compile binary file (.olvc)",
                };
                write!(
                    f,
                    "{} {} {}",
                    "error".red().bold(),
                    format!("(in '{}'):", file).bold(),
                    message
                )
            }
        }
    }
}

impl OliveError {
    //TODO
    fn get_line_and_column(start: usize, source: &str) -> (usize, usize) {
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(
                source
                    .char_indices()
                    .take(start)
                    .filter(|(_, ch)| *ch == '\n')
                    .map(|t| t.0),
            )
            .collect();
        let line_start = line_starts.last().unwrap();
        (line_starts.len(), 1 + start - line_start)
    }

    pub fn new_code_error(
        start: usize,
        filename: &str,
        source: &str,
        data: OliveCodeError,
    ) -> Self {
        let (line, col) = OliveError::get_line_and_column(start, source);
        OliveError::Code {
            line,
            col,
            file: String::from(filename),
            data,
        }
    }

    pub fn new_runtime_error(
        start: Option<usize>,
        filename: &str,
        source: &str,
        data: OliveRuntimeError,
    ) -> Self {
        if let Some(start) = start {
            let (line, col) = OliveError::get_line_and_column(start, source);
            OliveError::Runtime {
                line: Some(line),
                col: Some(col),
                file: String::from(filename),
                data,
            }
        } else {
            OliveError::Runtime {
                line: None,
                col: None,
                file: String::from(filename),
                data,
            }
        }
    }

    pub fn from_parse_err(
        err: ParseError<usize, Token<'_>, &str>,
        file: &str,
        source: &str,
    ) -> Self {
        match err {
            ParseError::UnrecognizedToken { token, expected } => OliveError::new_code_error(
                token.0,
                file,
                source,
                OliveCodeError::Parse {
                    found: String::from((token.1).1),
                    expected,
                },
            ),
            ParseError::InvalidToken { location } => {
                OliveError::new_code_error(location, file, source, OliveCodeError::InvalidToken)
            }
            _ => unimplemented!("{:?}", err),
        }
    }
}
