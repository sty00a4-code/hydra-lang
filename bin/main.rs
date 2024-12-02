extern crate clap;
extern crate hydra_lang;
use clap::Parser;
use hydra_lang::{compile, scan::{ast::Chunk, position::Located}};
use std::{fs, process::exit};

fn main() {
    let args = HydraArgs::parse();
    if let Some(path) = args.input {
        let text = fs::read_to_string(&path)
            .map_err(|err| {
                eprintln!("ERROR {path}: {err}");
                exit(1)
            })
            .unwrap();
        let closure = compile::<Chunk>(&text)
            .map_err(|Located { value: err, pos }| {
                eprintln!("ERROR {path}:{}:{}: {err}", pos.ln.start + 1, pos.col.start + 1);
                exit(1)
            })
            .unwrap();
        dbg!(closure);
    }
}

#[derive(Debug, Parser)]
pub struct HydraArgs {
    input: Option<String>,
}
