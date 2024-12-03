extern crate clap;
extern crate hydra_lang;
use hydra_lang::{
    std_hydra,
    lex, parse,
    run::{
        compiler::{Compilable, Compiler, Frame, Scope},
        interpreter::{Interpreter, RunTimeError},
        value::{Function, Value},
    },
    scan::{
        self,
        ast::{Chunk, Expression, Statement},
        lexer::Line,
        parser::{Parsable, Parser},
        position::{Located, Position},
    },
};
use std::{
    error::Error,
    fmt::{Debug, Display},
    fs,
    io::{self, Write},
    process::exit,
    rc::Rc,
};

fn main() {
    use clap::Parser;
    let args = HydraArgs::parse();
    if let Some(path) = &args.input {
        let text = fs::read_to_string(path)
            .map_err(|err| {
                eprintln!("ERROR {path}: {err}");
                exit(1)
            })
            .unwrap();
        let value = run_args(&text, vec![], &args)
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

#[derive(Debug, clap::Parser)]
pub struct HydraArgs {
    input: Option<String>,

    #[clap(long, short, action)]
    tokens: bool,
    #[clap(long, short, action)]
    ast: bool,
    #[clap(long, short, action)]
    code: bool,
    #[clap(long, short, action)]
    debug: bool,
}

pub fn lex_args(text: &str, args: &HydraArgs) -> Result<Vec<Line>, Located<Box<dyn Error>>> {
    let lines = lex(text)?;
    if args.tokens {
        println!("TOKENS:");
        for Line { ln, indent, tokens } in &lines {
            print!("[{ln}] {}", " ".repeat(*indent));
            for token in tokens {
                print!("{token:?} ");
            }
            println!();
        }
    }
    Ok(lines)
}
pub fn parse_args<N: Parsable>(
    text: &str,
    args: &HydraArgs,
) -> Result<Located<N>, Located<Box<dyn Error>>>
where
    <N as scan::parser::Parsable>::Error: 'static,
{
    let lines = lex_args(text, args)?;
    let mut parser = Parser::new(lines);
    let ast = N::parse(&mut parser)
        .map_err(|Located { value: err, pos }| Located::new(err.into(), pos))?;
    if args.ast {
        println!("AST:");
        println!("{ast:#?}");
    }
    Ok(ast)
}
pub fn compile_args<N: Parsable>(
    text: &str,
    args: &HydraArgs,
) -> Result<<Located<N> as Compilable>::Output, Located<Box<dyn Error>>>
where
    <N as scan::parser::Parsable>::Error: 'static,
    Located<N>: Compilable,
    <Located<N> as Compilable>::Output: Display,
{
    let ast = parse_args::<N>(text, args)?;
    let mut compiler = Compiler {
        path: args.input.clone(),
        frame_stack: vec![Frame {
            scopes: vec![Scope::default()],
            ..Default::default()
        }],
    };
    let code = ast.compile(&mut compiler);
    if args.code {
        println!("CODE:");
        println!("<main>:\n{code}")
    }
    Ok(code)
}
pub fn run_args(
    text: &str,
    func_args: Vec<Value>,
    args: &HydraArgs,
) -> Result<Option<Value>, Located<Box<dyn Error>>> {
    let closure = compile_args::<Chunk>(text, args)?;
    let mut interpreter = Interpreter::default();
    std_hydra::import(&mut interpreter);
    interpreter
        .call(
            &Function {
                closure: Rc::new(closure),
            },
            func_args,
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
