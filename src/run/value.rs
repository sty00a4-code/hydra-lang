use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}};

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
    NativObject(Pointer<dyn NativObject>)
}
#[derive(Clone)]
pub enum FnKind {
    
}
pub trait NativObject {
    fn typ(&self) -> &'static str;
    fn get(&self, key: &str) -> Option<Value>;
}