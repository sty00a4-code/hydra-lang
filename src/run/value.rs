use super::code::Closure;
use std::{
    collections::HashMap,
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
#[derive(Debug, Clone)]
pub enum FnKind {
    Function(Pointer<Function>),
    Native(Rc<NativeFn>),
}
#[derive(Debug, Clone)]
pub struct Function {
    pub closure: Rc<Closure>,
}
pub type NativeFn = ();
pub trait NativeObject {
    fn typ(&self) -> &'static str;
    fn get(&self, key: &str) -> Option<Value>;
}

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
                Rc::as_ptr(left) == Rc::as_ptr(right)
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
