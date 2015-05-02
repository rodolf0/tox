extern crate linenoise;
extern crate tox;

#[cfg(not(test))]
mod repl {
    use tox::lexer::{MathLexer, LexComp};
    use tox::shunting::MathParser;
    use tox::rpneval::MathContext;

    pub fn evalexpr(input: &str) {
        match MathParser::parse_str(input) {
            Err(e) => println!("Parse error: {:?}", e),
            Ok(expr) => match MathContext::new().eval(&expr) {
                Err(e) => println!("Eval error: {:?}", e),
                Ok(result) => println!("{}", result)
            }
        };
    }

    pub fn parse_statement(cx: &mut MathContext, input: &str) {
        let mut ml = MathLexer::lex_str(input);
        let backtrack = ml.pos();
        if let (Some(var), Some(assig)) = (ml.next(), ml.next()) {
            if var.is(&LexComp::Variable) && assig.is(&LexComp::Assign) {
                match MathParser::parse(&mut ml) {
                    Err(e) => println!("Parse error: {:?}", e),
                    Ok(rpn) => match cx.eval(&rpn) {
                        Err(e) => println!("Eval error: {:?}", e),
                        Ok(result) => cx.setvar(&var.lexeme[..], result)
                    }
                };
                return;
            }
        }
        // wasn't assignment... try evaluating expression
        ml.set_pos(backtrack);
        match MathParser::parse(&mut ml) {
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
        use tox::rpneval::MathContext;
        let mut cx = MathContext::new();
        while let Some(input) = linenoise::input(">> ") {
            linenoise::history_add(&input[..]);
            repl::parse_statement(&mut cx, &input[..]);
        }
    }
}
