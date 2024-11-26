use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    rc::Rc,
    sync::{Arc, Mutex},
};

use super::{code::Closure, typ::Type};

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
    Tuple(Pointer<[Self]>),
    Map(Pointer<HashMap<String, Self>>),
    Set(Pointer<HashSet<Self>>),
    Fn(FnKind),
    NativeObject(Pointer<dyn NativeObject>),
    Type(Type),
}
#[derive(Debug, Clone)]
pub enum FnKind {
    Function(Rc<Closure>),
    Native(Rc<NativeFn>),
}
pub type NativeFn = ();
pub trait NativeObject {
    fn typ(&self) -> &'static str;
    fn get(&self, key: &str) -> Option<Value>;
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
            Value::Tuple(arc) => write!(
                f,
                "({})",
                arc.lock()
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
            Value::Set(arc) => write!(
                f,
                "{{{}}}",
                arc.lock()
                    .unwrap()
                    .iter()
                    .map(|v| format!("{v:?}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Value::Fn(FnKind::Function(rc)) => write!(f, "fn:{:08x?}", Rc::as_ptr(rc)),
            Value::Fn(FnKind::Native(rc)) => write!(f, "fn:{:08x?}", Rc::as_ptr(rc)),
            Value::NativeObject(arc) => {
                write!(f, "{}:{:08x?}", arc.lock().unwrap().typ(), Arc::as_ptr(arc))
            }
            Value::Type(typ) => write!(f, "{typ:?}"),
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

impl Value {
    pub fn check(&self, typ: &Type) -> bool {
        match typ {
            Type::Null => matches!(self, Self::Null),
            Type::Int => matches!(self, Self::Int(_)),
            Type::Float => matches!(self, Self::Float(_)),
            Type::Bool => matches!(self, Self::Bool(_)),
            Type::Char => matches!(self, Self::Char(_)),
            Type::String => matches!(self, Self::String(_)),
            Type::Vector(sub) => {
                if let Self::Vector(arc) = self {
                    if let Some(sub) = sub {
                        arc.lock().unwrap().iter().all(|v| v.check(sub.as_ref()))
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            Type::Tuple(subs) => {
                if let Self::Tuple(arc) = self {
                    if let Some(subs) = subs {
                        arc.lock()
                            .unwrap()
                            .iter()
                            .zip(subs.iter())
                            .all(|(v, t)| v.check(t))
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            Type::Map(map_type) => todo!(),
            Type::Set(_) => todo!(),
            Type::Fn(fn_type) => todo!(),
            Type::NativeObject(_) => todo!(),
            Type::Type => todo!(),
            Type::Many(_) => todo!(),
            Type::Maybe(_) => todo!(),
        }
    }
}
