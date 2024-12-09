use std::{
    fs,
    io::{Read, Write},
    sync::{Arc, Mutex},
};

use run::{
    interpreter::RunTimeErrorKind,
    value::{FnKind, NativeFn, NativeObject},
};

use super::run::interpreter::Interpreter;
use crate::*;

pub fn import(interpreter: &mut Interpreter) {
    set_global!(interpreter: "fs" = make_map!{
        "open" = native_fn!(_open),
        "list" = native_fn!(_list),
        "meta_data" = native_fn!(_meta_data)
    });
}

pub struct FileObject {
    pub file: fs::File,
    pub fn_read: Rc<NativeFn>,
    pub fn_write: Rc<NativeFn>,
    pub fn_meta_data: Rc<NativeFn>,
}
impl FileObject {
    pub const TYPE: &'static str = "file";
    define_native_fn!(_read (i args): _self = typed!(args: Self::TYPE) => {
        let mut _self = _self.lock().unwrap();
        _self.call_mut("read", i, args.map(|(_, v)| v).collect())
    });
    pub fn read_(
        &mut self,
        _i: &mut Interpreter,
        _args: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        let mut content = String::new();
        self.file.read_to_string(&mut content)?;
        Ok(Some(Value::String(content)))
    }
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
        Ok(Some(self.file.write(text.as_bytes())?.into()))
    }
    define_native_fn!(_meta_data (i args): _self = typed!(args: Self::TYPE) => {
        let mut _self = _self.lock().unwrap();
        _self.call_mut("meta_data", i, args.map(|(_, v)| v).collect())
    });
    pub fn meta_data_(
        &mut self,
        _i: &mut Interpreter,
        _args: Vec<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        Ok(self
            .file
            .metadata()
            .ok()
            .map(|data| Value::NativeObject(Arc::new(Mutex::new(MetaDataObject { data })))))
    }
}
impl NativeObject for FileObject {
    fn typ(&self) -> &'static str {
        Self::TYPE
    }
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "read" => Some(Value::Fn(FnKind::Native(Rc::clone(&self.fn_read)))),
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
            "read" => self.read_(interpreter, args),
            "write" => self.write_(interpreter, args),
            _ => Err(RunTimeErrorKind::CannotCall(Value::default().typ())
                .to_string()
                .into()),
        }
    }
}
unsafe impl Sync for FileObject {}
unsafe impl Send for FileObject {}
define_native_fn!(_open (_i args): path = typed!(args: String), options = typed!(args: String) => {
    let Ok(file) = fs::File::options()
        .create(options.contains('w'))
        .write(options.contains('w'))
        .read(options.contains('r'))
        .open(path) else {
        return Ok(None)
    };
    Ok(Some(Value::NativeObject(Arc::new(Mutex::new(FileObject {
        file,
        fn_read: Rc::new(FileObject::_read),
        fn_write: Rc::new(FileObject::_write),
        fn_meta_data: Rc::new(FileObject::_meta_data),
    })))))
});
define_native_fn!(_list (_i args): path = typed!(args: String) => {
    Ok(Some(Value::Vector(Arc::new(Mutex::new(
        fs::read_dir(path)?
            .flatten()
            .map(|entry| Value::String(
                entry
                    .file_name()
                    .to_str()
                    .unwrap_or_default()
                    .to_string()
            )).collect()
    )))))
});
define_native_fn!(_meta_data (_i args): path = typed!(args: String) => {
    Ok(fs::metadata(path)
        .ok()
        .map(|data| Value::NativeObject(Arc::new(Mutex::new(MetaDataObject { data })))))
});

#[derive(Debug, Clone)]
pub struct MetaDataObject {
    pub data: fs::Metadata,
}
impl MetaDataObject {
    pub const TYPE: &str = "meta-data";
}
impl NativeObject for MetaDataObject {
    fn typ(&self) -> &'static str {
        Self::TYPE
    }
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "type" => Some(
                {
                    let file_type = self.data.file_type();
                    if file_type.is_dir() {
                        "dir"
                    } else if file_type.is_symlink() {
                        "symlink"
                    } else {
                        "file"
                    }
                }
                .to_string()
                .into(),
            ),
            "len" => Some(self.data.len().into()),
            _ => None,
        }
    }
}
