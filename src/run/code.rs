use super::value::Value;
use crate::scan::ast::{BinaryOperator, UnaryOperator};
use std::{fmt::Display, rc::Rc};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ByteCode {
    #[default]
    None,

    Jump {
        addr: usize,
    },
    JumpIf {
        negative: bool,
        cond: Source,
        addr: usize,
    },
    JumpIfSome {
        negative: bool,
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
impl Display for BinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperation::Add => write!(f, "+"),
            BinaryOperation::Sub => write!(f, "-"),
            BinaryOperation::Mul => write!(f, "*"),
            BinaryOperation::Div => write!(f, "/"),
            BinaryOperation::Mod => write!(f, "%"),
            BinaryOperation::Pow => write!(f, "^"),
            BinaryOperation::EE => write!(f, "=="),
            BinaryOperation::NE => write!(f, "!="),
            BinaryOperation::LT => write!(f, "<"),
            BinaryOperation::GT => write!(f, ">"),
            BinaryOperation::LE => write!(f, "<="),
            BinaryOperation::GE => write!(f, ">="),
            BinaryOperation::And => write!(f, "and"),
            BinaryOperation::Or => write!(f, "or"),
            BinaryOperation::Is => write!(f, "is"),
            BinaryOperation::In => write!(f, "in"),
            BinaryOperation::As => write!(f, "as"),
        }
    }
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
impl Display for UnaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperation::Neg => write!(f, "-"),
            UnaryOperation::Not => write!(f, "not"),
        }
    }
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
    pub path: Option<String>,
    pub name: Option<String>,
    pub code: Vec<ByteCode>,
    pub lines: Vec<usize>,
    pub parameters: u8,
    pub registers: u8,
    pub varargs: bool,
    pub closures: Vec<Rc<Closure>>,
    pub constants: Vec<Value>,
}

impl Display for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "  path: {}",
            self.path.clone().unwrap_or("?".to_string())
        )?;
        writeln!(f, "  registers: {}", self.registers)?;
        writeln!(f, "  parameters: {}", self.parameters)?;
        writeln!(f, "  varargs: {}", self.varargs)?;
        writeln!(f, "  code:")?;
        let width = 30;
        for ((addr, bytecode), line) in self.code.iter().enumerate().zip(self.lines.iter()) {
            let s = bytecode.to_string();
            writeln!(
                f,
                "    [{addr:04}] {s}{}({})",
                " ".repeat(width - s.len()),
                line + 1
            )?;
        }
        writeln!(f, "  constants:")?;
        for (addr, value) in self.constants.iter().enumerate() {
            writeln!(f, "    [{addr}] {value:?}")?;
        }
        writeln!(f, "  closures:")?;
        for (addr, closure) in self.closures.iter().enumerate() {
            writeln!(f, "    [{addr}] {:08x?}", Rc::as_ptr(closure))?;
        }

        for closure in self.closures.iter() {
            write!(
                f,
                "<{}{}:{:08x?}>:\n{closure}",
                if let Some(path) = &closure.path {
                    path
                } else {
                    ""
                },
                if let Some(name) = &closure.name {
                    if closure.path.is_some() {
                        format!(":{name}")
                    } else {
                        name.clone()
                    }
                } else {
                    "".into()
                },
                Rc::as_ptr(closure)
            )?;
        }
        Ok(())
    }
}
impl Display for ByteCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ByteCode::None => write!(f, "none"),
            ByteCode::Jump { addr } => write!(f, "jump [{addr:04}]"),
            ByteCode::JumpIf {
                negative: false,
                cond,
                addr,
            } => write!(f, "jumpif     {cond} [{addr:04}]"),
            ByteCode::JumpIf {
                negative: true,
                cond,
                addr,
            } => write!(f, "jumpif not {cond} [{addr:04}]"),
            ByteCode::JumpIfSome {
                negative: false,
                src,
                addr,
            } => write!(f, "jumpifsome {src} [{addr:04}]"),
            ByteCode::JumpIfSome {
                negative: true,
                src,
                addr,
            } => write!(f, "jumpifnone {src} [{addr:04}]"),
            ByteCode::Call {
                dst: None,
                func,
                start,
                amount,
            } => write!(f, "call       {func} ({start}..{})", start + amount - 1),
            ByteCode::Call {
                dst: Some(dst),
                func,
                start,
                amount,
            } => write!(
                f,
                "call       {func} ({start}..{}) -> {dst}",
                start + amount - 1
            ),
            ByteCode::Return { src: None } => write!(f, "return"),
            ByteCode::Return { src: Some(src) } => write!(f, "return     {src}"),
            ByteCode::Move { dst, src } => write!(f, "move       {dst} = {src}"),
            ByteCode::Field { dst, head, field } => {
                write!(f, "field      {dst} = {head} . {field}")
            }
            ByteCode::SetField { head, field, src } => {
                write!(f, "setfield   {head} . {field} = {src}")
            }
            ByteCode::Vector { dst, start, amount } => {
                write!(f, "vec        {start}..{} -> {dst}", start + amount - 1)
            }
            ByteCode::Tuple { dst, start, amount } => {
                write!(f, "tuple      {start}..{} -> {dst}", start + amount - 1)
            }
            ByteCode::Map { dst } => write!(f, "map        {dst}"),
            ByteCode::Fn { dst, addr } => write!(f, "fn         {dst} = c#{addr}"),
            ByteCode::Binary {
                op,
                dst,
                left,
                right,
            } => write!(f, "binary     {dst} = {left} {op} {right}"),
            ByteCode::Unary { op, dst, right } => write!(f, "unary     {dst} = {op} {right}"),
        }
    }
}
impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Null => write!(f, "null"),
            Source::Bool(v) => write!(f, "{v:?}"),
            Source::Char(v) => write!(f, "{v:?}"),
            Source::Int(v) => write!(f, "{v:?}"),
            Source::Float(v) => write!(f, "{v:?}"),
            Source::Register(reg) => write!(f, "@{reg}"),
            Source::Global(addr) => write!(f, "g#{addr}"),
            Source::Constant(addr) => write!(f, "#{addr}"),
        }
    }
}
impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::Register(reg) => write!(f, "!{reg}"),
            Location::Global(addr) => write!(f, "!g#{addr}"),
        }
    }
}
