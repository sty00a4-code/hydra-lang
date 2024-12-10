use crate::*;
use crate::run::interpreter::{Interpreter, MAP_MODULE};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: MAP_MODULE = make_map!{
        
    });
}