extern crate linenoise;
extern crate lexers;
extern crate lisp;

#[cfg(not(test))]
fn main() {
    use std::rc::Rc;
    use lisp::{LispContext, Parser};
    let cx = Rc::new(LispContext::new());
    while let Some(input) = linenoise::input("~> ") {
        linenoise::history_add(&input[..]);
        match Parser::parse_str(&input[..]) {
            Err(e) => println!("Parse error: {:?}", e),
            Ok(exp) => match LispContext::eval(&exp, &cx) {
                Err(e) => println!("Eval error: {:?}", e),
                Ok(res) => println!("{}", res.to_string())
            }
        }
    }
}
