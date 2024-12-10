use crate::run::interpreter::{Interpreter, MAP_MODULE};
use crate::*;
use std::collections::HashMap;

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: MAP_MODULE = make_map!{
        "len" = native_fn!(_len),
        "get" = native_fn!(_get),
        "set" = native_fn!(_set),
        "key_of" = native_fn!(_key_of),
        "keys" = native_fn!(_keys),
        "values" = native_fn!(_values),
        "clear" = native_fn!(_clear),
        "copy" = native_fn!(_copy),
    });
}
define_native_fn!(_len (_i args): value = typed!(args: Map) => {
    let value = value.lock().unwrap();
    Ok(Some(value.len().into()))
});
define_native_fn!(_get (_i args): value = typed!(args: Map), key = typed!(args: String), default = typed!(args) => {
    let value = value.lock().unwrap();
    Ok(Some(value.get(&key).cloned().unwrap_or(default)))
});
define_native_fn!(_set (_i args): value = typed!(args: Map), key = typed!(args: String), new_value = typed!(args) => {
    let mut value = value.lock().unwrap();
    Ok(value.insert(key, new_value))
});
define_native_fn!(_key_of (_i args): value = typed!(args: Map), search = typed!(args) => {
    let value = value.lock().unwrap();
    Ok(value.iter().find_map(|(k, v)| if v == &search { Some(Value::from(k.as_str())) } else { None }))
});
define_native_fn!(_keys (_i args): value = typed!(args: Map) => {
    let value = value.lock().unwrap();
    Ok(Some(value.keys().cloned().collect::<Vec<String>>().into()))
});
define_native_fn!(_values (_i args): value = typed!(args: Map) => {
    let value = value.lock().unwrap();
    Ok(Some(value.values().cloned().collect::<Vec<Value>>().into()))
});
define_native_fn!(_clear (_i args): value = typed!(args: Map) => {
    let mut value = value.lock().unwrap();
    value.clear();
    Ok(None)
});
define_native_fn!(_copy (_i args): value = typed!(args: Map) => {
    let value = value.lock().unwrap();
    Ok(Some(value.clone().into()))
});
define_native_fn!(_create_set (_i args): => {
    Ok(Some(args.map(|(_, v)| (v.to_string(), true)).collect::<HashMap<String, bool>>().into()))
});
