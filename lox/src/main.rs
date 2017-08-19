#![deny(warnings)]

//mod lox_grammar;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};


fn run(source: String) {
    eprintln!("{}", source);
}

fn main() {
    if env::args().len() > 2 {
        eprintln!("usage: lox [script]");
        return;
    } else if env::args().len() == 2 {
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
                Err(e) => eprintln!("read_line error: {:?}", e)
            }
        }
    }
}
