use std::collections::HashMap;
use lexers::MathToken;
use crate::parser::RPNExpr;


// a shorthand for checking number of arguments before eval_fn
macro_rules! nargs {
    ($argcheck:expr, $ifok:expr) => {
        if $argcheck { $ifok } else { Err(format!("Wrong number of arguments")) }
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

    pub fn eval(&self, rpn: &RPNExpr) -> Result<f64, String> {
        let mut operands = Vec::new();

        for token in rpn.0.iter() {
            match *token {
                MathToken::Number(num)       => operands.push(num),
                MathToken::Variable(ref var) => match self.0.get(var) {
                    Some(value) => operands.push(*value),
                    None => return Err(format!("Unknown Variable: {}", var.to_string()))
                },
                MathToken::BOp(ref op) => {
                    let r = operands.pop().ok_or(format!("Wrong number of arguments"))?;
                    let l = operands.pop().ok_or(format!("Wrong number of arguments"))?;
                    match &op[..] {
                        "+" => operands.push(l + r),
                        "-" => operands.push(l - r),
                        "*" => operands.push(l * r),
                        "/" => operands.push(l / r),
                        "%" => operands.push(l % r),
                        "^" => operands.push(l.powf(r)),
                        _ => return Err(format!("Bad Token: {}", op.clone()))
                    }
                },
                MathToken::UOp(ref op) => {
                    let o = operands.pop().ok_or(format!("Wrong number of arguments"))?;
                    match &op[..] {
                        "-" => operands.push(-o),
                        "!" => operands.push(Self::eval_fn("tgamma", vec![o + 1.0])?),
                        _ => return Err(format!("Bad Token: {}", op.clone()))
                    }
                },
                MathToken::Function(ref fname, arity) => {
                    if arity > operands.len() {
                        return Err(format!("Wrong number of arguments"));
                    }
                    let cut = operands.len() - arity;
                    let args = operands.split_off(cut);
                    operands.push(Self::eval_fn(fname, args)?)
                },
                _ => return Err(format!("Bad Token: {:?}", *token))
            }
        }
        operands.pop().ok_or(format!("Wrong number of arguments"))
    }

    fn eval_fn(fname: &str, args: Vec<f64>) -> Result<f64, String> {
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
            // Resolve function fname and call it
            _       => nargs!(args.len() == 1, Ok((mathlink::link_fn(fname)?)(args[0])))
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
