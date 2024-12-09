use crate::*;
use crate::run::interpreter::Interpreter;

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: "io" = make_map!{
    });
}