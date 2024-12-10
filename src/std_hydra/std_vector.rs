use crate::*;
use crate::run::interpreter::{Interpreter, VECTOR_MODULE};

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: VECTOR_MODULE = make_map!{
        "len" = native_fn!(_len),
        "get" = native_fn!(_get),
        "pos" = native_fn!(_pos),
        "push" = native_fn!(_push),
        "pop" = native_fn!(_pop),
        "clear" = native_fn!(_clear),
        "copy" = native_fn!(_copy),
        "swap" = native_fn!(_swap),
    });
}
define_native_fn!(_len (_i args): value = typed!(args: Vector) => {
    let value = value.lock().unwrap();
    Ok(Some(value.len().into()))
});
define_native_fn!(_get (_i args): value = typed!(args: Vector), index = typed!(args: Int), default = typed!(args) => {
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
define_native_fn!(_pos (_i args): value = typed!(args: Vector), search = typed!(args) => {
    let value = value.lock().unwrap();
    Ok(value.iter().position(|v| v == &search).map(Value::from))
});
define_native_fn!(_push (_i args): value = typed!(args: Vector), v = typed!(args), index = typed!(args: Int?) => {
    let mut value = value.lock().unwrap();
    if let Some(index) = index {
        let index = if index <= -1 {
            if (index.unsigned_abs() - 1) as usize > value.len() {
                0
            } else {
                value.len() - index.unsigned_abs() as usize
            }
        } else {
            index.unsigned_abs() as usize
        };
        value.insert(index, v);
    } else {
        value.push(v);
    }
    Ok(None)
});
define_native_fn!(_pop (_i args): value = typed!(args: Vector), index = typed!(args: Int?) => {
    let mut value = value.lock().unwrap();
    Ok(if let Some(index) = index {
        let index = if index <= -1 {
            if (index.unsigned_abs() - 1) as usize > value.len() {
                0
            } else {
                value.len() - index.unsigned_abs() as usize
            }
        } else {
            index.unsigned_abs() as usize
        };
        Some(value.remove(index))
    } else {
        value.pop()
    })
});
define_native_fn!(_clear (_i args): value = typed!(args: Vector) => {
    let mut value = value.lock().unwrap();
    value.clear();
    Ok(None)
});
define_native_fn!(_copy (_i args): value = typed!(args: Vector) => {
    let value = value.lock().unwrap();
    Ok(Some(make_vec!(value.clone())))
});
define_native_fn!(_swap (_i args): value = typed!(args: Vector), index1 = typed!(args: Int), index2 = typed!(args: Int) => {
    let mut value = value.lock().unwrap();
    let index1 = if index1 <= -1 {
        if (index1.unsigned_abs() - 1) as usize > value.len() {
            0
        } else {
            value.len() - index1.unsigned_abs() as usize
        }
    } else {
        index1.unsigned_abs() as usize
    };
    let index2 = if index2 <= -1 {
        if (index2.unsigned_abs() - 1) as usize > value.len() {
            0
        } else {
            value.len() - index2.unsigned_abs() as usize
        }
    } else {
        index2.unsigned_abs() as usize
    };
    value.swap(index1, index2);
    Ok(None)
});