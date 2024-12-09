use super::{
    code::{ByteCode, Closure, Location, Source},
    value::Value,
};
use crate::scan::{
    ast::{
        AssignOperator, Atom, BinaryOperator, Block, Chunk, Expression, Parameter, Path, Statement,
    },
    position::Located,
};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Debug, Default)]
pub struct Compiler {
    pub path: Option<String>,
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
    pub breaks: HashSet<usize>,
    pub continues: HashSet<usize>,
}

impl Compiler {
    pub fn push_frame(&mut self, path: Option<String>, name: Option<String>) {
        self.frame_stack.push(Frame {
            closure: Closure {
                path,
                name,
                ..Default::default()
            },
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
    pub fn new_constant(&mut self, value: Value) -> u16 {
        let frame = self.frame_mut().unwrap();
        if let Some(addr) = frame.closure.constants.iter().position(|v| v == &value) {
            return addr as u16;
        }
        let addr = frame.closure.constants.len() as u16;
        frame.closure.constants.push(value);
        addr
    }
    pub fn new_closure(&mut self, closure: Rc<Closure>) -> u16 {
        let frame = self.frame_mut().unwrap();
        let addr = frame.closure.closures.len() as u16;
        frame.closure.closures.push(closure);
        addr
    }
    pub fn addr(&self) -> usize {
        self.frame().unwrap().closure.code.len()
    }
    pub fn write(&mut self, bytecode: ByteCode, ln: usize) -> usize {
        let frame = self.frame_mut().unwrap();
        let addr = frame.closure.code.len();
        frame.closure.code.push(bytecode);
        frame.closure.lines.push(ln);
        addr
    }
    pub fn overwrite(&mut self, addr: usize, bytecode: ByteCode, ln: usize) {
        let frame = self.frame_mut().unwrap();
        frame.closure.code[addr] = bytecode;
        frame.closure.lines[addr] = ln;
    }
    pub fn overwrite_no_ln(&mut self, addr: usize, bytecode: ByteCode) {
        let frame = self.frame_mut().unwrap();
        frame.closure.code[addr] = bytecode;
    }
    pub fn overwrite_jump_if(
        &mut self,
        addr: usize,
        negative: bool,
        cond: Source,
        to: usize,
        ln: usize,
    ) {
        if to != addr + 1 {
            self.overwrite(
                addr,
                ByteCode::JumpIf {
                    negative,
                    cond,
                    addr: to,
                },
                ln,
            );
        }
    }
    pub fn overwrite_jump_if_some(
        &mut self,
        addr: usize,
        negative: bool,
        src: Source,
        to: usize,
        ln: usize,
    ) {
        if to != addr + 1 {
            self.overwrite(
                addr,
                ByteCode::JumpIfSome {
                    negative,
                    src,
                    addr: to,
                },
                ln,
            );
        }
    }
    pub fn overwrite_jump(&mut self, addr: usize, to: usize, ln: usize) {
        if to != addr + 1 {
            self.overwrite(addr, ByteCode::Jump { addr: to }, ln);
        }
    }
    pub fn none(&mut self) -> usize {
        self.write(ByteCode::None, 0)
    }
    pub fn alloc_break(&mut self, ln: usize) {
        let addr = self.write(ByteCode::None, ln);
        self.frame_mut().unwrap().alloc_break(addr);
    }
    pub fn alloc_continue(&mut self, ln: usize) {
        let addr = self.write(ByteCode::None, ln);
        self.frame_mut().unwrap().alloc_continue(addr);
    }
    pub fn return_safe(&mut self, ln: usize) -> usize {
        let frame = self.frame_mut().unwrap();
        if let Some(ByteCode::Return { src: _ }) = frame.closure.code.last() {
            return frame.closure.code.len() - 1;
        }
        let addr = frame.closure.code.len();
        frame.closure.code.push(ByteCode::Return { src: None });
        frame.closure.lines.push(ln);
        addr
    }
    pub fn move_checked(&mut self, dst: Location, src: Source, ln: usize) -> usize {
        if dst.eq_source(&src) {
            let addr = self.frame().unwrap().closure.code.len() - 1;
            return addr;
        }
        self.write(ByteCode::Move { dst, src }, ln)
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
            if let Some(current) = self.scope_mut() {
                current.breaks.extend(scope.breaks);
                current.continues.extend(scope.continues);
            }
        }
    }
    pub fn pop_scope_loop(&mut self) -> Option<Scope> {
        if let Some(scope) = self.scopes.pop() {
            self.registers = scope.offset;
            Some(scope)
        } else {
            None
        }
    }
    pub fn scope(&self) -> Option<&Scope> {
        self.scopes.last()
    }
    pub fn scope_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }
    pub fn new_register(&mut self) -> u8 {
        let reg = self.registers;
        self.registers += 1;
        if self.max_registers < self.registers {
            self.max_registers = self.registers;
            self.closure.registers = self.max_registers;
        }
        reg
    }
    pub fn alloc_registers(&mut self, amount: u8) -> Vec<u8> {
        let mut regs = vec![];
        for offset in 0..amount {
            regs.push(self.registers + offset);
        }
        self.registers += amount;
        if self.max_registers < self.registers {
            self.max_registers = self.registers;
            self.closure.registers = self.max_registers;
        }
        regs
    }
    pub fn get_local(&self, name: &str) -> Option<u8> {
        for scope in self.scopes.iter().rev() {
            if let Some(register) = scope.locals.get(name) {
                return Some(*register);
            }
        }
        None
    }
    pub fn set_local(&mut self, name: String, register: u8) {
        self.scope_mut().unwrap().locals.insert(name, register);
    }
    pub fn new_local(&mut self, name: String) -> u8 {
        if let Some(register) = self.get_local(&name) {
            return register;
        }
        let register = self.new_register();
        self.set_local(name, register);
        register
    }
    pub fn alloc_break(&mut self, addr: usize) {
        self.scope_mut().unwrap().breaks.insert(addr);
    }
    pub fn alloc_continue(&mut self, addr: usize) {
        self.scope_mut().unwrap().continues.insert(addr);
    }
}

pub trait Compilable: Sized {
    type Output;
    fn compile(self, compiler: &mut Compiler) -> Self::Output;
}

impl Compilable for Located<Chunk> {
    type Output = Closure;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let Located { value: chunk, pos } = self;
        let ln = pos.ln.end;
        compiler.push_frame(compiler.path.clone(), None);
        for stat in chunk.stats {
            if stat.compile(compiler).is_some() {
                break;
            }
        }
        compiler.return_safe(ln);
        compiler.pop_frame().unwrap().closure
    }
}
impl Compilable for Located<Block> {
    type Output = Option<Source>;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let Located {
            value: block,
            pos: _,
        } = self;
        compiler.frame_mut().unwrap().push_scope();
        for stat in block.stats {
            if let Some(src) = stat.compile(compiler) {
                compiler.frame_mut().unwrap().pop_scope();
                return Some(src);
            }
        }
        compiler.frame_mut().unwrap().pop_scope();
        None
    }
}
impl Compilable for Located<Statement> {
    type Output = Option<Source>;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let Located { value: stat, pos } = self;
        let ln = pos.ln.start;
        match stat {
            Statement::LetBinding {
                param:
                    Located {
                        value: param,
                        pos: _,
                    },
                expr,
            } => {
                let src = expr.compile(compiler);
                match param {
                    Parameter::Ident(ident) => {
                        let dst =
                            Location::Register(compiler.frame_mut().unwrap().new_local(ident));
                        compiler.move_checked(dst, src, ln);
                    }
                    Parameter::Vector(idents) | Parameter::Tuple(idents) => {
                        for (
                            idx,
                            Located {
                                value: ident,
                                pos: _,
                            },
                        ) in idents.into_iter().enumerate()
                        {
                            let dst =
                                Location::Register(compiler.frame_mut().unwrap().new_local(ident));
                            compiler.write(
                                ByteCode::Field {
                                    dst,
                                    head: src,
                                    field: Source::Int(idx as i64),
                                },
                                ln,
                            );
                        }
                    }
                    Parameter::Map(keys) => {
                        for Located { value: key, pos: _ } in keys {
                            let field =
                                Source::Constant(compiler.new_constant(Value::String(key.clone())));
                            let dst =
                                Location::Register(compiler.frame_mut().unwrap().new_local(key));
                            compiler.write(
                                ByteCode::Field {
                                    dst,
                                    head: src,
                                    field,
                                },
                                ln,
                            );
                        }
                    }
                }
            }
            Statement::Assign { op, path, expr } => {
                let dst = path.compile(compiler);
                let src = expr.compile(compiler);
                match op {
                    AssignOperator::None => {
                        compiler.move_checked(dst, src, ln);
                    }
                    op => {
                        compiler.write(
                            ByteCode::Binary {
                                op: TryInto::<BinaryOperator>::try_into(op).unwrap().into(),
                                dst,
                                left: dst.into(),
                                right: src,
                            },
                            ln,
                        );
                    }
                }
            }
            Statement::Fn {
                name:
                    Located {
                        value: name,
                        pos: _,
                    },
                params,
                varargs,
                body,
            } => {
                let dst = Location::Register(compiler.frame_mut().unwrap().new_local(name));
                compiler.push_frame(compiler.path.clone(), None);
                {
                    compiler
                        .frame_mut()
                        .unwrap()
                        .alloc_registers(params.len() as u8);
                    if let Some(Located {
                        value: ident,
                        pos: _,
                    }) = varargs
                    {
                        compiler.frame_mut().unwrap().new_local(ident);
                        compiler.frame_mut().unwrap().closure.varargs = true;
                    }
                    for (
                        reg,
                        Located {
                            value: param,
                            pos: param_pos,
                        },
                    ) in params.into_iter().enumerate()
                    {
                        let param_ln = param_pos.ln.start;
                        match param {
                            Parameter::Ident(ident) => {
                                compiler.frame_mut().unwrap().closure.parameters += 1;
                                compiler.frame_mut().unwrap().set_local(ident, reg as u8);
                            }
                            Parameter::Tuple(params) | Parameter::Vector(params) => {
                                for (
                                    idx,
                                    Located {
                                        value: ident,
                                        pos: _,
                                    },
                                ) in params.into_iter().enumerate()
                                {
                                    compiler.frame_mut().unwrap().closure.parameters += 1;
                                    let dst = Location::Register(
                                        compiler.frame_mut().unwrap().new_local(ident),
                                    );
                                    compiler.write(
                                        ByteCode::Field {
                                            dst,
                                            head: Source::Register(reg as u8),
                                            field: Source::Int(idx as i64),
                                        },
                                        param_ln,
                                    );
                                }
                            }
                            Parameter::Map(params) => {
                                for Located {
                                    value: ident,
                                    pos: _,
                                } in params
                                {
                                    compiler.frame_mut().unwrap().closure.parameters += 1;
                                    let dst = Location::Register(
                                        compiler.frame_mut().unwrap().new_local(ident.clone()),
                                    );
                                    let ident = compiler.new_constant(Value::String(ident));
                                    compiler.write(
                                        ByteCode::Field {
                                            dst,
                                            head: Source::Register(reg as u8),
                                            field: Source::Constant(ident),
                                        },
                                        param_ln,
                                    );
                                }
                            }
                        }
                    }
                    body.compile(compiler);
                }
                let Frame { closure, .. } = compiler.pop_frame().unwrap();
                let addr = compiler.new_closure(Rc::new(closure));
                compiler.write(ByteCode::Fn { dst, addr }, ln);
            }
            Statement::Call { head, args } => {
                let func = Source::from(head.compile(compiler));
                compiler.frame_mut().unwrap().push_scope();
                let start = compiler.frame().unwrap().registers;
                let amount = args.len() as u8;
                {
                    let registers = compiler.frame_mut().unwrap().alloc_registers(amount);
                    for (arg, reg) in args.into_iter().zip(registers) {
                        let ln = arg.pos.ln.start;
                        let arg = arg.compile(compiler);
                        compiler.move_checked(Location::Register(reg), arg, ln);
                    }
                }
                compiler.frame_mut().unwrap().pop_scope();
                compiler.write(
                    ByteCode::Call {
                        dst: None,
                        func,
                        start,
                        amount,
                    },
                    ln,
                );
            }
            Statement::SelfCall {
                head,
                field:
                    Located {
                        value: field,
                        pos: field_pos,
                    },
                args,
            } => {
                let head_ln = head.pos.ln.start;
                let head = Source::from(head.compile(compiler));
                let func = {
                    let dst = compiler.frame_mut().unwrap().new_register();
                    let field = compiler.new_constant(Value::String(field));
                    compiler.write(
                        ByteCode::Field {
                            dst: Location::Register(dst),
                            head,
                            field: Source::Constant(field),
                        },
                        field_pos.ln.start,
                    );
                    Source::Register(dst)
                };
                {
                    let dst = compiler.frame_mut().unwrap().new_register();
                    compiler.move_checked(Location::Register(dst), head, head_ln);
                }
                compiler.frame_mut().unwrap().push_scope();
                let start = compiler.frame().unwrap().registers;
                let amount = args.len() as u8;
                {
                    let registers = compiler.frame_mut().unwrap().alloc_registers(amount);
                    for (arg, reg) in args.into_iter().zip(registers) {
                        let ln = arg.pos.ln.start;
                        let arg = arg.compile(compiler);
                        compiler.move_checked(Location::Register(reg), arg, ln);
                    }
                }
                compiler.frame_mut().unwrap().pop_scope();
                compiler.write(
                    ByteCode::Call {
                        dst: None,
                        func,
                        start,
                        amount,
                    },
                    ln,
                );
            }
            Statement::Return(Some(expr)) => {
                let src = expr.compile(compiler);
                compiler.write(ByteCode::Return { src: Some(src) }, ln);
                return Some(Source::default());
            }
            Statement::Return(None) => {
                compiler.write(ByteCode::Return { src: None }, ln);
                return Some(Source::default());
            }
            Statement::If {
                cond,
                case,
                else_case,
            } => {
                compiler.frame_mut().unwrap().push_scope();
                {
                    let cond = cond.compile(compiler);
                    let jump_to_else = compiler.none();
                    case.compile(compiler);
                    let jump_to_exit = compiler.none();
                    let _else = compiler.addr();
                    if let Some(else_case) = else_case {
                        else_case.compile(compiler);
                    }
                    let exit = compiler.addr();
                    compiler.overwrite_jump_if(jump_to_else, true, cond, _else, ln);
                    compiler.overwrite_jump(jump_to_exit, exit, ln);
                }
                compiler.frame_mut().unwrap().pop_scope();
            }
            Statement::IfLet {
                param:
                    Located {
                        value: param,
                        pos: param_pos,
                    },
                expr,
                case,
                else_case,
            } => {
                compiler.frame_mut().unwrap().push_scope();
                {
                    let src = expr.compile(compiler);
                    let jump_to_else = compiler.none();
                    compiler.frame_mut().unwrap().push_scope();
                    {
                        let ln = param_pos.ln.start;
                        match param {
                            Parameter::Ident(ident) => {
                                let dst = Location::Register(
                                    compiler.frame_mut().unwrap().new_local(ident),
                                );
                                compiler.move_checked(dst, src, ln);
                            }
                            Parameter::Vector(idents) | Parameter::Tuple(idents) => {
                                for (
                                    idx,
                                    Located {
                                        value: ident,
                                        pos: _,
                                    },
                                ) in idents.into_iter().enumerate()
                                {
                                    let dst = Location::Register(
                                        compiler.frame_mut().unwrap().new_local(ident),
                                    );
                                    compiler.write(
                                        ByteCode::Field {
                                            dst,
                                            head: src,
                                            field: Source::Int(idx as i64),
                                        },
                                        ln,
                                    );
                                }
                            }
                            Parameter::Map(keys) => {
                                for Located { value: key, pos: _ } in keys {
                                    let field = Source::Constant(
                                        compiler.new_constant(Value::String(key.clone())),
                                    );
                                    let dst = Location::Register(
                                        compiler.frame_mut().unwrap().new_local(key),
                                    );
                                    compiler.write(
                                        ByteCode::Field {
                                            dst,
                                            head: src,
                                            field,
                                        },
                                        ln,
                                    );
                                }
                            }
                        }
                        case.compile(compiler);
                    }
                    compiler.frame_mut().unwrap().pop_scope();
                    let jump_to_exit = compiler.none();
                    let _else = compiler.addr();
                    if let Some(else_case) = else_case {
                        else_case.compile(compiler);
                    }
                    let exit = compiler.addr();
                    compiler.overwrite_jump_if_some(jump_to_else, true, src, _else, ln);
                    compiler.overwrite_jump(jump_to_exit, exit, ln);
                }
                compiler.frame_mut().unwrap().pop_scope();
            }
            Statement::While { cond, body } => {
                compiler.frame_mut().unwrap().push_scope();
                let start = compiler.addr();
                let cond = cond.compile(compiler);
                let jump_to_exit = compiler.none();
                body.compile(compiler);
                compiler.alloc_continue(ln);
                let exit = compiler.addr();
                compiler.overwrite_jump_if(jump_to_exit, true, cond, exit, ln);
                let scope = compiler.frame_mut().unwrap().pop_scope_loop().unwrap();
                for addr in scope.breaks {
                    if exit != addr + 1 {
                        compiler.overwrite_no_ln(addr, ByteCode::Jump { addr: exit });
                    }
                }
                for addr in scope.continues {
                    if start != addr + 1 {
                        compiler.overwrite_no_ln(addr, ByteCode::Jump { addr: start });
                    }
                }
            }
            Statement::WhileLet {
                param:
                    Located {
                        value: param,
                        pos: param_pos,
                    },
                expr,
                body,
            } => {
                compiler.frame_mut().unwrap().push_scope();
                let start = compiler.addr();
                let src = expr.compile(compiler);
                let jump_to_exit = compiler.none();
                {
                    let ln = param_pos.ln.start;
                    match param {
                        Parameter::Ident(ident) => {
                            let dst =
                                Location::Register(compiler.frame_mut().unwrap().new_local(ident));
                            compiler.move_checked(dst, src, ln);
                        }
                        Parameter::Vector(idents) | Parameter::Tuple(idents) => {
                            for (
                                idx,
                                Located {
                                    value: ident,
                                    pos: _,
                                },
                            ) in idents.into_iter().enumerate()
                            {
                                let dst = Location::Register(
                                    compiler.frame_mut().unwrap().new_local(ident),
                                );
                                compiler.write(
                                    ByteCode::Field {
                                        dst,
                                        head: src,
                                        field: Source::Int(idx as i64),
                                    },
                                    ln,
                                );
                            }
                        }
                        Parameter::Map(keys) => {
                            for Located { value: key, pos: _ } in keys {
                                let field = Source::Constant(
                                    compiler.new_constant(Value::String(key.clone())),
                                );
                                let dst = Location::Register(
                                    compiler.frame_mut().unwrap().new_local(key),
                                );
                                compiler.write(
                                    ByteCode::Field {
                                        dst,
                                        head: src,
                                        field,
                                    },
                                    ln,
                                );
                            }
                        }
                    }
                }
                body.compile(compiler);
                compiler.alloc_continue(ln);
                let exit = compiler.addr();
                compiler.overwrite_jump_if_some(jump_to_exit, true, src, exit, ln);
                let scope = compiler.frame_mut().unwrap().pop_scope_loop().unwrap();
                for addr in scope.breaks {
                    compiler.overwrite_no_ln(addr, ByteCode::Jump { addr: exit });
                }
                for addr in scope.continues {
                    compiler.overwrite_no_ln(addr, ByteCode::Jump { addr: start });
                }
            }
            Statement::For {
                param:
                    Located {
                        value: param,
                        pos: param_pos,
                    },
                iter,
                body,
            } => {
                compiler.frame_mut().unwrap().push_scope();
                let iter = {
                    let dst = Location::Register(compiler.frame_mut().unwrap().new_register());
                    let iter = iter.compile(compiler);
                    let arg_reg = compiler.frame_mut().unwrap().new_register();
                    let arg_dst = Location::Register(arg_reg);
                    compiler.move_checked(arg_dst, iter, ln);
                    let func = Source::Global(compiler.new_constant(Value::String("iter".into())));
                    compiler.write(
                        ByteCode::Call {
                            dst: Some(dst),
                            func,
                            start: arg_reg,
                            amount: 1,
                        },
                        ln,
                    );
                    dst.into()
                };
                let start = compiler.addr();
                let dst_reg = compiler.frame_mut().unwrap().new_register();
                let src = Source::Register(dst_reg);
                let dst = Location::Register(dst_reg);
                {
                    let arg_reg = compiler.frame_mut().unwrap().new_register();
                    let arg_dst = Location::Register(arg_reg);
                    compiler.move_checked(arg_dst, iter, ln);
                    let next = Source::Global(compiler.new_constant(Value::String("next".into())));
                    compiler.write(
                        ByteCode::Call {
                            dst: Some(dst),
                            func: next,
                            start: arg_reg,
                            amount: 1,
                        },
                        ln,
                    );
                }
                let jump_to_exit = compiler.none();
                {
                    let ln = param_pos.ln.start;
                    match param {
                        Parameter::Ident(ident) => {
                            let dst =
                                Location::Register(compiler.frame_mut().unwrap().new_local(ident));
                            compiler.move_checked(dst, src, ln);
                        }
                        Parameter::Vector(idents) | Parameter::Tuple(idents) => {
                            for (
                                idx,
                                Located {
                                    value: ident,
                                    pos: _,
                                },
                            ) in idents.into_iter().enumerate()
                            {
                                let dst = Location::Register(
                                    compiler.frame_mut().unwrap().new_local(ident),
                                );
                                compiler.write(
                                    ByteCode::Field {
                                        dst,
                                        head: src,
                                        field: Source::Int(idx as i64),
                                    },
                                    ln,
                                );
                            }
                        }
                        Parameter::Map(keys) => {
                            for Located { value: key, pos: _ } in keys {
                                let field = Source::Constant(
                                    compiler.new_constant(Value::String(key.clone())),
                                );
                                let dst = Location::Register(
                                    compiler.frame_mut().unwrap().new_local(key),
                                );
                                compiler.write(
                                    ByteCode::Field {
                                        dst,
                                        head: src,
                                        field,
                                    },
                                    ln,
                                );
                            }
                        }
                    }
                }
                body.compile(compiler);
                compiler.alloc_continue(ln);
                let exit = compiler.addr();
                compiler.overwrite_jump_if_some(jump_to_exit, true, src, exit, ln);
                let scope = compiler.frame_mut().unwrap().pop_scope_loop().unwrap();
                for addr in scope.breaks {
                    compiler.overwrite_no_ln(addr, ByteCode::Jump { addr: exit });
                }
                for addr in scope.continues {
                    compiler.overwrite_no_ln(addr, ByteCode::Jump { addr: start });
                }
            }
            Statement::Continue => {
                compiler.alloc_continue(ln);
            }
            Statement::Break => {
                compiler.alloc_break(ln);
            }
        }
        None
    }
}
impl Compilable for Located<Expression> {
    type Output = Source;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let Located { value: expr, pos } = self;
        let ln = pos.ln.start;
        match expr {
            Expression::Atom(atom) => Located::new(atom, pos).compile(compiler),
            Expression::Call { head, args } => {
                let func = head.compile(compiler);
                compiler.frame_mut().unwrap().push_scope();
                let start = compiler.frame().unwrap().registers;
                let amount = args.len() as u8;
                {
                    let registers = compiler.frame_mut().unwrap().alloc_registers(amount);
                    for (arg, reg) in args.into_iter().zip(registers) {
                        let ln = arg.pos.ln.start;
                        let arg = arg.compile(compiler);
                        compiler.move_checked(Location::Register(reg), arg, ln);
                    }
                }
                compiler.frame_mut().unwrap().pop_scope();
                let dst = compiler.frame_mut().unwrap().new_register();
                compiler.write(
                    ByteCode::Call {
                        dst: Some(Location::Register(dst)),
                        func,
                        start,
                        amount,
                    },
                    ln,
                );
                Source::Register(dst)
            }
            Expression::SelfCall {
                head,
                field:
                    Located {
                        value: field,
                        pos: field_pos,
                    },
                args,
            } => {
                let head_ln = head.pos.ln.start;
                let head = head.compile(compiler);
                let func = {
                    let dst = compiler.frame_mut().unwrap().new_register();
                    let field = compiler.new_constant(Value::String(field));
                    compiler.write(
                        ByteCode::Field {
                            dst: Location::Register(dst),
                            head,
                            field: Source::Constant(field),
                        },
                        field_pos.ln.start,
                    );
                    Source::Register(dst)
                };
                {
                    let dst = compiler.frame_mut().unwrap().new_register();
                    compiler.move_checked(Location::Register(dst), head, head_ln);
                }
                compiler.frame_mut().unwrap().push_scope();
                let start = compiler.frame().unwrap().registers;
                let amount = args.len() as u8;
                {
                    let registers = compiler.frame_mut().unwrap().alloc_registers(amount);
                    for (arg, reg) in args.into_iter().zip(registers) {
                        let ln = arg.pos.ln.start;
                        let arg = arg.compile(compiler);
                        compiler.move_checked(Location::Register(reg), arg, ln);
                    }
                }
                compiler.frame_mut().unwrap().pop_scope();
                let dst = compiler.frame_mut().unwrap().new_register();
                compiler.write(
                    ByteCode::Call {
                        dst: Some(Location::Register(dst)),
                        func,
                        start,
                        amount,
                    },
                    ln,
                );
                Source::Register(dst)
            }
            Expression::Field {
                head,
                field:
                    Located {
                        value: field,
                        pos: _,
                    },
            } => {
                let head = head.compile(compiler);
                let field = compiler.new_constant(Value::String(field));
                let dst = compiler.frame_mut().unwrap().new_register();
                compiler.write(
                    ByteCode::Field {
                        dst: Location::Register(dst),
                        head,
                        field: Source::Constant(field),
                    },
                    ln,
                );
                Source::Register(dst)
            }
            Expression::Index { head, index } => {
                let head = head.compile(compiler);
                let field = index.compile(compiler);
                let dst = compiler.frame_mut().unwrap().new_register();
                compiler.write(
                    ByteCode::Field {
                        dst: Location::Register(dst),
                        head,
                        field,
                    },
                    ln,
                );
                Source::Register(dst)
            }
            Expression::Binary { op, left, right } => {
                let left = left.compile(compiler);
                let right = right.compile(compiler);
                let dst = Location::Register(compiler.frame_mut().unwrap().new_register());
                compiler.write(
                    ByteCode::Binary {
                        op: op.into(),
                        dst,
                        left,
                        right,
                    },
                    ln,
                );
                Source::from(dst)
            }
            Expression::Unary { op, right } => {
                let right = right.compile(compiler);
                let dst = Location::Register(compiler.frame_mut().unwrap().new_register());
                compiler.write(
                    ByteCode::Unary {
                        op: op.into(),
                        dst,
                        right,
                    },
                    ln,
                );
                Source::from(dst)
            }
        }
    }
}
impl Compilable for Located<Atom> {
    type Output = Source;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let Located { value: expr, pos } = self;
        let ln = pos.ln.start;
        match expr {
            Atom::Path(path) => Located::new(path, pos).compile(compiler).into(),
            Atom::Null => Source::Null,
            Atom::Int(v) => Source::Int(v),
            Atom::Float(v) => Source::Float(v),
            Atom::Bool(v) => Source::Bool(v),
            Atom::Char(v) => Source::Char(v),
            Atom::String(v) => Source::Constant(compiler.new_constant(Value::String(v))),
            Atom::Tuple(exprs) => {
                let dst = compiler.frame_mut().unwrap().new_register();
                let amount = exprs.len() as u8;
                let registers = compiler.frame().unwrap().registers;
                let start = registers;
                for expr in exprs {
                    let ln = expr.pos.ln.start;
                    let dst = compiler.frame_mut().unwrap().new_register();
                    let src = expr.compile(compiler);
                    compiler.move_checked(Location::Register(dst), src, ln);
                }
                compiler.write(
                    ByteCode::Tuple {
                        dst: Location::Register(dst),
                        start,
                        amount,
                    },
                    ln,
                );
                compiler.frame_mut().unwrap().registers = registers;
                Source::Register(dst)
            }
            Atom::Vector(exprs) => {
                let dst = compiler.frame_mut().unwrap().new_register();
                let amount = exprs.len() as u8;
                let registers = compiler.frame().unwrap().registers;
                let start = registers;
                for expr in exprs {
                    let ln = expr.pos.ln.start;
                    let dst = compiler.frame_mut().unwrap().new_register();
                    let src = expr.compile(compiler);
                    compiler.move_checked(Location::Register(dst), src, ln);
                }
                compiler.write(
                    ByteCode::Vector {
                        dst: Location::Register(dst),
                        start,
                        amount,
                    },
                    ln,
                );
                compiler.frame_mut().unwrap().registers = registers;
                Source::Register(dst)
            }
            Atom::Map(pairs) => {
                let dst = compiler.frame_mut().unwrap().new_register();
                compiler.write(
                    ByteCode::Map {
                        dst: Location::Register(dst),
                    },
                    ln,
                );
                let registers = compiler.frame().unwrap().registers;
                for (
                    Located {
                        value: field,
                        pos,
                    },
                    expr,
                ) in pairs
                {
                    let ln = pos.ln.start;
                    let src = expr.compile(compiler);
                    let field = Source::Constant(compiler.new_constant(Value::String(field)));
                    compiler.write(
                        ByteCode::SetField {
                            head: Source::Register(dst),
                            field,
                            src,
                        },
                        ln,
                    );
                }
                compiler.frame_mut().unwrap().registers = registers;
                Source::Register(dst)
            }
            Atom::Expression(expr) => expr.compile(compiler),
        }
    }
}
impl Compilable for Located<Path> {
    type Output = Location;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let Located { value: path, pos } = self;
        let ln = pos.ln.start;
        match path {
            Path::Ident(ident) => {
                if let Some(reg) = compiler.frame().unwrap().get_local(&ident) {
                    Location::Register(reg)
                } else {
                    let addr = compiler.new_constant(Value::String(ident));
                    Location::Global(addr)
                }
            }
            Path::Field {
                head,
                field:
                    Located {
                        value: field,
                        pos: _,
                    },
            } => {
                let head = head.compile(compiler);
                let field = compiler.new_constant(Value::String(field));
                let dst = compiler.frame_mut().unwrap().new_register();
                compiler.write(
                    ByteCode::Field {
                        dst: Location::Register(dst),
                        head: head.into(),
                        field: Source::Constant(field),
                    },
                    ln,
                );
                Location::Register(dst)
            }
            Path::Index { head, index } => {
                let head = head.compile(compiler);
                let field = index.compile(compiler);
                let dst = compiler.frame_mut().unwrap().new_register();
                compiler.write(
                    ByteCode::Field {
                        dst: Location::Register(dst),
                        head: head.into(),
                        field,
                    },
                    ln,
                );
                Location::Register(dst)
            }
        }
    }
}
