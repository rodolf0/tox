use lexer::LexComp;
use shunting::{RPNExpr, Token};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub enum EvalErr {
    UnknownVar(String),
    LinkError(String),
    WrongNumberOfArgs,
    BadNumber,
    BadToken(String),
}

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

pub struct MathContext {
    context: HashMap<String, f64>
}

impl MathContext {
    pub fn new() -> MathContext {
        let mut cx = HashMap::new();
        //cx.insert("pi".to_string(), std::f64::consts::PI);
        //cx.insert("e".to_string(), std::f64::consts::E);
        MathContext{context: cx}
    }

    pub fn eval(&self, rpn: &RPNExpr) -> Result<f64, EvalErr> {
        let mut operands = Vec::new();
        for &Token{lxtoken: ref token, arity: arity} in rpn.iter() {

            match token.lexcomp {
                LexComp::Number => {
                    match f64::from_str(&token.lexeme[..]) {
                        Ok(n) => operands.push(n),
                        Err(e) => return Err(EvalErr::BadNumber)
                    }
                },
                LexComp::Variable => {
                    let var = &token.lexeme[..];
                    match self.context.get(var) {
                        Some(value) => operands.push(*value),
                        None => return Err(EvalErr::UnknownVar(var.to_string()))
                    }
                },
                LexComp::Plus | LexComp::Minus |
                LexComp::Times  | LexComp::Divide |
                LexComp::Modulo  | LexComp::Power => {
                    let r = try!(operands.pop().ok_or(EvalErr::WrongNumberOfArgs));
                    let l = try!(operands.pop().ok_or(EvalErr::WrongNumberOfArgs));
                    match token.lexcomp {
                        LexComp::Plus => operands.push(l + r),
                        LexComp::Minus => operands.push(l - r),
                        LexComp::Times => operands.push(l * r),
                        LexComp::Divide => operands.push(l / r),
                        LexComp::Modulo => operands.push(l % r),
                        LexComp::Power => operands.push(l.powf(r)),
                        _ => unreachable!()
                    }
                },
                LexComp::UMinus => {
                    let o = try!(operands.pop().ok_or(EvalErr::WrongNumberOfArgs));
                    operands.push(-o);
                },
                LexComp::Factorial => {
                    let o = try!(operands.pop().ok_or(EvalErr::WrongNumberOfArgs));
                    // gamma(o + 1.0);
                    return Err(EvalErr::LinkError("couldn't link tgamma".to_string()));
                },

                /*
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
                */

                _ => return Err(EvalErr::BadToken(token.lexeme.clone()))
            }
        }
        operands.pop().ok_or(EvalErr::WrongNumberOfArgs)
    }
}
