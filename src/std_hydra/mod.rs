use crate::run::{
    interpreter::{Interpreter, RunTimeErrorKind},
    value::{FnKind, NativeFn, NativeObject, Value},
};
use crate::*;
use std::{
    error::Error,
    fmt::Display,
    io::Write,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub mod std_math;
pub mod std_fs;
pub mod std_io;
pub mod std_os;
pub mod std_net;
pub mod std_env;

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: "print" = native_fn!(_print));
    set_global!(interpreter: "write" = native_fn!(_write));
    set_global!(interpreter: "input" = native_fn!(_input));
    set_global!(interpreter: "debug" = native_fn!(_debug));
    set_global!(interpreter: "error" = native_fn!(_error));
    set_global!(interpreter: "iter" = native_fn!(_iter));
    set_global!(interpreter: "next" = native_fn!(_next));
    set_global!(interpreter: "int" = native_fn!(_int));
    set_global!(interpreter: "float" = native_fn!(_float));
    set_global!(interpreter: "bool" = native_fn!(_bool));
    set_global!(interpreter: "char" = native_fn!(_char));
    set_global!(interpreter: "str" = native_fn!(_str));
    set_global!(interpreter: "vec" = native_fn!(_vec));
    set_global!(interpreter: "tuple" = native_fn!(_tuple));
    set_global!(interpreter: "type" = native_fn!(_type));
    set_global!(interpreter: "check" = native_fn!(_check));
    set_global!(interpreter: "enumerate" = native_fn!(_enumerate));
    std_math::import(interpreter);
    std_fs::import(interpreter);
    std_io::import(interpreter);
    std_os::import(interpreter);
    std_net::import(interpreter);
    std_env::import(interpreter);
}

define_native_fn!(_print (_i args): => {
    println!("{}", args.map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(" "));
    Ok(None)
});
define_native_fn!(_write (_i args): => {
    print!("{}", args.map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(" "));
    Ok(None)
});
define_native_fn!(_input (_i args): text = typed!(args: String) => {
    let mut input = String::new();
    print!("{text}");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    Ok(Some(Value::String(input)))
});
define_native_fn!(_debug (_i args): => {
    let mut args = args.map(|(_, v)| {
        println!("{v:?}");
        v
    }).collect::<Vec<Value>>();
    if args.is_empty() {
        return Ok(None)
    }
    if args.len() == 1 {
        return Ok(Some(args.remove(0)))
    }
    Ok(Some(Value::Tuple(Arc::new(Mutex::new(
        args.into_boxed_slice()
    )))))
});
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorObject {
    msg: String,
    path: Option<String>,
    ln: usize,
}
impl ErrorObject {
    pub const TYPE: &'static str = "error";
}
impl NativeObject for ErrorObject {
    fn typ(&self) -> &'static str {
        Self::TYPE
    }
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "msg" => Some(Value::String(self.msg.clone())),
            "path" => self.path.clone().map(Value::String),
            "ln" => Some(Value::Int(self.ln as i64)),
            _ => None,
        }
    }
}
impl Display for ErrorObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.msg
        )
    }
}
impl Error for ErrorObject {}
define_native_fn!(_error (i args): msg = typed!(args: String) => {
    Err(ErrorObject {
        msg,
        path: i.path().cloned(),
        ln: i.ln().unwrap_or_default(),
    }.into())
});

pub struct IteratorObject {
    pub iter: Box<dyn Iterator<Item = Value>>,
    pub fn_next: Rc<NativeFn>,
}
unsafe impl Send for IteratorObject {}
unsafe impl Sync for IteratorObject {}
impl NativeObject for IteratorObject {
    fn typ(&self) -> &'static str {
        Self::TYPE
    }
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "next" => Some(Value::Fn(FnKind::Native(Rc::clone(&self.fn_next)))),
            _ => None,
        }
    }
    fn call_mut(
        &mut self,
        key: &str,
        _: &mut Interpreter,
        _: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        match key {
            "next" => Ok(self.next_()),
            _ => Err(RunTimeErrorKind::CannotCall(Value::default().typ())
                .to_string()
                .into()),
        }
    }
}
impl IteratorObject {
    pub const TYPE: &'static str = "iterator";
    pub fn next_(&mut self) -> Option<Value> {
        self.iter.next()
    }
    define_native_fn!(_next (i args): _self = typed!(args: Self::TYPE) => {
        let mut _self = _self.lock().unwrap();
        _self.call_mut("next", i, args.map(|(_, v)| v).collect())
    });
}
define_native_fn!(_iter (i args): value = typed!(args) => {
    match value {
        Value::Vector(values) => {
            Ok(Some(Value::NativeObject(Arc::new(Mutex::new(IteratorObject {
                iter: Box::new(values.lock().unwrap().clone().into_iter()),
                fn_next: Rc::new(IteratorObject::_next)
            })))))
        }
        Value::Tuple(values) => {
            Ok(Some(Value::NativeObject(Arc::new(Mutex::new(IteratorObject {
                #[allow(clippy::unnecessary_to_owned)]
                iter: Box::new(values.lock().unwrap().to_vec().into_iter()),
                fn_next: Rc::new(IteratorObject::_next)
            })))))
        }
        Value::Map(values) => {
            Ok(Some(Value::NativeObject(Arc::new(Mutex::new(IteratorObject {
                iter: Box::new(values.lock().unwrap().clone().into_keys().map(Value::String)),
                fn_next: Rc::new(IteratorObject::_next)
            })))))
        }
        Value::String(string) => {
            Ok(Some(Value::NativeObject(Arc::new(Mutex::new(IteratorObject {
                iter: Box::new(string.into_bytes().into_iter().map(|byte| Value::Char(byte as char))),
                fn_next: Rc::new(IteratorObject::_next)
            })))))
        }
        Value::NativeObject(ref object) => {
            let next = object.lock().unwrap().get("next").unwrap_or_default();
            if let Value::Fn(_) = next {
                return Ok(Some(value))
            }
            object.lock().unwrap().call("iter", i, args.map(|(_, v)| v).collect())
        }
        value => Err(format!("can't iterate over {}", value.typ()).into())
    }
});
define_native_fn!(_next (i args): value = typed!(args) => {
    match value {
        Value::NativeObject(object) => {
            object.lock().unwrap().call_mut("next", i, args.map(|(_, v)| v).collect())
        }
        value => Err(format!("can't get next iteration of {}", value.typ()).into())
    }
});

define_native_fn!(_int (_i args): value = typed!(args) => {
    Ok(Some(Value::Int(match value {
        Value::Int(v) => v,
        Value::Float(v) => v as i64,
        Value::Bool(v) => if v { 1 } else { 0 },
        Value::Char(v) => v as u8 as i64,
        Value::String(v) => if let Ok(v) = v.parse::<i64>() { v } else { return Ok(None); },
        _ => return Ok(None)
    })))
});
define_native_fn!(_float (_i args): value = typed!(args) => {
    Ok(Some(Value::Float(match value {
        Value::Int(v) => v as f64,
        Value::Float(v) => v,
        Value::Bool(v) => if v { 1.0 } else { 0.0 },
        Value::Char(v) => v as u8 as f64,
        Value::String(v) => if let Ok(v) = v.parse::<f64>() { v } else { return Ok(None); },
        _ => return Ok(None)
    })))
});
define_native_fn!(_bool (_i args): value = typed!(args) => {
    Ok(Some(Value::Bool(bool::from(value))))
});
define_native_fn!(_char (_i args): value = typed!(args) => {
    Ok(Some(Value::Char(match value {
        Value::Int(v) => if let Ok(v) = TryInto::<u8>::try_into(v) { v as char } else { todo!() },
        Value::Float(v) => if let Ok(v) = TryInto::<u8>::try_into(v as i64) { v as char } else { todo!() },
        Value::Char(v) => v,
        _ => return Ok(None)
    })))
});
define_native_fn!(_str (_i args): => {
    Ok(Some(Value::String(args.map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(""))))
});
define_native_fn!(_vec (_i args): value = typed!(args) => {
    if args.len() == 0 {
        Ok(Some(make_vec!(match value {
            Value::Vector(arc) => arc.lock().unwrap().clone(),
            Value::Tuple(arc) => arc.lock().unwrap().to_vec(),
            Value::Map(arc) => arc
                .lock()
                .unwrap()
                .iter()
                .map(|(k, v)| make_tuple!(Value::String(k.clone()), v.clone()))
                .collect(),
            value => vec![value],
        })))
    } else {
        let mut values: Vec<Value> = args.map(|(_, v)| v).collect();
        values.insert(0, value);
        Ok(Some(make_vec!(values)))
    }
});
define_native_fn!(_tuple (_i args): value = typed!(args) => {
    if args.len() == 0 {
        Ok(Some(make_tuple!(match value {
            Value::Vector(arc) => arc.lock().unwrap().clone().into_boxed_slice(),
            Value::Tuple(arc) => arc.lock().unwrap().clone(),
            Value::Map(arc) => arc
                .lock()
                .unwrap()
                .iter()
                .map(|(k, v)| make_tuple!(Value::String(k.clone()), v.clone()))
                .collect(),
            value => Box::new([value]),
        })))
    } else {
        let mut values: Vec<Value> = args.map(|(_, v)| v).collect();
        values.insert(0, value);
        Ok(Some(make_vec!(values)))
    }
});
define_native_fn!(_type (_i args): value = typed!(args) => {
    Ok(Some(Value::String(value.typ().to_string())))
});
define_native_fn!(_check (_i args): value = typed!(args) => {
    for (idx, arg) in args {
        if let Value::String(typ) = arg {
            if value.typ() == typ {
                return Ok(Some(value))
            }
        } else {
            return Err(format!(
                "expected {} for argument #{}, got {}",
                Value::String(Default::default()).typ(),
                idx + 1,
                arg.typ()
            )
            .into());
        }
    }
    Ok(Some(Value::default()))
});
define_native_fn!(_enumerate (i args): value = typed!(args) => {
    match value {
        Value::Vector(values) => {
            Ok(Some(Value::NativeObject(Arc::new(Mutex::new(IteratorObject {
                iter: Box::new(values
                    .lock()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| make_tuple!(Value::Int(i as i64), v))
                ),
                fn_next: Rc::new(IteratorObject::_next)
            })))))
        }
        Value::Tuple(values) => {
            Ok(Some(Value::NativeObject(Arc::new(Mutex::new(IteratorObject {
                #[allow(clippy::unnecessary_to_owned)]
                iter: Box::new(values
                    .lock()
                    .unwrap()
                    .to_vec()
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| make_tuple!(Value::Int(i as i64), v))
                ),
                fn_next: Rc::new(IteratorObject::_next)
            })))))
        }
        Value::Map(values) => {
            Ok(Some(Value::NativeObject(Arc::new(Mutex::new(IteratorObject {
                iter: Box::new(values
                    .lock()
                    .unwrap()
                    .clone()
                    .into_keys()
                    .enumerate()
                    .map(|(i, v)| make_tuple!(Value::Int(i as i64), Value::String(v)))
                ),
                fn_next: Rc::new(IteratorObject::_next)
            })))))
        }
        Value::String(string) => {
            Ok(Some(Value::NativeObject(Arc::new(Mutex::new(IteratorObject {
                iter: Box::new(string
                    .into_bytes()
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| make_tuple!(Value::Int(i as i64), Value::Char(v as char)))
                ),
                fn_next: Rc::new(IteratorObject::_next)
            })))))
        }
        Value::NativeObject(ref object) => {
            object.lock().unwrap().call("enumerate", i, args.map(|(_, v)| v).collect())
        }
        value => Err(format!("can't enumerate over {}", value.typ()).into())
    }
});
