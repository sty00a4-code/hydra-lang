use super::position::Located;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Chunk {
    pub stats: Vec<Located<Statement>>,
}
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Block {
    pub stats: Vec<Located<Statement>>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    LetBinding {
        param: Located<Parameter>,
        expr: Located<Expression>,
    },
    Assign {
        path: Located<Path>,
        expr: Located<Expression>,
    },
    Fn {
        name: Located<String>,
        params: Vec<Located<Parameter>>,
        varargs: Option<Located<String>>,
        body: Located<Block>,
    },
    Call {
        head: Located<Path>,
        args: Vec<Located<Expression>>,
    },
    SelfCall {
        head: Located<Path>,
        field: Located<String>,
        args: Vec<Located<Expression>>,
    },
    Return(Option<Located<Expression>>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Exponent,
    EqualEqual,
    ExclamationEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Is,
    In,
    As,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Minus,
    Not,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Atom(Atom),
    Call {
        head: Box<Located<Self>>,
        args: Vec<Located<Expression>>,
    },
    SelfCall {
        head: Box<Located<Self>>,
        field: Located<String>,
        args: Vec<Located<Expression>>,
    },
    Binary {
        op: BinaryOperator,
        left: Box<Located<Self>>,
        right: Box<Located<Self>>,
    },
    Unary {
        op: UnaryOperator,
        right: Box<Located<Self>>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Path(Path),
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Tuple(Vec<Located<Expression>>),
    Vector(Vec<Located<Expression>>),
    Map(Vec<(Located<String>, Located<Expression>)>),
    Expression(Box<Located<Expression>>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Path {
    Ident(String),
    Field {
        head: Box<Located<Self>>,
        field: Located<String>,
    },
    Index {
        head: Box<Located<Self>>,
        index: Box<Located<Expression>>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Parameter {
    Ident(String),
    Tuple(Vec<Located<Self>>),
    Vector(Vec<Located<Self>>),
    Map(Vec<(Located<String>, Option<Located<Self>>)>),
}
