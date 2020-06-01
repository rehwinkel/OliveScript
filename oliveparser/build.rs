extern crate lalrpop;

use lalrpop::Configuration;

fn main() {
    Configuration::new().emit_whitespace(false).emit_report(true).process_current_dir().unwrap();
}