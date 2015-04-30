use lexer::LexComp;
use shunting::RPNExpr;
use std::collections::HashMap;
use std::str::FromStr;


#[derive(Debug)]
pub enum EvalErr {
    UnknownVariable(String),
    NoContextProvided,
    LinkError(String),
    WrongNumberOfArgs,
    BadNumber,
}

// Evaluate known functions, fallback to math-library
fn eval_fn(fname: &str, params: &[f64]) -> Result<f64, EvalErr> {
    match fname {
        "sin" => return Ok(params.last().unwrap().sin()),
        _ => {
            //match mathlink::link_fn(fname) {
                //Ok(func) => {
                    //let p = params.last().unwrap();
                    //return Ok(func(*p));
                //},
                //Err(e) => return Err(EvalErr::LinkError(e))
            //}
            Err(EvalErr::LinkError("blah!".to_string()))
        }
    }
}

fn pop2(stack: &mut Vec<f64>) -> Result<(f64, f64), EvalErr> {
    let r = stack.pop();
    let l = stack.pop();
    if r.is_none() || l.is_none() {
        return Err(EvalErr::WrongNumberOfArgs);
    }
    Ok((r.unwrap(), l.unwrap()))
}

pub type Context = HashMap<String, f64>;

// Evaluate a RPN expression
pub fn eval(rpn: &RPNExpr, cx: Option<&Context>) -> Result<f64, EvalErr> {
    let mut stack = Vec::new();

    for tok in rpn.iter() {
        match tok.lxtoken.lexcomp {
            LexComp::Number => {
                let s = &tok.lxtoken.lexeme[..];
                if let Ok(n) = f64::from_str(s) {
                    stack.push(n);
                } else {
                    return Err(EvalErr::BadNumber);
                }
            },

            LexComp::Plus => { let (r, l) = try!(pop2(&mut stack)); stack.push(l + r); },
            LexComp::Minus => { let (r, l) = try!(pop2(&mut stack)); stack.push(l - r); },
            LexComp::Times => { let (r, l) = try!(pop2(&mut stack)); stack.push(l * r); },
            LexComp::Divide => { let (r, l) = try!(pop2(&mut stack)); stack.push(l / r); },
            LexComp::Modulo => { let (r, l) = try!(pop2(&mut stack)); stack.push(l % r); },
            LexComp::Power => { let (r, l) = try!(pop2(&mut stack)); stack.push(l.powf(r)); },

            LexComp::UMinus => {
                if let Some(r) = stack.pop() {
                    stack.push(-r);
                } else {
                    return Err(EvalErr::WrongNumberOfArgs);
                }
            },

            //LexComp::Factorial => {
                //if let Some(l) = stack.pop() {
                    //match mathlink::link_fn("tgamma") {
                        //Ok(func) => stack.push(func(l + 1.0)),
                        //Err(e) => return Err(EvalErr::LinkError(e))
                    //}
                //} else {
                    //return Err(EvalErr::WrongNumberOfArgs);
                //}
            //},

            LexComp::Function => {
                let mut r: f64;
                let midp = stack.len() - tok.arity;
                if tok.arity > stack.len() {
                    return Err(EvalErr::WrongNumberOfArgs);
                } else {
                    let args = &stack[midp..];
                    let fname = &tok.lxtoken.lexeme[..];
                    match eval_fn(fname, args) {
                        Ok(evaled) => r = evaled,
                        Err(e) => return Err(e)
                    }
                }
                stack.truncate(midp);
                stack.push(r);
            },

            LexComp::Variable => {
                let vname = &tok.lxtoken.lexeme[..];
                if let Some(context) = cx {
                    if let Some(v) = context.get(vname) {
                        stack.push(*v);
                    } else {
                        return Err(EvalErr::UnknownVariable(vname.to_string()));
                    }
                } else {
                    return Err(EvalErr::NoContextProvided);
                }
            },

            LexComp::Factorial | // TODO: allow factorial
            LexComp::Unknown | LexComp::OParen | LexComp::Assign |
            LexComp::CParen | LexComp::Comma => panic!("rpneval::eval: parser error")
        }
    }
    if let Some(res) = stack.pop() {
        return Ok(res);
    }
    panic!("rpneval::eval: parser error 2");
}
