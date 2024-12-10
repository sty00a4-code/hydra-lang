use crate::*;
use crate::run::interpreter::{Interpreter, CHAR_MODULE};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: CHAR_MODULE = make_map!{
        
    });
}