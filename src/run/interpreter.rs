use super::{
    code::Closure,
    value::{Pointer, Value},
};
use std::{collections::HashMap, rc::Rc};

pub struct Interpreter {
    pub call_stack: Vec<CallFrame>,
    pub globals: HashMap<String, Pointer<Value>>,
}
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub closure: Rc<Closure>,
    pub stack: Vec<Pointer<Value>>,
}
