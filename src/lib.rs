#![feature(if_let_guard)]
use std::error::Error;

use run::{
    code::Closure,
    compiler::{Compilable, Compiler, Frame, Scope},
};
use scan::{
    lexer::Lexer,
    parser::{Parsable, Parser},
    position::Located,
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
    let lines = Lexer::from(text)
        .lex()
        .map_err(|Located { value: err, pos }| Located::new(err.into(), pos))?;
    let mut parser = Parser::new(lines);
    let ast = N::parse(&mut parser)
        .map_err(|Located { value: err, pos }| Located::new(err.into(), pos))?;
    let mut compiler = Compiler {
        frame_stack: vec![Frame {
            scopes: vec![Scope::default()],
            ..Default::default()
        }],
    };
    ast.compile(&mut compiler);
    Ok(compiler.pop_frame().unwrap().closure)
}
