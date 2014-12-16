#![cfg(not(test))]

extern crate linenoise;
extern crate tox;

fn parse_n_eval(input: &str, cx: Option<&tox::rpneval::Context>) {
    match tox::shunting::parse(input) {
        Err(e) => println!("Parse error: {}", e),
        Ok(expr) => {
            match tox::rpneval::eval(&expr, cx) {
                Err(e) => println!("Eval error: {}", e),
                Ok(result) => println!("{}", result)
            }
        }
    }
}

fn main() {
    use std::collections::HashMap;
    use std::{os, f64};
    // init a context...
    let mut cx = HashMap::new();
    cx.insert(String::from_str("pi"), f64::consts::PI);
    cx.insert(String::from_str("e"), f64::consts::E);

    if os::args().len() > 1 {
        let input = os::args().tail().connect(" ");
        parse_n_eval(input.as_slice(), Some(&cx));
    } else {
        loop {
            let inopt = linenoise::input(">> ");
            match inopt {
                None => break,
                Some(input) => {
                    linenoise::history_add(input.as_slice());
                    parse_n_eval(input.as_slice(), Some(&cx));
                }
            }
        }
    }
}
