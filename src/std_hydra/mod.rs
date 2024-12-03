use crate::run::{
    interpreter::Interpreter,
    value::{FnKind, Value},
};
use std::{
    error::Error,
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
}

native_fn!(_print (_i args): {
    println!("{}", args.map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(" "));
    Ok(None)
});
native_fn!(_write (_i args): {
    print!("{}", args.map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(" "));
    Ok(None)
});