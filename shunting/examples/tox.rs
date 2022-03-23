mod repl {
    use lexers::{MathToken, MathTokenizer};
    use shunting::{MathContext, ShuntingParser, MathOp};

    pub fn evalexpr(input: &str) {
        match ShuntingParser::parse_str(input) {
            Err(e) => println!("Parse error: {:?}", e),
            Ok(expr) => match MathContext::new().eval(&expr) {
                Err(e) => println!("Eval error: {:?}", e),
                Ok(r) => println!("{} = {}", expr, r),
            },
        };
    }

    pub fn parse_statement(cx: &MathContext, input: &str) {
        let mut ml = MathTokenizer::scanner(input.chars());
        let backtrack = ml.buffer_pos();
        if let (Some(MathToken::Variable(var)), Some(op)) = (ml.next(), ml.next()) {
            if op == MathToken::BOp("=".to_string()) {
                match ShuntingParser::parse(&mut ml) {
                    Err(e) => println!("Parse error: {:?}", e),
                    Ok(expr) => match cx.compile(&expr) {
                        Err(e) => println!("Compile error: {:?}", e),
                        Ok(code) => cx.setvar(&var, code),
                    }
                }
                return;
            }
        }
        // wasn't assignment... try evaluating expression
        ml.set_buffer_pos(backtrack);
        match ShuntingParser::parse(&mut ml) {
            Err(e) => println!("Parse error: {:?}", e),
            Ok(expr) => match cx.compile(&expr) {
                Err(e) => println!("Compile error: {:?}", e),
                Ok(MathOp::Number(n)) => println!("{}", n),
                Ok(x) => println!("{:?}", x.histogram::<15>(2000)),
            }
        };
    }
}

fn main() {
    if std::env::args().len() > 1 {
        let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
        repl::evalexpr(&input[..]);
    } else {
        use shunting::MathContext;
        let cx = MathContext::new();
        let histpath = home::home_dir().map(|h| h.join(".tox_history")).unwrap();
        let mut rl = rustyline::Editor::<()>::new();
        if rl.load_history(&histpath).is_err() {
            println!("No history yet");
        }
        while let Ok(input) = rl.readline(">> ") {
            rl.add_history_entry(input.as_str());
            repl::parse_statement(&cx, &input[..]);
        }
        rl.save_history(&histpath).unwrap();
    }
}
