extern crate tox;
extern crate linenoise;

#[cfg(not(test))]
fn main() {
    use tox::shunting::parse;
    use tox::rpneval::eval;
    use std::collections::HashMap;
    use std::f64;

    let mut cx = HashMap::new();
    cx.insert(String::from_str("pi"), f64::consts::PI);
    cx.insert(String::from_str("e"), f64::consts::E);

    loop {
        let inopt = linenoise::input(">> ");
        match inopt {
            None => break,
            Some(input) => {
                linenoise::history_add(input.as_slice());
                match parse(input.as_slice()) {
                    Ok(expr) => {
                        match eval(&expr, Some(&cx)) {
                            Ok(result) => println!("{}", result),
                            Err(e) => println!("Eval error: {}", e)
                        }
                    },
                    Err(e) => println!("Parse error: {}", e)
                }
            }
        }
    }
}
