use crate::run::interpreter::Interpreter;
use crate::*;
use std::{process, time};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: "os" = make_map!{
        "id" = native_fn!(_id),
        "exit" = native_fn!(_exit),
        "time" = native_fn!(_time),
    });
}
define_native_fn!(_id (_i args): => {
    Ok(Some(process::id().into()))
});
define_native_fn!(_exit (_i args): code = typed!(args: Int) => {
    process::exit(code as i32)
});
define_native_fn!(_time (_i args): => {
    Ok(
        time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)
        .ok()
        .map(|d| Value::Float(d.as_secs_f64()))
    )
});
