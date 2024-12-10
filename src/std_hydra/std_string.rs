use crate::run::interpreter::{Interpreter, STRING_MODULE};
use crate::*;

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: STRING_MODULE = make_map!{
        "len" = native_fn!(_len),
        "get" = native_fn!(_get),
        "lower" = native_fn!(_lower),
        "upper" = native_fn!(_upper),
        "sub" = native_fn!(_sub),
        "remove" = native_fn!(_remove),
        "split" = native_fn!(_split),
        "sep" = native_fn!(_split),
        "split_once" = native_fn!(_split_once),
        "split_at" = native_fn!(_split_at),
        "split_off" = native_fn!(_split_off),
        "trim" = native_fn!(_trim),
        "trim_start" = native_fn!(_trim_start),
        "trim_end" = native_fn!(_trim_end),
        "trim_start_matches" = native_fn!(_trim_start_matches),
        "trim_end_matches" = native_fn!(_trim_end_matches),
    });
}

define_native_fn!(_len (_i args): value = typed!(args: String) => {
    Ok(Some(value.len().into()))
});
define_native_fn!(_get (_i args): value = typed!(args: String), index = typed!(args: Int) => {
    let index = if index <= -1 {
        if (index.unsigned_abs() - 1) as usize > value.len() {
            0
        } else {
            value.len() - index.unsigned_abs() as usize
        }
    } else {
        index.unsigned_abs() as usize
    };
    Ok(value.get(index..=index).and_then(|s| s.chars().next()).map(Value::Char))
});
define_native_fn!(_lower (_i args): value = typed!(args: String) => {
    Ok(Some(value.to_ascii_lowercase().into()))
});
define_native_fn!(_upper (_i args): value = typed!(args: String) => {
    Ok(Some(value.to_ascii_uppercase().into()))
});
define_native_fn!(_sub (_i args): value = typed!(args: String), start = typed!(args: Int), end = typed!(args: Int?) => {
    if let Some(end) = end {
        Ok(value.get(start as usize..end as usize).map(|s| Value::String(s.to_string())))
    } else {
        Ok(value.get(start as usize..).map(|s| Value::String(s.to_string())))
    }
});
define_native_fn!(_remove (_i args): mut value = typed!(args: String), index = typed!(args: Int) => {
    let index = if index <= -1 {
        if (index.unsigned_abs() - 1) as usize > value.len() {
            0
        } else {
            value.len() - index.unsigned_abs() as usize
        }
    } else {
        index.unsigned_abs() as usize
    };
    Ok(Some(value.remove(index).into()))
});
define_native_fn!(_split (_i args): value = typed!(args: String), sep = typed!(args: String) => {
    Ok(Some(value.split(&sep).map(|s| Value::String(s.to_string())).collect::<Vec<Value>>().into()))
});
define_native_fn!(_split_once (_i args): value = typed!(args: String), sep = typed!(args: String) => {
    Ok(value.split_once(&sep).map(|(a, b)| make_tuple!(a.to_string(), b.to_string())))
});
define_native_fn!(_split_off (_i args): mut value = typed!(args: String), index = typed!(args: Int) => {
    let index = if index <= -1 {
        if (index.unsigned_abs() - 1) as usize > value.len() {
            0
        } else {
            value.len() - index.unsigned_abs() as usize
        }
    } else {
        index.unsigned_abs() as usize
    };
    Ok(Some(value.split_off(index).into()))
});
define_native_fn!(_split_at (_i args): value = typed!(args: String), index = typed!(args: Int) => {
    let index = if index <= -1 {
        if (index.unsigned_abs() - 1) as usize > value.len() {
            0
        } else {
            value.len() - index.unsigned_abs() as usize
        }
    } else {
        index.unsigned_abs() as usize
    };
    Ok(value.split_at_checked(index).map(|(a, b)| make_tuple!(a.to_string(), b.to_string())))
});
define_native_fn!(_trim (_i args): value = typed!(args: String) => {
    Ok(Some(value.trim_ascii().into()))
});
define_native_fn!(_trim_start (_i args): value = typed!(args: String) => {
    Ok(Some(value.trim_ascii_start().into()))
});
define_native_fn!(_trim_end (_i args): value = typed!(args: String) => {
    Ok(Some(value.trim_ascii_end().into()))
});
define_native_fn!(_trim_matches (_i args): value = typed!(args: String), pattern = typed!(args: Char) => {
    Ok(Some(value.trim_matches(pattern).into()))
});
define_native_fn!(_trim_start_matches (_i args): value = typed!(args: String), pattern = typed!(args: Char) => {
    Ok(Some(value.trim_start_matches(pattern).into()))
});
define_native_fn!(_trim_end_matches (_i args): value = typed!(args: String), pattern = typed!(args: Char) => {
    Ok(Some(value.trim_end_matches(pattern).into()))
});
