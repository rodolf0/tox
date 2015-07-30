extern crate linenoise;
extern crate tox;

#[cfg(not(test))]
mod repl {
    use tox::shunting::{Lexer, Token, ShuntingParser, MathContext};

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
        let mut ml = Lexer::from_str(input);
        let backtrack = ml.pos();
        if let (Some(Token::Variable(var)), Some(assig)) = (ml.next(), ml.next()) {
            if assig.is_op("=", 2) {
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

#[cfg(not(test))]
fn main() {
    if std::env::args().len() > 1 {
        let input = std::env::args().skip(1).
            collect::<Vec<String>>().connect(" ");
        repl::evalexpr(&input[..]);
    } else {
        use tox::shunting::MathContext;
        let mut cx = MathContext::new();
        while let Some(input) = linenoise::input(">> ") {
            linenoise::history_add(&input[..]);
            repl::parse_statement(&mut cx, &input[..]);
        }
    }
}
