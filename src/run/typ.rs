use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

#[derive(Clone, PartialEq)]
pub enum Type {
    Null,
    Int,
    Float,
    Bool,
    Char,
    String,
    Vector(Option<Box<Self>>),
    Tuple(Option<Box<[Self]>>),
    Map(Option<Box<MapType>>),
    Set(Option<Box<Self>>),
    Fn(Option<Box<FnType>>),
    NativeObject(String),
    Type,

    Many(Box<[Self]>),
    Maybe(Box<Self>),
}
#[derive(Clone, PartialEq)]
pub enum MapType {
    Single(Type),
    Struct(HashMap<String, Type>),
}
#[derive(Clone, PartialEq)]
pub struct FnType {
    args: Box<[Type]>,
    varargs: Option<Type>,
    return_type: Option<Type>,
}

impl Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Null => write!(f, "null"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::String => write!(f, "str"),
            Type::Vector(None) => write!(f, "vec"),
            Type::Vector(sub) => write!(f, "{sub:?}[]"),
            Type::Tuple(None) => write!(f, "tuple"),
            Type::Tuple(Some(subs)) => write!(
                f,
                "({})",
                subs.iter()
                    .map(|v| format!("{v:?}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Type::Map(None) => write!(f, "map"),
            Type::Map(Some(v)) => match v.as_ref() {
                MapType::Single(sub) => write!(f, "map<{sub:?}>"),
                MapType::Struct(hash_map) => write!(
                    f,
                    "{{ {} }}",
                    hash_map
                        .iter()
                        .map(|(k, v)| format!("{k} = {v:?}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
            },
            Type::Set(None) => write!(f, "set"),
            Type::Set(Some(sub)) => write!(f, "{{ {sub:?} }}"),
            Type::Fn(None) => write!(f, "fn"),
            Type::Fn(Some(fn_type)) => write!(
                f,
                "fn({}{}){}",
                fn_type
                    .args
                    .iter()
                    .map(|v| format!("{v:?}"))
                    .collect::<Vec<String>>()
                    .join(", "),
                if let Some(varargs) = &fn_type.varargs {
                    format!(", ...{varargs:?}")
                } else {
                    Default::default()
                },
                if let Some(return_type) = &fn_type.return_type {
                    format!(": {return_type:?}")
                } else {
                    Default::default()
                },
            ),
            Type::NativeObject(name) => write!(f, "{name}"),
            Type::Type => write!(f, "type"),
            Type::Many(v) => write!(
                f,
                "{}",
                v.iter()
                    .map(|v| format!("{v:?}"))
                    .collect::<Vec<String>>()
                    .join("|")
            ),
            Type::Maybe(v) => write!(f, "{v:?}?"),
        }
    }
}
impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
