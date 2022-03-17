mod repl {
    use lexers::{MathToken, MathTokenizer};
    use shunting::{MathContext, ShuntingParser, MathValue};

    pub fn evalexpr(input: &str) {
        match ShuntingParser::parse_str(input) {
            Err(e) => println!("Parse error: {:?}", e),
            Ok(expr) => match MathContext::new().eval(&expr) {
                Err(e) => println!("Eval error: {:?}", e),
                Ok(MathValue::Number(r)) => println!("{} = {}", expr, r),
                Ok(MathValue::RandVar(r)) => println!("{} = {}", expr, r.sample()),
            },
        };
    }

    pub fn parse_statement(cx: &mut MathContext, input: &str) {
        let mut ml = MathTokenizer::scanner(input.chars());
        let backtrack = ml.buffer_pos();
        if let (Some(MathToken::Variable(var)), Some(assig)) = (ml.next(), ml.next()) {
            if assig == MathToken::BOp(format!("=")) {
                match ShuntingParser::parse(&mut ml) {
                    Err(e) => println!("Parse error: {:?}", e),
                    Ok(rpn) => match cx.eval(&rpn) {
                        Err(e) => println!("Eval error: {:?}", e),
                        Ok(result) => cx.setvar(&var[..], result),
                    },
                }
                return;
            }
        }
        // wasn't assignment... try evaluating expression
        ml.set_buffer_pos(backtrack);
        match ShuntingParser::parse(&mut ml) {
            Err(e) => println!("Parse error: {:?}", e),
            Ok(rpn) => match cx.eval(&rpn) {
                Err(e) => println!("Eval error: {:?}", e),
                Ok(MathValue::Number(r)) => println!("{}", r),
                Ok(MathValue::RandVar(r)) => println!("*{}", r.sample()),
            },
        };
    }
}

fn main() {
    if std::env::args().len() > 1 {
        let input = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
        repl::evalexpr(&input[..]);
    } else {
        use shunting::MathContext;
        let mut cx = MathContext::new();
        let histpath = home::home_dir().map(|h| h.join(".tox_history")).unwrap();
        let mut rl = rustyline::Editor::<()>::new();
        if rl.load_history(&histpath).is_err() {
            println!("No history yet");
        }
        while let Ok(input) = rl.readline(">> ") {
            rl.add_history_entry(input.as_str());
            repl::parse_statement(&mut cx, &input[..]);
        }
        rl.save_history(&histpath).unwrap();
    }
}
