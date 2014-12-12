extern crate tox;

#[cfg(not(test))]
fn main() {
    use tox::shunting::parse;
    use tox::rpneval::eval;
    use std::collections::HashMap;

    let mut cx = HashMap::new();
    cx.insert(String::from_str("x"), 3.4);

    let rpn = parse("3!+x").ok().unwrap();
    println!("{}", eval(&rpn, Some(cx)).unwrap());
}
