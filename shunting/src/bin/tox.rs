extern crate linenoise;
extern crate lexers;
extern crate shunting;

mod repl {
    use shunting::{ShuntingParser, MathContext};
    use lexers::{MathTokenizer, MathToken};

    pub fn evalexpr(input: &str) {
        match ShuntingParser::parse_str(input) {
            Err(e) => println!("Parse error: {:?}", e),
            Ok(expr) => match MathContext::new().eval(&expr) {
                Err(e) => println!("Eval error: {:?}", e),
                Ok(result) => println!("{} = {}", expr, result)
            }
        };
    }

    pub fn parse_statement(cx: &mut MathContext, input: &str) {
        let mut ml = MathTokenizer::from_str(input);
        let backtrack = ml.pos();
        if let (Some(MathToken::Variable(var)), Some(assig)) = (ml.next(), ml.next()) {
            if assig == MathToken::BOp(format!("=")) {
                match ShuntingParser::parse(&mut ml) {
                    Err(e) => println!("Parse error: {:?}", e),
                    Ok(rpn) => match cx.eval(&rpn) {
                        Err(e) => println!("Eval error: {:?}", e),
                        Ok(result) => cx.setvar(&var[..], result)
                    }
                }
                return;
            }
        }
        // wasn't assignment... try evaluating expression
        ml.set_pos(backtrack);
        match ShuntingParser::parse(&mut ml) {
            Err(e) => println!("Parse error: {:?}", e),
            Ok(rpn) => match cx.eval(&rpn) {
                Err(e) => println!("Eval error: {:?}", e),
                Ok(result) => println!("{}", result)
            }
        };
    }
}

fn main() {
    if std::env::args().len() > 1 {
        let input = std::env::args().skip(1).
            collect::<Vec<String>>().join(" ");
        repl::evalexpr(&input[..]);
    } else {
        use shunting::MathContext;
        let mut cx = MathContext::new();
        linenoise::history_load("~/.tox_history");
        linenoise::history_set_max_len(1000);
        while let Some(input) = linenoise::input(">> ") {
            linenoise::history_add(input.as_ref());
            linenoise::history_save("~/.tox_history");
            repl::parse_statement(&mut cx, &input[..]);
        }
    }
}
