use crate::*;
use crate::run::interpreter::{Interpreter, STRING_MODULE};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: STRING_MODULE = make_map!{
        
    });
}