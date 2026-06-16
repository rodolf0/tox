#![deny(warnings)]

fn main() {
    use lisp::LispContext;
    use std::rc::Rc;
    let cx = Rc::new(LispContext::new());
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    while let Ok(input) = rl.readline("~> ") {
        let _ = rl.add_history_entry(&input);
        match lisp::parse(&input[..]) {
            Err(e) => println!("Parse error: {:?}", e),
            Ok(exp) => match LispContext::eval(&exp, &cx) {
                Err(e) => println!("Eval error: {:?}", e),
                Ok(res) => println!("{}", res.to_string()),
            },
        }
    }
}
