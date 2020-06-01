#[derive(Debug)]
pub enum Statement<'a> {
    Break,
    Continue,
    Return {
        value: Located<Expression<'a>>,
    },
    Block {
        statements: Vec<Located<Statement<'a>>>,
    },
    While {
        condition: Located<Expression<'a>>,
        block: Vec<Located<Statement<'a>>>,
    },
    If {
        condition: Located<Expression<'a>>,
        block: Vec<Located<Statement<'a>>>,
        elseblock: Option<Vec<Located<Statement<'a>>>>,
    },
    Assign {
        left: Box<Located<Expression<'a>>>,
        right: Box<Located<Expression<'a>>>,
    },
    Call {
        expression: Box<Located<Expression<'a>>>,
        args: Vec<Located<Expression<'a>>>,
    },
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    IntDiv,
    FloatDiv,
    Mod,
    BitLsh,
    BitRsh,
    BitAnd,
    BitOr,
    BitXOr,
    Equals,
    NotEquals,
    LessThan,
    LessEquals,
    GreaterThan,
    GreaterEquals,
    BoolAnd,
    BoolOr,
    Concat,
    Access,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Neg,
    BoolNot,
}

#[derive(Debug)]
pub enum Expression<'a> {
    List {
        elements: Vec<Located<Expression<'a>>>,
    },
    Bendy {
        elements: Vec<(Located<&'a str>, Located<Expression<'a>>)>,
    },
    Integer {
        value: &'a str,
    },
    Float {
        value: &'a str,
    },
    String {
        value: String,
    },
    Boolean {
        value: bool,
    },
    None,
    Variable {
        name: &'a str,
    },
    Binary {
        left: Box<Located<Expression<'a>>>,
        right: Box<Located<Expression<'a>>>,
        operator: BinaryOperator,
    },
    Unary {
        expression: Box<Located<Expression<'a>>>,
        operator: UnaryOperator,
    },
    Index {
        expression: Box<Located<Expression<'a>>>,
        index: Box<Located<Expression<'a>>>,
    },
    Call {
        expression: Box<Located<Expression<'a>>>,
        args: Vec<Located<Expression<'a>>>,
    },
    Function {
        parameters: Vec<Located<&'a str>>,
        block: Vec<Located<Statement<'a>>>,
    },
}

#[derive(Debug)]
pub struct Located<T> {
    pub start: usize,
    pub end: usize,
    pub inner: T,
}
