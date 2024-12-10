use crate::*;
use crate::run::interpreter::{Interpreter, INT_MODULE};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: INT_MODULE = make_map!{
        "from_bin" = native_fn!(_from_bin),
        "from_hex" = native_fn!(_from_hex),
        "to_bin" = native_fn!(_to_bin),
        "to_hex" = native_fn!(_to_hex),
    });
}

define_native_fn!(_from_bin (_i args): src = typed!(args: String) => {
    Ok(i64::from_str_radix(&src, 2).ok().map(Value::Int))
});
define_native_fn!(_from_hex (_i args): src = typed!(args: String) => {
    Ok(i64::from_str_radix(&src, 16).ok().map(Value::Int))
});
define_native_fn!(_to_bin (_i args): n = typed!(args: Int) => {
    Ok(Some(format!("{:b}", n).into()))
});
define_native_fn!(_to_hex (_i args): n = typed!(args: Int) => {
    Ok(Some(format!("{:x}", n).into()))
});