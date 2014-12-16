use std::collections::HashMap;
use std::num::FloatMath;
use std::num::Float;
use math_lexer::LexComp;
use shunting::RPNExpr;
use std::dynamic_lib::DynamicLibrary;
use std::mem;

#[deriving(Show)]
pub enum EvalErr {
    UnknownVariable(String),
    NoContextProvided,
    LinkError(String),
    WrongNumberOfArgs,
}

// Use dynamic linker to get hold of math library functions
fn link_fn(fname: &str) -> Result<fn(f64) -> f64, String> {
    // http://doc.rust-lang.org/std/dynamic_lib/struct.DynamicLibrary.html
    match DynamicLibrary::open::<&str>(None) { // open self
        Err(e) => return Err(e),
        Ok(lib) => {
            let func = unsafe {
                // a very generic pointer: '*mut u8'
                match lib.symbol(fname) {
                    Err(e) => return Err(e),
                    Ok(f) => mem::transmute::<*mut u8, fn(f64) -> f64>(f)
                }
            };
            return Ok(func);
        }
    }
}

// Evaluate known functions, fallback to math-library
fn eval_fn(fname: &str, params: &[f64]) -> Result<f64, EvalErr> {
    match fname {
        "sin" => return Ok(params.last().unwrap().sin()),
        _ => {
            match link_fn(fname) {
                Ok(func) => {
                    let p = params.last().unwrap();
                    return Ok(func(*p));
                },
                Err(e) => return Err(EvalErr::LinkError(e))
            }
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
        match tok.lxtok.lexcomp {
            LexComp::Number => {
                let s = tok.lxtok.lexeme.as_slice();
                let n = from_str::<f64>(s).unwrap();
                stack.push(n);
            },

            LexComp::Plus => { let (r, l) = try!(pop2(&mut stack)); stack.push(l + r); },
            LexComp::Minus => { let (r, l) = try!(pop2(&mut stack)); stack.push(l - r); },
            LexComp::Times => { let (r, l) = try!(pop2(&mut stack)); stack.push(l * r); },
            LexComp::Divide => { let (r, l) = try!(pop2(&mut stack)); stack.push(l / r); },
            LexComp::Modulo => { let (r, l) = try!(pop2(&mut stack)); stack.push(l.rem(&r)); },
            LexComp::Power => { let (r, l) = try!(pop2(&mut stack)); stack.push(l.powf(r)); },

            LexComp::UMinus => {
                if let Some(r) = stack.pop() {
                    stack.push(-r);
                } else {
                    return Err(EvalErr::WrongNumberOfArgs);
                }
            },

            LexComp::Factorial => {
                if let Some(l) = stack.pop() {
                    match link_fn("tgamma") {
                        Ok(func) => stack.push(func(l + 1.0)),
                        Err(e) => return Err(EvalErr::LinkError(e))
                    }
                } else {
                    return Err(EvalErr::WrongNumberOfArgs);
                }
            },

            LexComp::Function => {
                let mut r: f64;
                let midp = stack.len() - tok.arity;
                if tok.arity > stack.len() {
                    return Err(EvalErr::WrongNumberOfArgs);
                } else {
                    let args = stack.slice_from(midp);
                    let fname = tok.lxtok.lexeme.as_slice();
                    match eval_fn(fname, args) {
                        Ok(evaled) => r = evaled,
                        Err(e) => return Err(e)
                    }
                }
                stack.truncate(midp);
                stack.push(r);
            },

            LexComp::Variable => {
                let vname = tok.lxtok.lexeme.as_slice();
                if let Some(context) = cx {
                    if let Some(v) = context.get(vname) {
                        stack.push(*v);
                    } else {
                        return Err(EvalErr::UnknownVariable(String::from_str(vname)));
                    }
                } else {
                    return Err(EvalErr::NoContextProvided);
                }
            },

            LexComp::Unknown | LexComp::OParen |
            LexComp::CParen | LexComp::Comma => panic!("rpneval::eval: parser error")
        }
    }
    if let Some(res) = stack.pop() {
        return Ok(res);
    }
    panic!("rpneval::eval: parser error 2");
}
