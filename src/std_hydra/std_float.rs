use crate::*;
use crate::run::interpreter::{Interpreter, FLOAT_MODULE};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: FLOAT_MODULE = make_map!{
        
    });
}