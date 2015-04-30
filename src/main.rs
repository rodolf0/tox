extern crate linenoise;
extern crate tox;

#[cfg(not(test))]
mod repl {
    use std::io;
    use tox::lexer::{MathLexer, LexComp};
    use tox::rpneval::{EvalErr, Context, eval};
    use tox::shunting::{ParseError, MathParser};
    use tox::{shunting, rpneval};

    #[derive(Debug)]
    enum REPLErr {
        ParseErr(ParseError),
        EvalErr(EvalErr),
    }

    pub fn evalexpr(input: &str, context: Option<&Context>) {
        match MathParser::parse(input) {
            Err(e) => println!("Parse error: {:?}", e),
            Ok(expr) => match rpneval::eval(&expr, context) {
                Err(e) => println!("Eval error: {:?}", e),
                Ok(result) => println!("{}", result)
            }
        };
    }

    //fn parse_n_eval_expression<R: io::Reader>(input: &mut MathLexer<R>,
                                //cx: Option<&Context>) -> Result<f64, REPLErr> {
        //match MathParser::(input) {
            //Err(e) => Err(REPLErr::ParseErr(e)),
            //Ok(expr) => match rpneval::eval(&expr, cx) {
                //Err(e) => Err(REPLErr::EvalErr(e)),
                //Ok(result) => Ok(result)
            //}
        //}
    //}

    //pub fn parse_statement(input: &str, context: Option<&mut Context>) {
        //let mut ml = MathLexer::from_str(input);
        //// check if this statement is an assignment
        //let backtrack = ml.pos;
        //if let (Some(var), Some(assig)) = (ml.next(), ml.next()) {
            //if var.lexcomp == LexComp::Variable && assig.lexcomp == LexComp::Assign {
                //if context.is_none() {
                    //println!("Assign error: no context");
                    //return;
                //}
                //let result = match parse_n_eval_expression(&mut ml, context.as_ref().map(|cx| &**cx)) {
                    //Err(e) => { println!("Assign error: {:?}", e); return; }
                    //Ok(result) => result
                //};
                //context.unwrap().insert(var.lexeme, result);
                //return;
            //}
        //}
        //// wasn't assignment... try evaluating expression
        //ml.pos = backtrack;
        //// that crazy map is doing Option<&mut T> -> Option<&T>
        //match parse_n_eval_expression(&mut ml, context.as_ref().map(|cx| &**cx)) {
            //Err(e) => println!("Error: {:?}", e),
            //Ok(result) => println!("{}", result)
        //};
    //}
}


#[cfg(not(test))]
fn main() {
    use std::collections::HashMap;
    // init a context...
    let mut cx = HashMap::new();
    cx.insert("pi".to_string(), std::f64::consts::PI);
    cx.insert("e".to_string(), std::f64::consts::E);

    if std::env::args().len() > 1 {
        let input = std::env::args().skip(1).collect::<Vec<String>>().connect(" ");
        repl::evalexpr(&input[..], Some(&cx));
    //} else {
        //loop {
            //let inopt = linenoise::input(">> ");
            //match inopt {
                //None => break,
                //Some(input) => {
                    //linenoise::history_add(input[..]);
                    //repl::parse_statement(input[..], Some(&mut cx));
                //}
            //}
        //}
    }
}
