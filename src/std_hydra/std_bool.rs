use crate::*;
use crate::run::interpreter::{Interpreter, BOOL_MODULE};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: BOOL_MODULE = make_map!{
        
    });
}