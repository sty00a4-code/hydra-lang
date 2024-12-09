use crate::run::interpreter::Interpreter;
use crate::*;
use std::process;

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: "os" = make_map!{
        "id" = native_fn!(_id),
        "exit" = native_fn!(_exit),
    });
}
define_native_fn!(_id (_i args): => {
    Ok(Some(process::id().into()))
});
define_native_fn!(_exit (_i args): code = typed!(args: Int) => {
    process::exit(code as i32)
});
