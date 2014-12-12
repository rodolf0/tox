use std::collections::HashMap;
use std::num::FloatMath;
use std::num::Float;
use math_lexer::LexComp;
use shunting::RPNExpr;

// temporal workaround until gamma is reachable in the library
extern {
    fn tgamma(x: f64) -> f64;
}

fn eval_fn(fname: &str, params: &[f64]) -> f64 {
    match fname {
        "sin(" => params.last().unwrap().sin(),
        "cos(" => params.last().unwrap().cos(),
        "tan(" => params.last().unwrap().cos(),
        _ => panic!("rpneval: undefined function [{}]", fname)
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
                stack.push(unsafe { tgamma(l + 1.0) });
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
