use crate::*;
use crate::run::interpreter::{Interpreter, TUPLE_MODULE};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: TUPLE_MODULE = make_map!{
        
    });
}