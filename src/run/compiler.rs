use std::collections::HashMap;

use crate::scan::{
    ast::{
        AssignOperator, Atom, BinaryOperator, Block, Chunk, Expression, Parameter, Path, Statement,
    },
    position::Located,
};

use super::{
    code::{ByteCode, Closure, Location, Source},
    value::Value,
};

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
    pub fn constant(&mut self, value: Value) -> usize {
        let frame = self.frame_mut().unwrap();
        let addr = frame.closure.constants.len();
        frame.closure.constants.push(value);
        addr
    }
    pub fn write(&mut self, bytecode: ByteCode, ln: usize) -> usize {
        let frame = self.frame_mut().unwrap();
        let addr = frame.closure.code.len();
        frame.closure.code.push(bytecode);
        frame.closure.lines.push(ln);
        addr
    }
    pub fn overwrite(&mut self, bytecode: ByteCode, ln: usize) -> usize {
        let frame = self.frame_mut().unwrap();
        let addr = frame.closure.code.len();
        frame.closure.code.push(bytecode);
        frame.closure.lines.push(ln);
        addr
    }
    pub fn none(&mut self) -> usize {
        self.write(ByteCode::None, 0)
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
            self.max_registers = self.registers
        }
        reg
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

pub trait Compilable: Sized {
    type Output;
    fn compile(self, compiler: &mut Compiler) -> Self::Output;
}

impl Compilable for Located<Chunk> {
    type Output = Option<Source>;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let Located {
            value: chunk,
            pos: _,
        } = self;
        for stat in chunk.stats {
            if let Some(src) = stat.compile(compiler) {
                return Some(src);
            }
        }
        None
    }
}
impl Compilable for Located<Block> {
    type Output = Option<Source>;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let Located {
            value: block,
            pos: _,
        } = self;
        for stat in block.stats {
            if let Some(src) = stat.compile(compiler) {
                return Some(src);
            }
        }
        None
    }
}
impl Compilable for Located<Statement> {
    type Output = Option<Source>;
    fn compile(self, compiler: &mut Compiler) -> Self::Output {
        let Located { value: stat, pos } = self;
        let ln = pos.ln.start;
        match stat {
            Statement::LetBinding { param, expr } => {
                let src = expr.compile(compiler);
                let dst = param.compile(compiler, src);
                compiler.move_checked(dst, src, ln);
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
                for Located {
                    value: param,
                    pos: param_pos,
                } in params
                {
                    todo!()
                }
            }
            Statement::Call { head, args } => todo!(),
            Statement::SelfCall { head, field, args } => todo!(),
            Statement::Return(Some(expr)) => {
                let src = expr.compile(compiler);
                compiler.write(ByteCode::Return { src: Some(src) }, ln);
                return Some(Source::default());
            }
            Statement::Return(None) => {
                compiler.write(ByteCode::Return { src: None }, ln);
                return Some(Source::default());
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
            Expression::Call { head, args } => todo!(),
            Expression::SelfCall { head, field, args } => todo!(),
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
            Atom::Int(v) => Source::Constant(compiler.constant(Value::Int(v)) as u16),
            Atom::Float(v) => Source::Constant(compiler.constant(Value::Float(v)) as u16),
            Atom::Bool(v) => Source::Constant(compiler.constant(Value::Bool(v)) as u16),
            Atom::Char(v) => Source::Constant(compiler.constant(Value::Char(v)) as u16),
            Atom::String(v) => Source::Constant(compiler.constant(Value::String(v)) as u16),
            Atom::Tuple(exprs) => {
                let dst = compiler.frame_mut().unwrap().new_register();
                let amount = exprs.len() as u8;
                let registers = compiler.frame().unwrap().registers;
                let start = registers;
                for expr in exprs {
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
                        pos: _,
                    },
                    expr,
                ) in pairs
                {
                    let ln = expr.pos.ln.start;
                    let src = expr.compile(compiler);
                    let field = Source::Constant(compiler.constant(Value::String(field)) as u16);
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
        todo!()
    }
}
impl Located<Parameter> {
    fn compile(self, compiler: &mut Compiler, src: Source) -> Location {
        let Located { value: param, pos } = self;
        let ln = pos.ln.start;
        match param {
            Parameter::Ident(ident) => todo!(),
            Parameter::Tuple(vec) => todo!(),
            Parameter::Vector(vec) => todo!(),
            Parameter::Map(vec) => todo!(),
        }
    }
}
