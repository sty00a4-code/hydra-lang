use std::collections::HashMap;

use super::code::Closure;

pub struct Compiler {
    pub frame_stack: Vec<Frame>,
}
#[derive(Debug, Default)]
pub struct Frame {
    pub closure: Closure,
    pub registers: u8,
    pub scopes: Vec<Scope>,
    pub max_registers: u8,
}
#[derive(Debug, Default)]
pub struct Scope {
    pub locals: HashMap<String, u8>,
    pub offset: u8,
}

impl Compiler {
    pub fn push_frame(&mut self) {
        self.frame_stack.push(Frame {
            scopes: vec![Scope::default()],
            ..Default::default()
        });
    }
    pub fn pop_frame(&mut self) -> Option<Frame> {
        self.frame_stack.pop()
    }
    pub fn frame(&self) -> Option<&Frame> {
        self.frame_stack.last()
    }
    pub fn frame_mut(&mut self) -> Option<&mut Frame> {
        self.frame_stack.last_mut()
    }
}
impl Frame {
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            offset: self.registers,
            ..Default::default()
        });
    }
    pub fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            self.registers = scope.offset;
        }
    }
    pub fn scope(&self) -> Option<&Scope> {
        self.scopes.last()
    }
    pub fn scope_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }
    pub fn new_register(&mut self) -> u8 {
        self.registers += 1;
        if self.max_registers < self.registers {
            self.max_registers = self.registers
        }
        self.registers
    }
    pub fn get_local(&self, name: &str) -> Option<u8> {
        for scope in self.scopes.iter().rev() {
            if let Some(register) = scope.locals.get(name) {
                return Some(*register);
            }
        }
        None
    }
    pub fn new_local(&mut self, name: String) -> u8 {
        if let Some(register) = self.get_local(&name) {
            return register;
        }
        let register = self.new_register();
        self.scope_mut()
            .and_then(|scope| scope.locals.insert(name, register));
        register
    }
}
