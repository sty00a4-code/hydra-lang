use crate::*;
use crate::run::interpreter::{Interpreter, TUPLE_MODULE};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: TUPLE_MODULE = make_map!{
        "len" = native_fn!(_len),
        "get" = native_fn!(_get),
        "pos" = native_fn!(_pos),
    });
}
define_native_fn!(_len (_i args): value = typed!(args: Tuple) => {
    let value = value.lock().unwrap();
    Ok(Some(value.len().into()))
});
define_native_fn!(_get (_i args): value = typed!(args: Tuple), index = typed!(args: Int), default = typed!(args) => {
    let value = value.lock().unwrap();
    let index = if index <= -1 {
        if (index.unsigned_abs() - 1) as usize > value.len() {
            0
        } else {
            value.len() - index.unsigned_abs() as usize
        }
    } else {
        index.unsigned_abs() as usize
    };
    Ok(Some(value.get(index).cloned().unwrap_or(default)))
});
define_native_fn!(_pos (_i args): value = typed!(args: Tuple), search = typed!(args) => {
    let value = value.lock().unwrap();
    Ok(value.iter().position(|v| v == &search).map(Value::from))
});