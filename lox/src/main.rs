#![deny(warnings)]

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

mod lox_scanner;
mod lox_parser;
mod lox_interpreter;
mod lox_environment;
mod lox_native;

use lox_scanner::LoxScanner;
use lox_parser::LoxParser;
use lox_interpreter::LoxInterpreter;


fn main() {
    if env::args().len() > 2 {
        eprintln!("usage: lox [script]");
        return;
    }

    let run = |source: String| {
        let scanner = LoxScanner::scanner(source);
        let mut parser = LoxParser::new(scanner);
        let mut interpreter = LoxInterpreter::new();
        match parser.parse() {
            Ok(stmts) => {
                if let Err(error) = interpreter.interpret(&stmts) {
                    eprintln!("LoxInterpreter error: {}", error)
                }
            }
            Err(errors) => for e in errors { eprintln!("{}", e); }
        }
    };

    if env::args().len() == 2 {
        let sourcefile = env::args().skip(1).next().unwrap();
        if let Ok(mut f) = File::open(&sourcefile) {
            let mut source = String::new();
            if f.read_to_string(&mut source).is_ok() {
                return run(source);
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
                Ok(_) => run(input),
                Err(e) => eprintln!("lox read_line error: {:?}", e)
            }
        }
    }
}
