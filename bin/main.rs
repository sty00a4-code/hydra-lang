extern crate clap;
extern crate hydra_lang;
use clap::Parser;
use hydra_lang::{
    parse, run,
    run::{
        compiler::{Compilable, Compiler, Frame, Scope},
        interpreter::{Interpreter, RunTimeError},
        value::Function,
    },
    scan::{
        ast::{Chunk, Expression, Statement},
        position::{Located, Position},
    },
};
use std::{
    fs,
    io::{self, Write},
    process::exit,
    rc::Rc,
};

fn main() {
    let args = HydraArgs::parse();
    if let Some(path) = args.input {
        let text = fs::read_to_string(&path)
            .map_err(|err| {
                eprintln!("ERROR {path}: {err}");
                exit(1)
            })
            .unwrap();
        let value = run(&text, vec![])
            .map_err(|Located { value: err, pos }| {
                eprintln!(
                    "ERROR {path}:{}:{}: {err}",
                    pos.ln.start + 1,
                    pos.col.start + 1
                );
                exit(1)
            })
            .unwrap();
        if let Some(value) = value {
            println!("{value:?}");
        }
    } else {
        let mut interpreter = Interpreter::default();
        loop {
            let mut input = String::new();
            print!("> ");
            let Ok(_) = io::stdout().flush().map_err(|err| {
                eprintln!("{err}");
            }) else {
                break;
            };
            let Ok(_) = io::stdin().read_line(&mut input).map_err(|err| {
                eprintln!("{err}");
            }) else {
                break;
            };
            let input = input.trim();
            let ast = parse::<Chunk>(input)
                .or_else(|_| {
                    parse::<Expression>(input).map(|expr| {
                        let pos = expr.pos.clone();
                        Located::new(
                            Chunk {
                                stats: vec![Located::new(Statement::Return(Some(expr)), pos)],
                            },
                            Position::default(),
                        )
                    })
                })
                .map_err(|Located { value: err, pos }| {
                    eprintln!(
                        "ERROR <stdin>:{}:{}: {err}",
                        pos.ln.start + 1,
                        pos.col.start + 1
                    );
                })
                .unwrap();
            let mut compiler = Compiler::default();
            let closure = ast.compile(&mut compiler);
            let Ok(_) = interpreter
                .call(
                    &Function {
                        closure: Rc::new(closure),
                    },
                    vec![],
                    None,
                )
                .map_err(|RunTimeError { err, ln }| {
                    eprintln!("ERROR <stdin>:{}:{}: {err}", ln + 1, 0);
                })
            else {
                continue;
            };
            let Ok(value) = interpreter.run().map_err(|RunTimeError { err, ln }| {
                eprintln!("ERROR <stdin>:{}:{}: {err}", ln + 1, 0);
            }) else {
                continue;
            };
            if let Some(value) = value {
                println!("{value:?}")
            }
        }
    }
}

#[derive(Debug, Parser)]
pub struct HydraArgs {
    input: Option<String>,
}
