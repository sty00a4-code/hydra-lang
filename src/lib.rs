use std::error::Error;

use scan::{
    lexer::Lexer,
    parser::{Parsable, Parser},
    position::Located,
};

#[cfg(test)]
mod tests;

pub mod scan;
pub mod run;

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
