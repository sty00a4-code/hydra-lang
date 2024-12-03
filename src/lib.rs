#![feature(if_let_guard)]
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
