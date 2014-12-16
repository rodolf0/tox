use std::collections::HashMap;
use std::num::FloatMath;
use std::num::Float;
use math_lexer::LexComp;
use shunting::RPNExpr;
use std::dynamic_lib::DynamicLibrary;
use std::mem;


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

// Evaluate some functions
fn eval_fn(fname: &str, params: &[f64]) -> f64 {
    match fname {
        "sin" => params.last().unwrap().sin(),
        _ => {
            if let Ok(func) = link_fn(fname) {
                let p = params.last().unwrap();
                return func(*p);
            }
            panic!("quack! rpneval::eval_fn");
        }
    }
}


pub type Context = HashMap<String, f64>;

// Evaluate a RPN expression
pub fn eval(rpn: &RPNExpr, cx: Option<Context>) -> Option<f64> {
    let mut stack = Vec::new();

    for tok in rpn.iter() {
        match tok.lxtok.lexcomp {
            LexComp::Number => {
                let s = tok.lxtok.lexeme.as_slice();
                let n = from_str::<f64>(s).unwrap();
                stack.push(n);
            },

            LexComp::Plus => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l + r);
            },

            LexComp::Minus => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l - r);
            },

            LexComp::Times => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l * r);
            },

            LexComp::Divide => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l / r);
            },

            LexComp::Modulo => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l.rem(&r));
            },

            LexComp::Power => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l.powf(r));
            },

            LexComp::UMinus => {
                let r = stack.pop().unwrap();
                stack.push(-r);
            },

            LexComp::Factorial => {
                let l = stack.pop().unwrap();
                if let Ok(func) = link_fn("tgamma") {
                    stack.push(func(l + 1.0));
                }
            },

            LexComp::Function => {
                let mut r: f64;
                let midp = stack.len() - tok.arity;
                {   let args = stack.slice_from(midp);
                    let fname = tok.lxtok.lexeme.as_slice();
                    r = eval_fn(fname, args); } // limit lifetime of args
                stack.truncate(midp);
                stack.push(r);
            },

            LexComp::Variable => {
                let vname = tok.lxtok.lexeme.as_slice();
                if let Some(ref context) = cx {
                    if let Some(v) = context.get(vname) {
                        stack.push(*v);
                    } else {
                        panic!("Unknown variable [{}]", vname);
                    }
                } else {
                    panic!("rpneval::eval: No context provided");
                }
            },

            LexComp::Unknown | LexComp::OParen |
            LexComp::CParen | LexComp::Comma => panic!("rpneval::eval: non-reachable")
        }
    }
    stack.pop()
}
