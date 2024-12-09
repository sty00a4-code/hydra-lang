use super::{
    code::{BinaryOperation, Closure, UnaryOperation},
    interpreter::{Interpreter, RunTimeError, RunTimeErrorKind},
};
use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    rc::Rc,
    sync::{Arc, Mutex},
};

pub type Pointer<T> = Arc<Mutex<T>>;

#[derive(Clone, Default)]
pub enum Value {
    #[default]
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Vector(Pointer<Vec<Self>>),
    Tuple(Pointer<Box<[Self]>>),
    Map(Pointer<HashMap<String, Self>>),
    Fn(FnKind),
    NativeObject(Pointer<dyn NativeObject>),
}
unsafe impl Send for Value {}
unsafe impl Sync for Value {}
#[derive(Clone)]
pub enum FnKind {
    Function(Pointer<Function>),
    Native(Rc<NativeFn>),
}
#[derive(Debug, Clone)]
pub struct Function {
    pub closure: Rc<Closure>,
}
pub type NativeFn = dyn Fn(&mut Interpreter, Vec<Value>) -> Result<Option<Value>, Box<dyn Error>>;
pub trait NativeObject {
    fn typ(&self) -> &'static str;
    #[allow(unused_variables)]
    fn get(&self, key: &str) -> Option<Value> {
        None
    }
    #[allow(unused_variables)]
    fn call(
        &self,
        key: &str,
        interpreter: &mut Interpreter,
        args: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        Err(RunTimeErrorKind::CannotCall(Value::default().typ())
            .to_string()
            .into())
    }
    #[allow(unused_variables)]
    fn call_mut(
        &mut self,
        key: &str,
        interpreter: &mut Interpreter,
        args: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        Err(RunTimeErrorKind::CannotCall(Value::default().typ())
            .to_string()
            .into())
    }
    fn __str(&self) -> Option<Rc<NativeFn>> {
        None
    }
}

unsafe impl Send for Function {}
unsafe impl Sync for Function {}
impl Value {
    pub fn typ(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Char(_) => "char",
            Value::String(_) => "str",
            Value::Vector(_) => "vec",
            Value::Tuple(_) => "tuple",
            Value::Map(_) => "map",
            Value::Fn(_) => "fn",
            Value::NativeObject(arc) => arc.lock().unwrap().typ(),
        }
    }
    pub fn field(self, field: Value, ln: usize) -> Result<Value, RunTimeError> {
        Ok(match self {
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
            Value::NativeObject(arc) => match field {
                Value::String(key) => {
                    let map = arc.lock().unwrap();
                    map.get(&key).unwrap_or_default()
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
        })
    }
    pub fn set_field(self, field: Value, src: Value, ln: usize) -> Result<(), RunTimeError> {
        match self {
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
        Ok(())
    }
    pub fn binary(
        op: BinaryOperation,
        left: Self,
        right: Self,
        ln: usize,
    ) -> Result<Self, RunTimeError> {
        if let (Value::Tuple(left), Value::Tuple(right)) = (&left, &right) {
            let left = left.lock().unwrap();
            let right = right.lock().unwrap();
            let mut new = Vec::with_capacity(left.len());
            for (left, right) in left.iter().zip(right.iter()) {
                new.push(Self::binary(op, left.clone(), right.clone(), ln)?);
            }
            return Ok(Self::Tuple(Arc::new(Mutex::new(new.into_boxed_slice()))));
        }
        Ok(match op {
            BinaryOperation::Add => match (left, right) {
                (Value::Int(left), Value::Int(right)) => Value::Int(left + right),
                (Value::Float(left), Value::Float(right)) => Value::Float(left + right),
                (Value::Int(left), Value::Float(right)) => Value::Float(left as f64 + right),
                (Value::Float(left), Value::Int(right)) => Value::Float(left + right as f64),
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
                (Value::Int(left), Value::Float(right)) => Value::Float(left as f64 - right),
                (Value::Float(left), Value::Int(right)) => Value::Float(left - right as f64),
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
                (Value::Int(left), Value::Float(right)) => Value::Float(left as f64 * right),
                (Value::Float(left), Value::Int(right)) => Value::Float(left * right as f64),
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
                (Value::Int(left), Value::Float(right)) => Value::Float(left as f64 / right),
                (Value::Float(left), Value::Int(right)) => Value::Float(left / right as f64),
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
                (Value::Int(left), Value::Float(right)) => Value::Float(left as f64 % right),
                (Value::Float(left), Value::Int(right)) => Value::Float(left % right as f64),
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
                (Value::Int(left), Value::Int(right)) => {
                    Value::Int(left.pow(right.max(0).unsigned_abs().try_into().unwrap_or_default()))
                }
                (Value::Float(left), Value::Float(right)) => Value::Float(left.powf(right)),
                (Value::Int(left), Value::Float(right)) => Value::Float((left as f64).powf(right)),
                (Value::Float(left), Value::Int(right)) => Value::Float(left.powf(right as f64)),
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
                (Value::Int(left), Value::Float(right)) => Value::Bool((left as f64) < right),
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
                (Value::Int(left), Value::Float(right)) => Value::Bool(left as f64 <= right),
                (Value::Float(left), Value::Int(right)) => Value::Bool(left <= right as f64),
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
                (Value::Int(left), Value::Float(right)) => Value::Bool(left as f64 >= right),
                (Value::Float(left), Value::Int(right)) => Value::Bool(left >= right as f64),
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
            BinaryOperation::Is => match (left, right) {
                (left, Value::String(right)) => Value::Bool(left.typ() == right),
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
            BinaryOperation::As => match (left, right) {
                (left, Value::String(right)) => match right.as_str() {
                    "int" => i64::try_from(left).ok().map(Value::Int).unwrap_or_default(),
                    "float" => f64::try_from(left)
                        .ok()
                        .map(Value::Float)
                        .unwrap_or_default(),
                    "bool" => Value::Bool(bool::from(left)),
                    "char" => char::try_from(left)
                        .ok()
                        .map(Value::Char)
                        .unwrap_or_default(),
                    "str" => String::try_from(left)
                        .ok()
                        .map(Value::String)
                        .unwrap_or_default(),
                    "vec" => Vec::try_from(left)
                        .ok()
                        .map(|v| Value::Vector(Arc::new(Mutex::new(v))))
                        .unwrap_or_default(),
                    "tuple" => TryFrom::<Value>::try_from(left)
                        .ok()
                        .map(|v| Value::Tuple(Arc::new(Mutex::new(v))))
                        .unwrap_or_default(),
                    _ => {
                        return Err(RunTimeError {
                            err: RunTimeErrorKind::UnknownTypeCast(right),
                            ln,
                        })
                    }
                },
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
            BinaryOperation::In => match (left, right) {
                (Value::Char(left), Value::String(right)) => Value::Bool(right.contains(left)),
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
        })
    }
    pub fn unary(op: UnaryOperation, right: Self, ln: usize) -> Result<Self, RunTimeError> {
        if let Value::Tuple(right) = &right {
            let right = right.lock().unwrap();
            let mut new = Vec::with_capacity(right.len());
            for right in right.iter() {
                new.push(Self::unary(op, right.clone(), ln)?);
            }
            return Ok(Self::Tuple(Arc::new(Mutex::new(new.into_boxed_slice()))));
        }
        Ok(match op {
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
        })
    }
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Int(left), Self::Int(right)) => left == right,
            (Self::Float(left), Self::Float(right)) => left == right,
            (Self::Int(left), Self::Float(right)) => (*left as f64) == *right,
            (Self::Float(left), Self::Int(right)) => *left == (*right as f64),
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Char(left), Self::Char(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Vector(left), Self::Vector(right)) => Arc::as_ptr(left) == Arc::as_ptr(right),
            (Self::Tuple(left), Self::Tuple(right)) => {
                let left = left.lock().unwrap();
                let right = right.lock().unwrap();
                for (idx, left) in left.iter().enumerate() {
                    if !right.get(idx).map(|v| left == v).unwrap_or_default() {
                        return false;
                    }
                }
                true
            }
            (Self::Fn(FnKind::Function(left)), Self::Fn(FnKind::Function(right))) => {
                Arc::as_ptr(left) == Arc::as_ptr(right)
            }
            (Self::Fn(FnKind::Native(left)), Self::Fn(FnKind::Native(right))) => {
                std::ptr::addr_eq(Rc::as_ptr(left), Rc::as_ptr(right))
            }
            (Self::NativeObject(left), Self::NativeObject(right)) => {
                std::ptr::addr_eq(Arc::as_ptr(left), Arc::as_ptr(right))
            }
            _ => false,
        }
    }
}
impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Int(v) => write!(f, "{v:?}"),
            Value::Float(v) => write!(f, "{v:?}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Char(v) => write!(f, "{v:?}"),
            Value::String(v) => write!(f, "{v:?}"),
            Value::Vector(arc) => write!(f, "{:?}", arc.lock().unwrap()),
            Value::Tuple(values) => write!(
                f,
                "({})",
                values
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|v| format!("{v:?}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::Map(arc) => write!(
                f,
                "{{ {} }}",
                arc.lock()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| format!("{k:?} = {v:?}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::Fn(FnKind::Function(arc)) => write!(f, "fn:{:08x?}", Arc::as_ptr(arc)),
            Value::Fn(FnKind::Native(rc)) => write!(f, "fn:{:08x?}", Rc::as_ptr(rc)),
            Value::NativeObject(arc) => {
                write!(f, "{}:{:08x?}", arc.lock().unwrap().typ(), Arc::as_ptr(arc))
            }
        }
    }
}
impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float(v) => write!(f, "{v}"),
            Self::Char(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "{v}"),
            _ => Debug::fmt(self, f),
        }
    }
}
impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => false,
            Value::Int(v) => v == 0,
            Value::Float(v) => v == 0.0,
            Value::Bool(v) => v,
            Value::Char(v) => v as u8 == 0,
            Value::String(v) => !v.is_empty(),
            Value::Vector(_) => true,
            Value::Tuple(_) => true,
            Value::Map(_) => true,
            Value::Fn(_) => true,
            Value::NativeObject(_) => true,
        }
    }
}
impl TryFrom<Value> for i64 {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Int(v) => v,
            Value::Float(v) => v as i64,
            _ => return Err(()),
        })
    }
}
impl TryFrom<Value> for f64 {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Int(v) => v as f64,
            Value::Float(v) => v,
            _ => return Err(()),
        })
    }
}
impl TryFrom<Value> for char {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Char(v) => v,
            _ => return Err(()),
        })
    }
}
impl TryFrom<Value> for String {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(value.to_string())
    }
}
impl TryFrom<Value> for Vec<Value> {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Vector(v) => v.lock().unwrap().clone(),
            Value::Tuple(v) => v.lock().unwrap().to_vec(),
            Value::Map(v) => v
                .lock()
                .unwrap()
                .keys()
                .map(|v| Value::String(v.clone()))
                .collect(),
            _ => return Err(()),
        })
    }
}
impl TryFrom<Value> for Box<[Value]> {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Vector(v) => v.lock().unwrap().clone().into_boxed_slice(),
            Value::Tuple(v) => v.lock().unwrap().clone(),
            _ => return Err(()),
        })
    }
}
impl TryFrom<Value> for HashMap<String, Value> {
    type Error = ();
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Map(v) => v.lock().unwrap().clone(),
            _ => return Err(()),
        })
    }
}
impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Self::Int(value.into())
    }
}
impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Self::Int(value.into())
    }
}
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Int(value.into())
    }
}
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}
impl From<isize> for Value {
    fn from(value: isize) -> Self {
        Self::Int(value as i64)
    }
}
impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Self::Int(value.into())
    }
}
impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Self::Int(value.into())
    }
}
impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Int(value.into())
    }
}
impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::Int(value as i64)
    }
}
impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Self::Int(value as i64)
    }
}
impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Float(value.into())
    }
}
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<char> for Value {
    fn from(value: char) -> Self {
        Self::Char(value)
    }
}
impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl<T: Into<Value> + Clone> From<Vec<T>> for Value {
    fn from(value: Vec<T>) -> Self {
        Self::Vector(Arc::new(Mutex::new(
            value.iter().map(|v| v.clone().into()).collect(),
        )))
    }
}
impl<T: Into<Value> + Clone> From<&[T]> for Value {
    fn from(value: &[T]) -> Self {
        Self::Vector(Arc::new(Mutex::new(
            value.iter().map(|v| v.clone().into()).collect(),
        )))
    }
}
impl<T: Into<Value> + Clone> From<Box<[T]>> for Value {
    fn from(value: Box<[T]>) -> Self {
        Self::Vector(Arc::new(Mutex::new(
            value.iter().map(|v| v.clone().into()).collect(),
        )))
    }
}
impl<T: Into<Value> + Clone, const SIZE: usize> From<[T; SIZE]> for Value {
    fn from(value: [T; SIZE]) -> Self {
        Self::Vector(Arc::new(Mutex::new(
            value.iter().map(|v| v.clone().into()).collect(),
        )))
    }
}
impl<T: Into<Value>> From<(T,)> for Value {
    fn from(value: (T,)) -> Self {
        Self::Tuple(Arc::new(Mutex::new(Box::new([value.0.into()]))))
    }
}
impl<T: Into<Value>> From<(T, T)> for Value {
    fn from(value: (T, T)) -> Self {
        Self::Tuple(Arc::new(Mutex::new(Box::new([
            value.0.into(),
            value.1.into(),
        ]))))
    }
}
impl<T: Into<Value>> From<(T, T, T)> for Value {
    fn from(value: (T, T, T)) -> Self {
        Self::Tuple(Arc::new(Mutex::new(Box::new([
            value.0.into(),
            value.1.into(),
            value.2.into(),
        ]))))
    }
}
impl<T: Into<Value>> From<(T, T, T, T)> for Value {
    fn from(value: (T, T, T, T)) -> Self {
        Self::Tuple(Arc::new(Mutex::new(Box::new([
            value.0.into(),
            value.1.into(),
            value.2.into(),
            value.3.into(),
        ]))))
    }
}
impl<T: Into<Value>> From<(T, T, T, T, T)> for Value {
    fn from(value: (T, T, T, T, T)) -> Self {
        Self::Tuple(Arc::new(Mutex::new(Box::new([
            value.0.into(),
            value.1.into(),
            value.2.into(),
            value.3.into(),
            value.4.into(),
        ]))))
    }
}
impl<T: Into<Value>> From<HashMap<String, T>> for Value {
    fn from(value: HashMap<String, T>) -> Self {
        Self::Map(Arc::new(Mutex::new(
            value.into_iter().map(|(k, v)| (k, v.into())).collect(),
        )))
    }
}
impl<T: Into<Value>> From<HashMap<&str, T>> for Value {
    fn from(value: HashMap<&str, T>) -> Self {
        Self::Map(Arc::new(Mutex::new(
            value
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.into()))
                .collect(),
        )))
    }
}
