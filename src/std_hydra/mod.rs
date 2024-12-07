use crate::run::{
    interpreter::{Interpreter, RunTimeErrorKind},
    value::{FnKind, NativeFn, NativeObject, Value},
};
use std::{
    error::Error,
    io::Write,
    rc::Rc,
    sync::{Arc, Mutex},
};

macro_rules! set_global {
    ($interpreter:ident: $key:literal = $value:expr) => {
        $interpreter
            .globals
            .insert($key.into(), Arc::new(Mutex::new($value)))
    };
}
macro_rules! typed {
    ($args:ident) => {{
        $args.next().map(|(_, v)| v).unwrap_or_default()
    }};
    ($args:ident: $typ:literal) => {{
        let Some((idx, arg)) = $args.next() else {
            return Err(format!(
                "expected {} for argument #last, got {}",
                $typ,
                Value::default().typ()
            )
            .into());
        };
        let Value::NativeObject(arc) = arg else {
            return Err(format!(
                "expected {} for argument #{}, got {}",
                $typ,
                idx + 1,
                arg.typ()
            )
            .into());
        };
        {
            let object = arc.lock().unwrap();
            if object.typ() != $typ {
                return Err(format!(
                    "expected {} for argument #{}, got {}",
                    $typ,
                    idx + 1,
                    object.typ()
                )
                .into());
            }
        }
        Arc::clone(&arc)
    }};
    ($args:ident: $typ:ident) => {{
        let Some((idx, arg)) = $args.next() else {
            return Err(format!(
                "expected {} for argument #last, got {}",
                Value::$typ(Default::default()).typ(),
                Value::default().typ()
            )
            .into());
        };
        if let Value::$typ(value) = arg {
            value
        } else {
            return Err(format!(
                "expected {} for argument #{}, got {}",
                Value::$typ(Default::default()).typ(),
                idx + 1,
                arg.typ()
            )
            .into());
        }
    }};
}
macro_rules! native_fn {
    ($fn_name:ident ($interpreter:ident $args:ident!) $body:block) => {
        pub fn $fn_name($interpreter: &mut Interpreter, $args: Vec<Value>) -> Result<Option<Value>, Box<dyn Error>> {
            $body
        }
    };
    ($fn_name:ident ($interpreter:ident $args:ident): $($name:ident = $macro:expr),* => $body:block) => {
        pub fn $fn_name($interpreter: &mut Interpreter, $args: Vec<Value>) -> Result<Option<Value>, Box<dyn Error>> {
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            let mut $args = $args.into_iter().enumerate();
            $(
                let $name = $macro;
            ) *
            $body
        }
    };
}

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: "print" = Value::Fn(FnKind::Native(Rc::new(_print))));
    set_global!(interpreter: "write" = Value::Fn(FnKind::Native(Rc::new(_write))));
    set_global!(interpreter: "input" = Value::Fn(FnKind::Native(Rc::new(_input))));
    set_global!(interpreter: "debug" = Value::Fn(FnKind::Native(Rc::new(_debug))));
    set_global!(interpreter: "error" = Value::Fn(FnKind::Native(Rc::new(_error))));
    set_global!(interpreter: "iter" = Value::Fn(FnKind::Native(Rc::new(_iter))));
    set_global!(interpreter: "next" = Value::Fn(FnKind::Native(Rc::new(_next))));
    set_global!(interpreter: "int" = Value::Fn(FnKind::Native(Rc::new(_int))));
    set_global!(interpreter: "float" = Value::Fn(FnKind::Native(Rc::new(_float))));
    set_global!(interpreter: "bool" = Value::Fn(FnKind::Native(Rc::new(_bool))));
    set_global!(interpreter: "char" = Value::Fn(FnKind::Native(Rc::new(_char))));
    set_global!(interpreter: "str" = Value::Fn(FnKind::Native(Rc::new(_str))));
    set_global!(interpreter: "vec" = Value::Fn(FnKind::Native(Rc::new(_vec))));
    set_global!(interpreter: "tuple" = Value::Fn(FnKind::Native(Rc::new(_tuple))));
    set_global!(interpreter: "enumerate" = Value::Fn(FnKind::Native(Rc::new(_enumerate))));
}

native_fn!(_print (_i args): => {
    println!("{}", args.map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(" "));
    Ok(None)
});
native_fn!(_write (_i args): => {
    print!("{}", args.map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(" "));
    Ok(None)
});
native_fn!(_input (_i args): text = typed!(args: String) => {
    let mut input = String::new();
    print!("{text}");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    Ok(Some(Value::String(input)))
});
native_fn!(_debug (_i args): => {
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
pub struct ErrorObject {
    msg: String,
    path: Option<String>,
    ln: usize,
}
impl NativeObject for ErrorObject {
    fn typ(&self) -> &'static str {
        "error"
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
native_fn!(_error (i args): msg = typed!(args: String) => {
    Ok(Some(Value::NativeObject(Arc::new(Mutex::new(ErrorObject {
        msg,
        path: i.path().cloned(),
        ln: i.ln().unwrap_or_default(),
    })))))
});

pub struct IteratorObject {
    pub iter: Box<dyn Iterator<Item = Value>>,
    pub fn_next: Rc<NativeFn>,
}
unsafe impl Send for IteratorObject {}
unsafe impl Sync for IteratorObject {}
impl NativeObject for IteratorObject {
    fn typ(&self) -> &'static str {
        "iterator"
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
    pub fn next_(&mut self) -> Option<Value> {
        self.iter.next()
    }
    native_fn!(_next (i args): _self = typed!(args: "iterator") => {
        let mut _self = _self.lock().unwrap();
        _self.call_mut("next", i, args.map(|(_, v)| v).collect())
    });
}
native_fn!(_iter (i args): value = typed!(args) => {
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
native_fn!(_next (i args): value = typed!(args) => {
    match value {
        Value::NativeObject(object) => {
            object.lock().unwrap().call_mut("next", i, args.map(|(_, v)| v).collect())
        }
        value => Err(format!("can't get next iteration of {}", value.typ()).into())
    }
});

native_fn!(_int (_i args): value = typed!(args) => {
    Ok(Some(Value::Int(match value {
        Value::Int(v) => v,
        Value::Float(v) => v as i64,
        Value::Bool(v) => if v { 1 } else { 0 },
        Value::Char(v) => v as u8 as i64,
        Value::String(v) => if let Ok(v) = v.parse::<i64>() { v } else { return Ok(None); },
        _ => return Ok(None)
    })))
});
native_fn!(_float (_i args): value = typed!(args) => {
    Ok(Some(Value::Float(match value {
        Value::Int(v) => v as f64,
        Value::Float(v) => v,
        Value::Bool(v) => if v { 1.0 } else { 0.0 },
        Value::Char(v) => v as u8 as f64,
        Value::String(v) => if let Ok(v) = v.parse::<f64>() { v } else { return Ok(None); },
        _ => return Ok(None)
    })))
});
native_fn!(_bool (_i args): value = typed!(args) => {
    Ok(Some(Value::Bool(bool::from(value))))
});
native_fn!(_char (_i args): value = typed!(args) => {
    Ok(Some(Value::Char(match value {
        Value::Int(v) => if let Ok(v) = TryInto::<u8>::try_into(v) { v as char } else { todo!() },
        Value::Float(v) => if let Ok(v) = TryInto::<u8>::try_into(v as i64) { v as char } else { todo!() },
        Value::Char(v) => v,
        _ => return Ok(None)
    })))
});
native_fn!(_str (_i args): => {
    Ok(Some(Value::String(args.map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(""))))
});
native_fn!(_vec (_i args): value = typed!(args) => {
    if args.len() == 0 {
        Ok(Some(Value::Vector(Arc::new(Mutex::new(match value {
            Value::Vector(arc) => arc.lock().unwrap().clone(),
            Value::Tuple(arc) => arc.lock().unwrap().to_vec(),
            Value::Map(arc) => arc
                .lock()
                .unwrap()
                .iter()
                .map(|(k, v)| Value::Tuple(Arc::new(Mutex::new(Box::new([Value::String(k.clone()), v.clone()])))))
                .collect(),
            value => vec![value],
        })))))
    } else {
        let mut values: Vec<Value> = args.map(|(_, v)| v).collect();
        values.insert(0, value);
        Ok(Some(Value::Vector(Arc::new(Mutex::new(values)))))
    }
});
native_fn!(_tuple (_i args): value = typed!(args) => {
    if args.len() == 0 {
        Ok(Some(Value::Tuple(Arc::new(Mutex::new(match value {
            Value::Vector(arc) => arc.lock().unwrap().clone().into_boxed_slice(),
            Value::Tuple(arc) => arc.lock().unwrap().clone(),
            Value::Map(arc) => arc
                .lock()
                .unwrap()
                .iter()
                .map(|(k, v)| Value::Tuple(Arc::new(Mutex::new(Box::new([Value::String(k.clone()), v.clone()])))))
                .collect(),
            value => Box::new([value]),
        })))))
    } else {
        let mut values: Vec<Value> = args.map(|(_, v)| v).collect();
        values.insert(0, value);
        Ok(Some(Value::Vector(Arc::new(Mutex::new(values)))))
    }
});
native_fn!(_enumerate (i args): value = typed!(args) => {
    match value {
        Value::Vector(values) => {
            Ok(Some(Value::NativeObject(Arc::new(Mutex::new(IteratorObject {
                iter: Box::new(values
                    .lock()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| Value::Tuple(Arc::new(Mutex::new(Box::new([Value::Int(i as i64), v])))))
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
                    .map(|(i, v)| Value::Tuple(Arc::new(Mutex::new(Box::new([Value::Int(i as i64), v])))))
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
                    .map(|(i, v)| Value::Tuple(Arc::new(Mutex::new(Box::new([Value::Int(i as i64), Value::String(v)])))))
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
                    .map(|(i, v)| Value::Tuple(Arc::new(Mutex::new(Box::new([Value::Int(i as i64), Value::Char(v as char)])))))
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
