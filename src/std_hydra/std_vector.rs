use crate::*;
use crate::run::interpreter::{Interpreter, VECTOR_MODULE};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: VECTOR_MODULE = make_map!{
        
    });
}