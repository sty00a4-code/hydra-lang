use crate::scan::ast::{BinaryOperator, UnaryOperator};

use super::value::Value;
use std::rc::Rc;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ByteCode {
    #[default]
    None,

    Jump {
        addr: usize,
    },
    JumpIf {
        negativ: bool,
        cond: Source,
        addr: usize,
    },
    JumpIfSome {
        negativ: bool,
        src: Source,
        addr: usize,
    },

    Call {
        dst: Option<Location>,
        func: Source,
        start: u8,
        amount: u8,
    },
    Return {
        src: Option<Source>,
    },

    Move {
        dst: Location,
        src: Source,
    },
    Field {
        dst: Location,
        head: Source,
        field: Source,
    },
    SetField {
        head: Source,
        field: Source,
        src: Source,
    },

    Vector {
        dst: Location,
        start: u8,
        amount: u8,
    },
    Tuple {
        dst: Location,
        start: u8,
        amount: u8,
    },
    Map {
        dst: Location,
    },
    Fn {
        dst: Location,
        addr: u16,
    },

    Binary {
        op: BinaryOperation,
        dst: Location,
        left: Source,
        right: Source,
    },
    Unary {
        op: UnaryOperation,
        dst: Location,
        right: Source,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    EE,
    NE,
    LT,
    GT,
    LE,
    GE,
    And,
    Or,
    Is,
    In,
    As,
}
impl From<BinaryOperator> for BinaryOperation {
    fn from(value: BinaryOperator) -> Self {
        match value {
            BinaryOperator::Plus => Self::Add,
            BinaryOperator::Minus => Self::Sub,
            BinaryOperator::Star => Self::Mul,
            BinaryOperator::Slash => Self::Div,
            BinaryOperator::Percent => Self::Mod,
            BinaryOperator::Exponent => Self::Pow,
            BinaryOperator::EqualEqual => Self::EE,
            BinaryOperator::ExclamationEqual => Self::NE,
            BinaryOperator::Less => Self::LT,
            BinaryOperator::Greater => Self::GT,
            BinaryOperator::LessEqual => Self::LE,
            BinaryOperator::GreaterEqual => Self::GE,
            BinaryOperator::And => Self::And,
            BinaryOperator::Or => Self::Or,
            BinaryOperator::Is => Self::Is,
            BinaryOperator::In => Self::In,
            BinaryOperator::As => Self::As,
        }
    }
}
impl From<UnaryOperator> for UnaryOperation {
    fn from(value: UnaryOperator) -> Self {
        match value {
            UnaryOperator::Minus => Self::Neg,
            UnaryOperator::Not => Self::Not,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    Neg,
    Not,
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Source {
    #[default]
    Null,
    Bool(bool),
    Char(char),
    Int(i64),
    Float(f64),
    Register(u8),
    Global(u16),
    Constant(u16),
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Register(u8),
    Global(u16),
}
impl Location {
    pub fn eq_source(&self, other: &Source) -> bool {
        match (self, other) {
            (Self::Register(loc), Source::Register(src)) => loc == src,
            (Self::Global(loc), Source::Global(src)) => loc == src,
            _ => false,
        }
    }
}
impl From<Location> for Source {
    fn from(value: Location) -> Self {
        match value {
            Location::Register(v) => Self::Register(v),
            Location::Global(v) => Self::Global(v),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Closure {
    pub code: Vec<ByteCode>,
    pub lines: Vec<usize>,
    pub parameters: u8,
    pub registers: u8,
    pub varargs: bool,
    pub closures: Vec<Rc<Closure>>,
    pub constants: Vec<Value>,
}
