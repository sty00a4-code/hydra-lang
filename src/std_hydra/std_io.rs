use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};

use run::interpreter::RunTimeErrorKind;
use run::value::{FnKind, NativeFn, NativeObject};

use crate::run::interpreter::Interpreter;
use crate::*;

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: "io" = make_map!{
        "stdin" = native_fn!(_stdin),
        "stdout" = native_fn!(_stdout),
        "stderr" = native_fn!(_stderr),
        "write" = native_fn!(_write),
    });
}

pub struct StdinObject {
    stdin: io::Stdin,
    fn_read: Rc<NativeFn>,
    fn_read_line: Rc<NativeFn>,
}
impl StdinObject {
    pub const TYPE: &str = "stdin";
    define_native_fn!(_read (i args): _self = typed!(args: Self::TYPE) => {
        let mut _self = _self.lock().unwrap();
        _self.call_mut("read", i, args.map(|(_, v)| v).collect())
    });
    pub fn read_(
        &mut self,
        _i: &mut Interpreter,
        _args: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        let mut buf = String::new();
        self.stdin.read_to_string(&mut buf)?;
        Ok(Some(buf.into()))
    }
    define_native_fn!(_read_line (i args): _self = typed!(args: Self::TYPE) => {
        let mut _self = _self.lock().unwrap();
        _self.call_mut("read_line", i, args.map(|(_, v)| v).collect())
    });
    pub fn read_line_(
        &mut self,
        _i: &mut Interpreter,
        _args: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        let mut buf = String::new();
        self.stdin.read_line(&mut buf)?;
        Ok(Some(buf.into()))
    }
}
impl NativeObject for StdinObject {
    fn typ(&self) -> &'static str {
        Self::TYPE
    }
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "read" => Some(Value::Fn(FnKind::Native(Rc::clone(&self.fn_read)))),
            "read_line" => Some(Value::Fn(FnKind::Native(Rc::clone(&self.fn_read_line)))),
            _ => None,
        }
    }
    fn call_mut(
        &mut self,
        key: &str,
        interpreter: &mut Interpreter,
        args: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        match key {
            "read" => self.read_(interpreter, args),
            "read_line" => self.read_line_(interpreter, args),
            _ => Err(RunTimeErrorKind::CannotCall(Value::default().typ())
                .to_string()
                .into()),
        }
    }
}
unsafe impl Sync for StdinObject {}
unsafe impl Send for StdinObject {}
define_native_fn!(_stdin (_i args): => {
    Ok(Some(Value::NativeObject(Arc::new(Mutex::new(StdinObject {
        stdin: io::stdin(),
        fn_read: Rc::new(StdinObject::_read),
        fn_read_line: Rc::new(StdinObject::_read_line),
    })))))
});
pub struct StdoutObject {
    stdout: io::Stdout,
    fn_write: Rc<NativeFn>,
}
impl StdoutObject {
    pub const TYPE: &str = "stdout";
    define_native_fn!(_write (i args): _self = typed!(args: Self::TYPE) => {
        let mut _self = _self.lock().unwrap();
        _self.call_mut("write", i, args.map(|(_, v)| v).collect())
    });
    pub fn write_(
        &mut self,
        _i: &mut Interpreter,
        args: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        let mut args = args.into_iter().enumerate();
        let text = typed!(args: String);
        Ok(Some(self.stdout.write(text.as_bytes())?.into()))
    }
}
impl NativeObject for StdoutObject {
    fn typ(&self) -> &'static str {
        Self::TYPE
    }
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "write" => Some(Value::Fn(FnKind::Native(Rc::clone(&self.fn_write)))),
            _ => None,
        }
    }
    fn call_mut(
        &mut self,
        key: &str,
        interpreter: &mut Interpreter,
        args: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        match key {
            "write" => self.write_(interpreter, args),
            _ => Err(RunTimeErrorKind::CannotCall(Value::default().typ())
                .to_string()
                .into()),
        }
    }
}
unsafe impl Sync for StdoutObject {}
unsafe impl Send for StdoutObject {}
define_native_fn!(_stdout (_i args): => {
    Ok(Some(Value::NativeObject(Arc::new(Mutex::new(StdoutObject {
        stdout: io::stdout(),
        fn_write: Rc::new(StdoutObject::_write),
    })))))
});
pub struct StderrObject {
    stderr: io::Stderr,
    fn_write: Rc<NativeFn>,
}
impl StderrObject {
    pub const TYPE: &str = "stderr";
    define_native_fn!(_write (i args): _self = typed!(args: Self::TYPE) => {
        let mut _self = _self.lock().unwrap();
        _self.call_mut("write", i, args.map(|(_, v)| v).collect())
    });
    pub fn write_(
        &mut self,
        _i: &mut Interpreter,
        args: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        let mut args = args.into_iter().enumerate();
        let text = typed!(args: String);
        Ok(Some(self.stderr.write(text.as_bytes())?.into()))
    }
}
impl NativeObject for StderrObject {
    fn typ(&self) -> &'static str {
        Self::TYPE
    }
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "write" => Some(Value::Fn(FnKind::Native(Rc::clone(&self.fn_write)))),
            _ => None,
        }
    }
    fn call_mut(
        &mut self,
        key: &str,
        interpreter: &mut Interpreter,
        args: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        match key {
            "write" => self.write_(interpreter, args),
            _ => Err(RunTimeErrorKind::CannotCall(Value::default().typ())
                .to_string()
                .into()),
        }
    }
}
unsafe impl Sync for StderrObject {}
unsafe impl Send for StderrObject {}
define_native_fn!(_stderr (_i args): => {
    Ok(Some(Value::NativeObject(Arc::new(Mutex::new(StderrObject {
        stderr: io::stderr(),
        fn_write: Rc::new(StderrObject::_write),
    })))))
});

define_native_fn!(_write (_i args): => {
    print!("{}", args.map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(" "));
    Ok(None)
});
