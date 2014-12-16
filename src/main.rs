extern crate tox;

#[cfg(not(test))]
fn main() {
    use tox::shunting::parse;
    use tox::rpneval::eval;
    use std::collections::HashMap;
    use std::f64;

    let mut cx = HashMap::new();
    cx.insert(String::from_str("pi"), f64::consts::PI);
    cx.insert(String::from_str("e"), f64::consts::E);

    let rpn = parse("e!+cos(pi/7)^2+sin(pi/7)^2").ok().unwrap();
    println!("{}", eval(&rpn, Some(cx)).unwrap());
}
