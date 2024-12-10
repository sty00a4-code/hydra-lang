use crate::*;
use crate::run::interpreter::{Interpreter, INT_MODULE};
use super::std_math::*;

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: INT_MODULE = make_map!{
        "from_bin" = native_fn!(_from_bin),
        "from_hex" = native_fn!(_from_hex),
        "to_bin" = native_fn!(_to_bin),
        "to_hex" = native_fn!(_to_hex),
        "abs" = native_fn!(_abs),
        "sqrt" = native_fn!(_sqrt),
        "max" = native_fn!(_max),
        "min" = native_fn!(_min),
        "log" = native_fn!(_log),
        "log2" = native_fn!(_log2),
        "log10" = native_fn!(_log10),
    });
}

define_native_fn!(_from_bin (_i args): src = typed!(args: String) => {
    Ok(i64::from_str_radix(&src, 2).ok().map(Value::Int))
});
define_native_fn!(_from_hex (_i args): src = typed!(args: String) => {
    Ok(i64::from_str_radix(&src, 16).ok().map(Value::Int))
});
define_native_fn!(_to_bin (_i args): value = typed!(args: Int) => {
    Ok(Some(format!("{:b}", value).into()))
});
define_native_fn!(_to_hex (_i args): value = typed!(args: Int) => {
    Ok(Some(format!("{:x}", value).into()))
});
define_native_fn!(_sqrt (_i args): value = typed!(args: Int) => {
    Ok(Some(value.isqrt().into()))
});
define_native_fn!(_log (_i args): value = typed!(args: Int), base = typed!(args: Int) => {
    Ok(Some(value.ilog(base).into()))
});
define_native_fn!(_log2 (_i args): value = typed!(args: Int) => {
    Ok(Some(value.ilog2().into()))
});
define_native_fn!(_log10 (_i args): value = typed!(args: Int) => {
    Ok(Some(value.ilog10().into()))
});