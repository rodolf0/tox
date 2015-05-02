extern crate rand;
use std::collections::HashMap;
use std::str::FromStr;

use lexer::LexComp;
use shunting::{RPNExpr, Token};
use mathlink;

#[derive(Debug)]
pub enum EvalErr {
    UnknownVar(String),
    LinkError(String),
    WrongNumberOfArgs,
    BadNumber,
    BadToken(String),
}

// a shorthand for checking number of arguments before eval_fn
macro_rules! nargs {
    ($argcheck:expr, $ifok:expr) => {
        if $argcheck { $ifok } else { Err(EvalErr::WrongNumberOfArgs) }
    }
}

pub struct MathContext {
    context: HashMap<String, f64>
}

impl MathContext {
    pub fn new() -> MathContext {
        use std::f64::consts;
        let mut cx = HashMap::new();
        cx.insert("pi".to_string(), consts::PI);
        cx.insert("e".to_string(), consts::E);
        MathContext{context: cx}
    }

    pub fn setvar(&mut self, var: &str, val: f64) {
        self.context.insert(var.to_string(), val);
    }

    pub fn eval(&self, rpn: &RPNExpr) -> Result<f64, EvalErr> {
        let mut operands = Vec::new();
        for &Token{lxtoken: ref token, arity} in rpn.iter() {

            match token.lexcomp {
                LexComp::Number => {
                    match f64::from_str(&token.lexeme[..]) {
                        Ok(n) => operands.push(n),
                        Err(_) => return Err(EvalErr::BadNumber)
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
                    match Self::eval_fn("tgamma", vec![o + 1.0]) {
                        Ok(n) => operands.push(n),
                        Err(e) => return Err(e)
                    }
                },
                LexComp::Function => {
                    if arity > operands.len() {
                        return Err(EvalErr::WrongNumberOfArgs);
                    }
                    let fname = &token.lexeme[..];
                    let args = operands.iter().cloned().take(arity).collect::<Vec<f64>>();
                    let ndrop = operands.len() - arity;
                    operands.truncate(ndrop);
                    match Self::eval_fn(fname, args) {
                        Ok(n) => operands.push(n),
                        Err(e) => return Err(e)
                    }
                },

                _ => return Err(EvalErr::BadToken(token.lexeme.clone()))
            }
        }
        operands.pop().ok_or(EvalErr::WrongNumberOfArgs)
    }

    fn eval_fn(fname: &str, args: Vec<f64>) -> Result<f64, EvalErr> {
        match fname {
            "sin" => nargs!(args.len() == 1, Ok(args[0].cos())),
            "cos" => nargs!(args.len() == 1, Ok(args[0].cos())),
            "atan2" => nargs!(args.len() == 2, Ok(args[0].atan2(args[1]))),
            "max" => nargs!(args.len() > 0,
                            Ok(args[1..].iter().fold(args[0], |a, &item| a.max(item)))),
            "min" => nargs!(args.len() > 0,
                            Ok(args[1..].iter().fold(args[0], |a, &item| a.min(item)))),
            "abs" => nargs!(args.len() == 1, Ok(f64::abs(args[0]))),
            "rand" => nargs!(args.len() == 1, Ok(rand::random::<f64>())),
            _ => match mathlink::link_fn(fname) {
                Ok(func) => nargs!(args.len() == 1, Ok(func(args[0]))),
                Err(e) => Err(EvalErr::LinkError(e))
            }
        }
    }
}
