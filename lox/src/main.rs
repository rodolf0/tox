#![deny(warnings)]

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

mod lox_scanner;
mod lox_parser;
mod lox_interpreter;
mod lox_environment;
mod lox_native;
mod lox_resolver;

use crate::lox_scanner::LoxScanner;
use crate::lox_parser::LoxParser;
use crate::lox_interpreter::LoxInterpreter;
use crate::lox_resolver::Resolver;


fn main() {
    if env::args().len() > 2 {
        eprintln!("usage: lox [script]");
        return;
    }

    let run = |source: String, interpreter: &mut LoxInterpreter| {
        let scanner = LoxScanner::scanner(source.chars());
        let mut parser = LoxParser::new(scanner);
        match parser.parse() {
            Ok(stmts) => {
                match Resolver::new(interpreter).resolve(&stmts) {
                    Ok(_) => if let Err(error) = interpreter.interpret(&stmts) {
                        eprintln!("LoxInterpreter error: {}", error)
                    },
                    Err(error) => eprintln!("Resolve error: {}", error)
                }
            }
            Err(errors) => for e in errors { eprintln!("{}", e); }
        }
    };

    let mut interpreter = LoxInterpreter::new();
    if env::args().len() == 2 {
        let sourcefile = env::args().skip(1).next().unwrap();
        if let Ok(mut f) = File::open(&sourcefile) {
            let mut source = String::new();
            if f.read_to_string(&mut source).is_ok() {
                return run(source, &mut interpreter);
            }
        }
        eprintln!("lox: failed to read source file {}", sourcefile);
        std::process::exit(1);
    } else {
        loop {
            let mut input = String::new();
            io::stdout().write(b"~> ").unwrap();
            io::stdout().flush().unwrap();
            match io::stdin().read_line(&mut input) {
                Ok(_) => run(input, &mut interpreter),
                Err(e) => eprintln!("lox read_line error: {:?}", e)
            }
        }
    }
}
