use std::collections::HashMap;
use lexers::MathToken;
use crate::parser::RPNExpr;

#[derive(Debug, PartialEq)]
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

pub struct MathContext(pub HashMap<String, f64>);

impl MathContext {
    pub fn new() -> MathContext {
        use std::f64::consts;
        let mut cx = HashMap::new();
        cx.insert(format!("pi"), consts::PI);
        cx.insert(format!("e"), consts::E);
        MathContext(cx)
    }

    pub fn setvar(&mut self, var: &str, val: f64) {
        self.0.insert(var.to_string(), val);
    }

    pub fn eval(&self, rpn: &RPNExpr) -> Result<f64, EvalErr> {
        let mut operands = Vec::new();

        for token in rpn.0.iter() {
            match *token {
                MathToken::Number(num)       => operands.push(num),
                MathToken::Variable(ref var) => match self.0.get(var) {
                    Some(value) => operands.push(*value),
                    None => return Err(EvalErr::UnknownVar(var.to_string()))
                },
                MathToken::BOp(ref op) => {
                    let r = operands.pop().ok_or(EvalErr::WrongNumberOfArgs)?;
                    let l = operands.pop().ok_or(EvalErr::WrongNumberOfArgs)?;
                    match &op[..] {
                        "+" => operands.push(l + r),
                        "-" => operands.push(l - r),
                        "*" => operands.push(l * r),
                        "/" => operands.push(l / r),
                        "%" => operands.push(l % r),
                        "^" => operands.push(l.powf(r)),
                        _ => return Err(EvalErr::BadToken(op.clone()))
                    }
                },
                MathToken::UOp(ref op) => {
                    let o = operands.pop().ok_or(EvalErr::WrongNumberOfArgs)?;
                    match &op[..] {
                        "-" => operands.push(-o),
                        "!" => match Self::eval_fn("tgamma", vec![o + 1.0]) {
                            Ok(n) => operands.push(n),
                            Err(e) => return Err(e)
                        },
                        _ => return Err(EvalErr::BadToken(op.clone()))
                    }
                },
                MathToken::Function(ref fname, arity) => {
                    if arity > operands.len() {
                        return Err(EvalErr::WrongNumberOfArgs);
                    }
                    let cut = operands.len() - arity;
                    let args = operands.split_off(cut);
                    match Self::eval_fn(fname, args) {
                        Ok(n) => operands.push(n),
                        Err(e) => return Err(e)
                    }
                },
                _ => return Err(EvalErr::BadToken(format!("{:?}", *token)))
            }
        }
        operands.pop().ok_or(EvalErr::WrongNumberOfArgs)
    }

    fn eval_fn(fname: &str, args: Vec<f64>) -> Result<f64, EvalErr> {
        match fname {
            "sin"   => nargs!(args.len() == 1, Ok(args[0].sin())),
            "cos"   => nargs!(args.len() == 1, Ok(args[0].cos())),
            "atan2" => nargs!(args.len() == 2, Ok(args[0].atan2(args[1]))),
            "max"   => nargs!(args.len() > 0,
                              Ok(args[1..].iter().fold(args[0], |a, &item| a.max(item)))),
            "min"   => nargs!(args.len() > 0,
                              Ok(args[1..].iter().fold(args[0], |a, &item| a.min(item)))),
            "abs"   => nargs!(args.len() == 1, Ok(f64::abs(args[0]))),
            "rand"  => nargs!(args.len() == 1, Ok(args[0] * rand::random::<f64>())),
            _       => match mathlink::link_fn(fname) {
                Ok(func) => nargs!(args.len() == 1, Ok(func(args[0]))),
                Err(e) => Err(EvalErr::LinkError(e))
            }
        }
    }
}

#[cfg(feature="dynlink-eval")]
mod mathlink {
    use std::mem;
    pub fn link_fn(fname: &str) -> Result<fn(f64) -> f64, String> {
        match dylib::DynamicLibrary::open(None) {
            Ok(lib) => unsafe {
                match lib.symbol(fname) {
                    Ok(f) => Ok(mem::transmute::<*mut u8, fn(f64) -> f64>(f)),
                    Err(e) => Err(e)
                }
            },
            Err(e) => Err(e)
        }
    }
}

#[cfg(not(feature="dynlink-eval"))]
mod mathlink {
    pub fn link_fn(fname: &str) -> Result<fn(f64) -> f64, String> {
        Err(format!("Dynamic linking not enabled, unknown function: {}", fname))
    }
}
