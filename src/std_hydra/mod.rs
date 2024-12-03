use crate::run::{
    interpreter::Interpreter,
    value::{FnKind, NativeObject, Value},
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
    ($fn_name:ident ($interpreter:ident $args:ident): $($typ:ident $name:ident),* $body:block) => {
        pub fn $fn_name($interpreter: &mut Interpreter, $args: Vec<Value>) -> Result<Option<Value>, Box<dyn Error>> {
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            let mut $args = $args.into_iter().enumerate();
            $(
                let $name = typed!($args: $typ);
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
}

native_fn!(_print (_i args): {
    println!("{}", args.map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(" "));
    Ok(None)
});
native_fn!(_write (_i args): {
    print!("{}", args.map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(" "));
    Ok(None)
});
native_fn!(_input (_i args): String text {
    let mut input = String::new();
    print!("{text}");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    Ok(Some(Value::String(input)))
});
native_fn!(_debug (_i args): {
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
native_fn!(_error (i args): String msg {
    Ok(Some(Value::NativeObject(Arc::new(Mutex::new(ErrorObject {
        msg,
        path: i.path().cloned(),
        ln: i.ln().unwrap_or_default(),
    })))))
});
