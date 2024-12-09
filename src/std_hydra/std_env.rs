use std::collections::HashMap;
use std::env;

use crate::run::interpreter::Interpreter;
use crate::*;

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: "env" = make_map!{
        "os" = env::consts::OS,
        "arch" = env::consts::ARCH,
        "family" = env::consts::FAMILY,
        "args" = native_fn!(_args),
        "current_dir" = native_fn!(_current_dir),
        "set_current_dir" = native_fn!(_set_current_dir),
        "current_exe" = native_fn!(_current_exe),
        "temp_dir" = native_fn!(_temp_dir),
        "var" = native_fn!(_var),
        "vars" = native_fn!(_vars),
        "set_var" = native_fn!(_set_var),
        "remove_var" = native_fn!(_remove_var),
    });
}
define_native_fn!(_args (_i args): => {
    Ok(Some(env::args().map(Value::String).collect::<Vec<Value>>().into()))
});
define_native_fn!(_current_dir (_i args): => {
    Ok(env::current_dir().map(|path| Value::String(path.to_str().unwrap_or_default().to_string())).ok())
});
define_native_fn!(_set_current_dir (_i args): path = typed!(args: String) => {
    env::set_current_dir(path)?;
    Ok(None)
});
define_native_fn!(_current_exe (_i args): => {
    Ok(env::current_exe().map(|path| Value::String(path.to_str().unwrap_or_default().to_string())).ok())
});
define_native_fn!(_temp_dir (_i args): => {
    Ok(Some(env::temp_dir().to_str().unwrap_or_default().to_string().into()))
});
define_native_fn!(_var (_i args): var = typed!(args: String) => {
    Ok(env::var(var).ok().map(Value::String))
});
define_native_fn!(_vars (_i args): => {
    Ok(Some(env::vars().map(|(k, v)| (k, Value::String(v))).collect::<HashMap<String, Value>>().into()))
});
define_native_fn!(_set_var (_i args): var = typed!(args: String), value = typed!(args: String) => {
    env::set_var(var, value);
    Ok(None)
});
define_native_fn!(_remove_var (_i args): var = typed!(args: String) => {
    env::remove_var(var);
    Ok(None)
});
