use super::{
    code::{BinaryOperation, ByteCode, Closure, Location, Source, UnaryOperation},
    value::{FnKind, Function, Pointer, Value},
};
use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    rc::Rc,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default)]
pub struct Interpreter {
    pub call_stack: Vec<CallFrame>,
    pub globals: HashMap<String, Pointer<Value>>,
}
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub idx: usize,
    pub closure: Rc<Closure>,
    pub stack: Vec<Pointer<Value>>,
    pub dst: Option<Location>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunTimeError {
    pub err: RunTimeErrorKind,
    pub ln: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub enum RunTimeErrorKind {
    IndexOutOfRange {
        index: i64,
        len: usize,
    },
    InvalidField {
        head: Type,
        field: Type,
    },
    InvalidFieldHead(Type),
    CannotCall(Type),
    IllegalBinaryOperation {
        op: BinaryOperation,
        left: Type,
        right: Type,
    },
    IllegalUnaryOperation {
        op: UnaryOperation,
        right: Type,
    },
}
pub type Type = &'static str;
impl Display for RunTimeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunTimeErrorKind::IndexOutOfRange { index, len } => {
                write!(f, "index {index} is out of range of {len}")
            }
            RunTimeErrorKind::InvalidField { head, field } => {
                write!(f, "invalid field operation on {head} with {field}")
            }
            RunTimeErrorKind::InvalidFieldHead(typ) => write!(f, "can't field into {typ}"),
            RunTimeErrorKind::CannotCall(typ) => write!(f, "can't call {typ}"),
            RunTimeErrorKind::IllegalBinaryOperation { op, left, right } => {
                write!(
                    f,
                    "illegal binary operation {:?} on {left} with {right}",
                    op.to_string()
                )
            }
            RunTimeErrorKind::IllegalUnaryOperation { op, right } => {
                write!(f, "illegal unary operation {:?} on {right}", op.to_string())
            }
        }
    }
}
impl Error for RunTimeErrorKind {}

impl Interpreter {
    pub fn call_frame(&self) -> Option<&CallFrame> {
        self.call_stack.last()
    }
    pub fn call_frame_mut(&mut self) -> Option<&mut CallFrame> {
        self.call_stack.last_mut()
    }
    pub fn source(&self, src: Source) -> Option<Value> {
        match src {
            Source::Null => Some(Value::Null),
            Source::Bool(v) => Some(Value::Bool(v)),
            Source::Char(v) => Some(Value::Char(v)),
            Source::Int(v) => Some(Value::Int(v)),
            Source::Float(v) => Some(Value::Float(v)),
            Source::Register(reg) => self
                .call_frame()?
                .stack
                .get(reg as usize)
                .map(|arc| arc.lock().unwrap().clone()),
            Source::Global(addr) => {
                let call_frame = self.call_frame()?;
                let Value::String(var) = call_frame.closure.constants.get(addr as usize)? else {
                    return None;
                };
                self.globals.get(var).map(|arc| arc.lock().unwrap().clone())
            }
            Source::Constant(addr) => self
                .call_frame()?
                .closure
                .constants
                .get(addr as usize)
                .cloned(),
        }
    }
    pub fn location(&mut self, dst: Location) -> Option<Pointer<Value>> {
        match dst {
            Location::Register(reg) => {
                let call_frame = self.call_frame()?;
                call_frame.stack.get(reg as usize).cloned()
            }
            Location::Global(addr) => {
                let Value::String(var) = self.call_frame()?.closure.constants.get(addr as usize).cloned()? else {
                    return None;
                };
                if let Some(value) = self.globals.get(&var).cloned() {
                    Some(value)
                } else {
                    self.globals
                        .insert(var.clone(), Arc::new(Mutex::new(Value::default())));
                    self.globals.get(&var).cloned()
                }
            }
        }
    }
    pub fn call(
        &mut self,
        Function { closure }: &Function,
        args: Vec<Value>,
        dst: Option<Location>,
    ) -> Result<(), RunTimeError> {
        let mut stack: Vec<Pointer<Value>> = Vec::with_capacity(closure.registers as usize);
        let mut args = args.into_iter();
        for _ in 0..=(closure.parameters - if closure.varargs { 1 } else { 0 }) {
            let arg = args.next().unwrap_or_default();
            stack.push(Arc::new(Mutex::new(arg)));
        }
        if closure.varargs {
            let mut values = vec![];
            for arg in args {
                values.push(arg);
            }
            stack.push(Arc::new(Mutex::new(Value::Vector(Arc::new(Mutex::new(
                values,
            ))))));
        }
        for _ in closure.parameters..=closure.registers {
            stack.push(Arc::new(Mutex::new(Default::default())));
        }
        let call_frame = CallFrame {
            idx: 0,
            closure: Rc::clone(closure),
            stack,
            dst,
        };
        self.call_stack.push(call_frame);
        Ok(())
    }
    pub fn return_call(&mut self, src: Source) -> Option<Value> {
        let return_value = self.source(src);
        let CallFrame { dst, .. } = self.call_stack.pop().unwrap();
        if let Some(dst) = dst {
            let value = return_value.unwrap_or_default();
            if let Some(dst_value) = self.location(dst) {
                *(dst_value.lock().unwrap()) = value;
            }
            None
        } else {
            return_value
        }
    }
    pub fn instr(&self) -> Option<ByteCode> {
        let call_frame = self.call_frame()?;
        self.call_frame()?.closure.code.get(call_frame.idx).copied()
    }
    pub fn ln(&self) -> Option<usize> {
        let call_frame = self.call_frame()?;
        call_frame.closure.lines.get(call_frame.idx).copied()
    }
    pub fn closure(&self, addr: u16) -> Option<&Rc<Closure>> {
        self.call_frame()?.closure.closures.get(addr as usize)
    }
    pub fn step(&mut self) -> Result<Option<Option<Value>>, RunTimeError> {
        let ln = self.ln().unwrap_or_default();
        let instr = self.instr().unwrap();
        self.call_frame_mut().unwrap().idx += 1;
        match instr {
            ByteCode::None => {}
            ByteCode::Jump { addr } => {
                self.call_frame_mut().unwrap().idx = addr;
            }
            ByteCode::JumpIf {
                negativ,
                cond,
                addr,
            } => {
                let cond = self.source(cond).unwrap_or_default();
                if bool::from(cond) && !negativ {
                    self.call_frame_mut().unwrap().idx = addr;
                }
            }
            ByteCode::JumpIfSome { negativ, src, addr } => {
                let src = self.source(src).unwrap_or_default();
                if src == Value::Null && !negativ {
                    self.call_frame_mut().unwrap().idx = addr;
                }
            }
            ByteCode::Call {
                dst,
                func,
                start,
                amount,
            } => {
                let func = self.source(func).unwrap();
                let mut args = Vec::with_capacity(amount as usize);
                for reg in start..(start + amount) {
                    args.push(self.source(Source::Register(reg)).unwrap());
                }
                match func {
                    Value::Fn(FnKind::Function(func)) => {
                        self.call(&func.lock().unwrap(), args, dst)?;
                    }
                    Value::Fn(FnKind::Native(_)) => todo!(),
                    value => {
                        return Err(RunTimeError {
                            err: RunTimeErrorKind::CannotCall(value.typ()),
                            ln,
                        })
                    }
                }
            }
            ByteCode::Return { src } => {
                return Ok(Some(self.return_call(src.unwrap_or_default())));
            }
            ByteCode::Move { dst, src } => {
                let dst = self.location(dst).unwrap();
                *dst.lock().unwrap() = self.source(src).unwrap();
            }
            ByteCode::Field { dst, head, field } => {
                let dst = self.location(dst).unwrap();
                let head = self.source(head).unwrap();
                let field = self.source(field).unwrap();
                *dst.lock().unwrap() = match head {
                    Value::String(string) => match field {
                        Value::Int(value) => if value <= -1 {
                            if (value.unsigned_abs() - 1) as usize > string.len() {
                                None
                            } else {
                                let index = string.len() - value.unsigned_abs() as usize;
                                string.get(index..=index)
                            }
                        } else {
                            let index = value.unsigned_abs() as usize;
                            string.get(index..=index)
                        }
                        .and_then(|s| s.chars().next())
                        .map(Value::Char)
                        .unwrap_or_default(),
                        field => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::InvalidField {
                                    head: Value::Vector(Default::default()).typ(),
                                    field: field.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    Value::Vector(arc) => match field {
                        Value::Int(value) => {
                            let values = arc.lock().unwrap();
                            if value <= -1 {
                                if (value.unsigned_abs() - 1) as usize > values.len() {
                                    None
                                } else {
                                    values.get(values.len() - value.unsigned_abs() as usize)
                                }
                            } else {
                                values.get(value.unsigned_abs() as usize)
                            }
                            .cloned()
                            .unwrap_or_default()
                        }
                        field => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::InvalidField {
                                    head: Value::Vector(Default::default()).typ(),
                                    field: field.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    Value::Tuple(arc) => match field {
                        Value::Int(value) => {
                            let values = arc.lock().unwrap();
                            if value <= -1 {
                                if (value.unsigned_abs() - 1) as usize > values.len() {
                                    None
                                } else {
                                    values.get(values.len() - value.unsigned_abs() as usize)
                                }
                            } else {
                                values.get(value.unsigned_abs() as usize)
                            }
                            .cloned()
                            .unwrap_or_default()
                        }
                        field => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::InvalidField {
                                    head: Value::Tuple(Arc::new(Mutex::new(Box::new([])))).typ(),
                                    field: field.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    Value::Map(arc) => match field {
                        Value::String(key) => {
                            let map = arc.lock().unwrap();
                            map.get(&key).cloned().unwrap_or_default()
                        }
                        field => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::InvalidField {
                                    head: Value::Map(Default::default()).typ(),
                                    field: field.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    head => {
                        return Err(RunTimeError {
                            err: RunTimeErrorKind::InvalidFieldHead(head.typ()),
                            ln,
                        })
                    }
                };
            }
            ByteCode::SetField { head, field, src } => {
                let head = self.source(head).unwrap();
                let field = self.source(field).unwrap();
                let src = self.source(src).unwrap();
                match head {
                    Value::Vector(arc) => match field {
                        Value::Int(value) => {
                            let len = arc.lock().unwrap().len();
                            let mut values = arc.lock().unwrap();
                            let dst = if value <= -1 {
                                if (value.unsigned_abs() - 1) as usize > len {
                                    None
                                } else {
                                    values.get_mut(len - value.unsigned_abs() as usize)
                                }
                            } else {
                                values.get_mut(value.unsigned_abs() as usize)
                            }
                            .ok_or(RunTimeError {
                                err: RunTimeErrorKind::IndexOutOfRange { index: value, len },
                                ln,
                            })?;
                            *dst = src;
                        }
                        field => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::InvalidField {
                                    head: Value::Vector(Default::default()).typ(),
                                    field: field.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    Value::Tuple(arc) => match field {
                        Value::Int(value) => {
                            let len = arc.lock().unwrap().len();
                            let mut values = arc.lock().unwrap();
                            let dst = if value <= -1 {
                                if (value.unsigned_abs() - 1) as usize > len {
                                    None
                                } else {
                                    values.get_mut(len - value.unsigned_abs() as usize)
                                }
                            } else {
                                values.get_mut(value.unsigned_abs() as usize)
                            }
                            .ok_or(RunTimeError {
                                err: RunTimeErrorKind::IndexOutOfRange { index: value, len },
                                ln,
                            })?;
                            *dst = src;
                        }
                        field => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::InvalidField {
                                    head: Value::Vector(Default::default()).typ(),
                                    field: field.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    Value::Map(arc) => match field {
                        Value::String(key) => {
                            let mut map = arc.lock().unwrap();
                            map.insert(key, src);
                        }
                        field => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::InvalidField {
                                    head: Value::Map(Default::default()).typ(),
                                    field: field.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    head => {
                        return Err(RunTimeError {
                            err: RunTimeErrorKind::InvalidFieldHead(head.typ()),
                            ln,
                        })
                    }
                };
            }
            ByteCode::Vector { dst, start, amount } => {
                let dst = self.location(dst).unwrap();
                let mut values = vec![];
                for reg in start..(start + amount) {
                    values.push(self.source(Source::Register(reg)).unwrap_or_default());
                }
                *dst.lock().unwrap() = Value::Vector(Arc::new(Mutex::new(values)));
            }
            ByteCode::Tuple { dst, start, amount } => {
                let dst = self.location(dst).unwrap();
                let mut values = vec![];
                for reg in start..(start + amount) {
                    values.push(self.source(Source::Register(reg)).unwrap_or_default());
                }
                *dst.lock().unwrap() =
                    Value::Tuple(Arc::new(Mutex::new(values.into_boxed_slice())));
            }
            ByteCode::Map { dst } => {
                let dst = self.location(dst).unwrap();
                *dst.lock().unwrap() = Value::Map(Arc::new(Mutex::new(Default::default())));
            }
            ByteCode::Fn { dst, addr } => {
                let dst = self.location(dst).unwrap();
                let closure = self.closure(addr).unwrap();
                *dst.lock().unwrap() =
                    Value::Fn(FnKind::Function(Arc::new(Mutex::new(Function {
                        closure: Rc::clone(closure),
                    }))));
            }
            ByteCode::Binary {
                op,
                dst,
                left,
                right,
            } => {
                let dst = self.location(dst).unwrap();
                let left = self.source(left).unwrap();
                let right = self.source(right).unwrap();
                *dst.lock().unwrap() = match op {
                    BinaryOperation::Add => match (left, right) {
                        (Value::Int(left), Value::Int(right)) => Value::Int(left + right),
                        (Value::Float(left), Value::Float(right)) => Value::Float(left + right),
                        (Value::Int(left), Value::Float(right)) => {
                            Value::Float(left as f64 + right)
                        }
                        (Value::Float(left), Value::Int(right)) => {
                            Value::Float(left + right as f64)
                        }
                        (Value::String(left), Value::String(right)) => Value::String(left + &right),
                        (left, right) => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalBinaryOperation {
                                    op,
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    BinaryOperation::Sub => match (left, right) {
                        (Value::Int(left), Value::Int(right)) => Value::Int(left - right),
                        (Value::Float(left), Value::Float(right)) => Value::Float(left - right),
                        (Value::Int(left), Value::Float(right)) => {
                            Value::Float(left as f64 - right)
                        }
                        (Value::Float(left), Value::Int(right)) => {
                            Value::Float(left - right as f64)
                        }
                        (left, right) => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalBinaryOperation {
                                    op,
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    BinaryOperation::Mul => match (left, right) {
                        (Value::Int(left), Value::Int(right)) => Value::Int(left * right),
                        (Value::Float(left), Value::Float(right)) => Value::Float(left * right),
                        (Value::Int(left), Value::Float(right)) => {
                            Value::Float(left as f64 * right)
                        }
                        (Value::Float(left), Value::Int(right)) => {
                            Value::Float(left * right as f64)
                        }
                        (Value::String(left), Value::Int(right)) => {
                            Value::String(left.repeat(right.max(0) as usize))
                        }
                        (left, right) => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalBinaryOperation {
                                    op,
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    BinaryOperation::Div => match (left, right) {
                        (Value::Int(left), Value::Int(right)) => Value::Int(left / right),
                        (Value::Float(left), Value::Float(right)) => Value::Float(left / right),
                        (Value::Int(left), Value::Float(right)) => {
                            Value::Float(left as f64 / right)
                        }
                        (Value::Float(left), Value::Int(right)) => {
                            Value::Float(left / right as f64)
                        }
                        (left, right) => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalBinaryOperation {
                                    op,
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    BinaryOperation::Mod => match (left, right) {
                        (Value::Int(left), Value::Int(right)) => Value::Int(left % right),
                        (Value::Float(left), Value::Float(right)) => Value::Float(left % right),
                        (Value::Int(left), Value::Float(right)) => {
                            Value::Float(left as f64 % right)
                        }
                        (Value::Float(left), Value::Int(right)) => {
                            Value::Float(left % right as f64)
                        }
                        (left, right) => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalBinaryOperation {
                                    op,
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    BinaryOperation::Pow => match (left, right) {
                        (Value::Int(left), Value::Int(right)) => Value::Int(
                            left.pow(right.max(0).unsigned_abs().try_into().unwrap_or_default()),
                        ),
                        (Value::Float(left), Value::Float(right)) => Value::Float(left.powf(right)),
                        (Value::Int(left), Value::Float(right)) => {
                            Value::Float((left as f64).powf(right))
                        }
                        (Value::Float(left), Value::Int(right)) => {
                            Value::Float(left.powf(right as f64))
                        }
                        (left, right) => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalBinaryOperation {
                                    op,
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    BinaryOperation::EE => Value::Bool(left == right),
                    BinaryOperation::NE => Value::Bool(left != right),
                    BinaryOperation::LT => match (left, right) {
                        (Value::Int(left), Value::Int(right)) => Value::Bool(left < right),
                        (Value::Float(left), Value::Float(right)) => Value::Bool(left < right),
                        (Value::Int(left), Value::Float(right)) => {
                            Value::Bool((left as f64) < right)
                        }
                        (Value::Float(left), Value::Int(right)) => Value::Bool(left < right as f64),
                        (Value::Char(left), Value::Char(right)) => Value::Bool(left < right),
                        (left, right) => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalBinaryOperation {
                                    op,
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    BinaryOperation::GT => match (left, right) {
                        (Value::Int(left), Value::Int(right)) => Value::Bool(left > right),
                        (Value::Float(left), Value::Float(right)) => Value::Bool(left > right),
                        (Value::Int(left), Value::Float(right)) => Value::Bool(left as f64 > right),
                        (Value::Float(left), Value::Int(right)) => Value::Bool(left > right as f64),
                        (Value::Char(left), Value::Char(right)) => Value::Bool(left > right),
                        (left, right) => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalBinaryOperation {
                                    op,
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    BinaryOperation::LE => match (left, right) {
                        (Value::Int(left), Value::Int(right)) => Value::Bool(left <= right),
                        (Value::Float(left), Value::Float(right)) => Value::Bool(left <= right),
                        (Value::Int(left), Value::Float(right)) => {
                            Value::Bool(left as f64 <= right)
                        }
                        (Value::Float(left), Value::Int(right)) => {
                            Value::Bool(left <= right as f64)
                        }
                        (Value::Char(left), Value::Char(right)) => Value::Bool(left <= right),
                        (left, right) => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalBinaryOperation {
                                    op,
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    BinaryOperation::GE => match (left, right) {
                        (Value::Int(left), Value::Int(right)) => Value::Bool(left >= right),
                        (Value::Float(left), Value::Float(right)) => Value::Bool(left >= right),
                        (Value::Int(left), Value::Float(right)) => {
                            Value::Bool(left as f64 >= right)
                        }
                        (Value::Float(left), Value::Int(right)) => {
                            Value::Bool(left >= right as f64)
                        }
                        (Value::Char(left), Value::Char(right)) => Value::Bool(left >= right),
                        (left, right) => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalBinaryOperation {
                                    op,
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    BinaryOperation::And => Value::Bool(bool::from(left) && bool::from(right)),
                    BinaryOperation::Or => Value::Bool(bool::from(left) && bool::from(right)),
                    BinaryOperation::Is => todo!(),
                    BinaryOperation::In => match (left, right) {
                        (Value::Char(left), Value::String(right)) => {
                            Value::Bool(right.contains(left))
                        }
                        (Value::String(left), Value::Map(right)) => {
                            let right = right.lock().unwrap();
                            Value::Bool(right.contains_key(&left))
                        }
                        (left, Value::Vector(right)) => {
                            let right = right.lock().unwrap();
                            Value::Bool(right.contains(&left))
                        }
                        (left, Value::Tuple(right)) => {
                            let right = right.lock().unwrap();
                            Value::Bool(right.contains(&left))
                        }
                        (left, right) => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalBinaryOperation {
                                    op,
                                    left: left.typ(),
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    BinaryOperation::As => todo!(),
                };
            }
            ByteCode::Unary { op, dst, right } => {
                let dst = self.location(dst).unwrap();
                let right = self.source(right).unwrap();
                *dst.lock().unwrap() = match op {
                    UnaryOperation::Neg => match right {
                        Value::Int(right) => Value::Int(-right),
                        Value::Float(right) => Value::Float(-right),
                        right => {
                            return Err(RunTimeError {
                                err: RunTimeErrorKind::IllegalUnaryOperation {
                                    op,
                                    right: right.typ(),
                                },
                                ln,
                            })
                        }
                    },
                    UnaryOperation::Not => Value::Bool(!bool::from(right)),
                };
            }
        }
        Ok(None)
    }
    pub fn run(&mut self) -> Result<Option<Value>, RunTimeError> {
        let offset = self.call_stack.len();
        if offset == 0 {
            return Ok(None);
        }
        loop {
            let return_call = self.step()?;
            if self.call_stack.len() < offset {
                if let Some(value) = return_call {
                    return Ok(value);
                }
            }
            if self.call_stack.len() < offset - 1 {
                break;
            }
        }
        Ok(None)
    }
}
