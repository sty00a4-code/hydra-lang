#![feature(if_let_guard)]
use std::{error::Error, rc::Rc};

use run::{
    code::Closure,
    compiler::{Compilable, Compiler, Frame, Scope},
    interpreter::Interpreter,
    value::{Function, Value},
};
use scan::{
    ast::Chunk,
    lexer::Lexer,
    parser::{Parsable, Parser},
    position::{Located, Position},
};

#[cfg(test)]
mod tests;

pub mod run;
pub mod scan;

pub fn parse<N: Parsable>(text: &str) -> Result<Located<N>, Located<Box<dyn Error>>>
where
    <N as scan::parser::Parsable>::Error: 'static,
{
    let lines = Lexer::from(text)
        .lex()
        .map_err(|Located { value: err, pos }| Located::new(err.into(), pos))?;
    let mut parser = Parser::new(lines);
    N::parse(&mut parser).map_err(|Located { value: err, pos }| Located::new(err.into(), pos))
}

pub fn compile<N: Parsable>(text: &str) -> Result<Closure, Located<Box<dyn Error>>>
where
    <N as scan::parser::Parsable>::Error: 'static,
    Located<N>: Compilable,
{
    let ast = parse::<N>(text)?;
    let mut compiler = Compiler {
        frame_stack: vec![Frame {
            scopes: vec![Scope::default()],
            ..Default::default()
        }],
    };
    ast.compile(&mut compiler);
    Ok(compiler.pop_frame().unwrap().closure)
}

pub fn run<N: Parsable>(
    text: &str,
    args: Vec<Value>,
) -> Result<Option<Value>, Located<Box<dyn Error>>>
where
    <N as scan::parser::Parsable>::Error: 'static,
    Located<N>: Compilable,
{
    let closure = dbg!(compile::<N>(text)?);
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
