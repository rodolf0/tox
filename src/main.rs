extern crate tox;

#[cfg(not(test))]
fn main() {
    use tox::shunting::parse;
    use tox::rpneval::eval;

    let rpn = parse("3+4*2/-(1-5)^2^3").ok().unwrap();
    println!("{}", eval(&rpn).unwrap());
}
