extern crate tox;

#[cfg(not(test))]
fn main() {
    use tox::shunting::parse;
    use tox::rpneval::eval;
    use std::collections::HashMap;
    use std::f64;
    use std::io;

    let mut cx = HashMap::new();
    cx.insert(String::from_str("pi"), f64::consts::PI);
    cx.insert(String::from_str("e"), f64::consts::E);

    let mut input = io::stdin();
    print!(">> ");
    while let Ok(line) = input.read_line() {
        match parse(line.as_slice()) {
            Err(e) => println!("ERROR: {}", e),
            Ok(e) => println!("{}", eval(&e, Some(&cx)).unwrap())
        };
        print!(">> ");
    }
}
