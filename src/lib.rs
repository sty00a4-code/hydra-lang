#![feature(integer_sign_cast)]
use run::{
    compiler::{Compilable, Compiler, Frame, Scope},
    interpreter::Interpreter,
    value::{Function, Value},
};
use scan::{
    ast::Chunk,
    lexer::{Lexer, Line},
    parser::{Parsable, Parser},
    position::{Located, Position},
};
use std::{error::Error, rc::Rc};

#[cfg(test)]
mod tests;

pub mod run;
pub mod scan;
pub mod std_hydra;

pub fn lex(text: &str) -> Result<Vec<Line>, Located<Box<dyn Error>>> {
    Lexer::from(text)
        .lex()
        .map_err(|Located { value: err, pos }| Located::new(err.into(), pos))
}

pub fn parse<N: Parsable>(text: &str) -> Result<Located<N>, Located<Box<dyn Error>>>
where
    <N as scan::parser::Parsable>::Error: 'static,
{
    let lines = lex(text)?;
    let mut parser = Parser::new(lines);
    N::parse(&mut parser).map_err(|Located { value: err, pos }| Located::new(err.into(), pos))
}

pub fn compile<N: Parsable>(
    text: &str,
    path: Option<String>,
) -> Result<<Located<N> as Compilable>::Output, Located<Box<dyn Error>>>
where
    <N as scan::parser::Parsable>::Error: 'static,
    Located<N>: Compilable,
{
    let ast = parse::<N>(text)?;
    let mut compiler = Compiler {
        path,
        frame_stack: vec![Frame {
            scopes: vec![Scope::default()],
            ..Default::default()
        }],
    };
    Ok(ast.compile(&mut compiler))
}

pub fn run(
    text: &str,
    args: Vec<Value>,
    path: Option<String>,
) -> Result<Option<Value>, Located<Box<dyn Error>>> {
    let closure = compile::<Chunk>(text, path)?;
    let mut interpreter = Interpreter::default();
    interpreter
        .call(
            &Function {
                closure: Rc::new(closure),
            },
            args,
            None,
        )
        .map_err(|err| Located {
            value: err.err.into(),
            pos: Position::new(err.ln..err.ln, 0..0),
        })?;
    interpreter.run().map_err(|err| Located {
        value: err.err.into(),
        pos: Position::new(err.ln..err.ln, 0..0),
    })
}

#[macro_export]
macro_rules! set_global {
    ($interpreter:ident: $key:literal = $value:expr) => {{
        use std::sync::{Arc, Mutex};
        $interpreter
            .globals
            .insert($key.into(), Arc::new(Mutex::new($value)))
    }};
    ($interpreter:ident: $key:ident = $value:expr) => {{
        use std::sync::{Arc, Mutex};
        $interpreter
            .globals
            .insert($key.into(), Arc::new(Mutex::new($value)))
    }};
}
#[macro_export]
macro_rules! typed {
    ($args:ident) => {{
        $args.next().map(|(_, v)| v).unwrap_or_default()
    }};
    ($args:ident: $typ:literal ?) => {{
        let (idx, arg) = $args.next().unwrap_or(($args.len(), Value::default()));
        if arg == Value::default() {
            None
        } else {
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
        }
    }};
    ($args:ident: $typ:ident ?) => {{
        let (idx, arg) = $args.next().unwrap_or(($args.len(), Value::default()));
        if arg == Value::default() {
            None
        } else if let Value::$typ(value) = arg {
            Some(value)
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
    ($args:ident: Fn) => {{
        let Some((idx, arg)) = $args.next() else {
            return Err(format!(
                "expected fn for argument #last, got {}",
                Value::default().typ()
            )
            .into());
        };
        if let Value::Fn(value) = arg {
            value
        } else {
            return Err(format!(
                "expected fn for argument #{}, got {}",
                idx + 1,
                arg.typ()
            )
            .into());
        }
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
    ($args:ident: $typ:expr) => {{
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
}
#[macro_export]
macro_rules! define_native_fn {
    ($fn_name:ident ($interpreter:ident $args:ident!) $body:block) => {
        pub fn $fn_name($interpreter: &mut Interpreter, $args: Vec<Value>) -> Result<Option<Value>, Box<dyn Error>> {
            $body
        }
    };
    ($fn_name:ident ($interpreter:ident $args:ident): $($name:pat = $macro:expr),* $(,) * => $body:block) => {
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
#[macro_export]
macro_rules! native_fn {
    ($name:ident) => {{
        use run::value::FnKind;
        use std::rc::Rc;
        Value::Fn(FnKind::Native(Rc::new($name)))
    }};
}
#[macro_export]
macro_rules! make_vec {
    ($value:expr) => {{
        use std::sync::{Arc, Mutex};
        Value::Vector(Arc::new(Mutex::new($value)))
    }};
    ($($value:expr),* $(,) *) => {{
        use std::sync::{Arc, Mutex};
        Value::Vector(Arc::new(Mutex::new(vec![$($value),*])))
    }};
}
#[macro_export]
macro_rules! make_tuple {
    ($value:expr) => {{
        use std::sync::{Arc, Mutex};
        Value::Tuple(Arc::new(Mutex::new($value.into())))
    }};
    ($($value:expr),* $(,) *) => {{
        use std::sync::{Arc, Mutex};
        Value::Tuple(Arc::new(Mutex::new(Box::new([$($value.into()),*]))))
    }};
}
#[macro_export]
macro_rules! make_map {
    ($($key:literal = $value:expr),* $(,) *) => {{
        use std::sync::{Arc, Mutex};
        use std::collections::HashMap;
        #[allow(unused_mut)]
        let mut map = HashMap::new();
        $(
            map.insert($key.into(), $value.into());
        ) *
        Value::Map(Arc::new(Mutex::new(map)))
    }};
    ($value:expr) => {{
        use std::sync::{Arc, Mutex};
        Value::Map(Arc::new(Mutex::new($value.into())))
    }};
}
