use crate::parser::parser::Statement;

#[derive(Debug, Clone)]
pub enum Code {
    Push,
}

pub fn generate(block: Statement) -> Vec<Code> {
    Vec::new()
}

#[cfg(test)]
mod test {
    #[test]
    fn test_generate() {
        unimplemented!();
    }
}
