use crate::parser::RPNExpr;
use lexers::MathToken;
use std::collections::HashMap;
use std::rc::Rc;


pub trait RandomVariable {
    fn sample(&self) -> f64;
}

impl<D: rand::distributions::Distribution<f64>> RandomVariable for D {
    fn sample(&self) -> f64 {
        self.sample(&mut rand::thread_rng())
    }
}

pub enum MathValue {
    Number(f64),
    RandVar(Rc<dyn RandomVariable>),
}

pub struct MathContext(pub HashMap<String, MathValue>);

impl MathContext {
    pub fn new() -> MathContext {
        use std::f64::consts;
        let mut cx = HashMap::new();
        cx.insert("pi".to_string(), MathValue::Number(consts::PI));
        cx.insert("e".to_string(), MathValue::Number(consts::E));
        MathContext(cx)
    }

    pub fn setvar(&mut self, var: &str, val: MathValue) {
        self.0.insert(var.to_string(), val);
    }

    pub fn eval(&self, rpn: &RPNExpr) -> Result<MathValue, String> {
        let mut operands = Vec::new();

        for token in &rpn.0 {
            match token {
                MathToken::Number(num) => operands.push(MathValue::Number(*num)),
                MathToken::Variable(ref var) => match self.0.get(var) {
                    Some(MathValue::Number(n)) => operands.push(MathValue::Number(*n)),
                    Some(MathValue::RandVar(r)) => operands.push(MathValue::RandVar(r.clone())),
                    None => return Err(format!("Unknown Variable: {}", var)),
                },
                MathToken::BOp(op) => {
                    let r = match operands.pop() {
                        Some(MathValue::Number(n)) => n,
                        Some(MathValue::RandVar(x)) => x.sample(),
                        None => return Err(format!("Missing args for operator {}", op)),
                    };
                    let l = match operands.pop() {
                        Some(MathValue::Number(n)) => n,
                        Some(MathValue::RandVar(x)) => x.sample(),
                        None => return Err(format!("Missing args for operator {}", op)),
                    };
                    match &op[..] {
                        "+" => operands.push(MathValue::Number(l + r)),
                        "-" => operands.push(MathValue::Number(l - r)),
                        "*" => operands.push(MathValue::Number(l * r)),
                        "/" => operands.push(MathValue::Number(l / r)),
                        "%" => operands.push(MathValue::Number(l % r)),
                        "^" | "**" => operands.push(MathValue::Number(l.powf(r))),
                        _ => return Err(format!("Unknown BOp: {}", op)),
                    }
                }
                MathToken::UOp(op) => {
                    let o = match operands.pop() {
                        Some(MathValue::Number(n)) => n,
                        Some(MathValue::RandVar(x)) => x.sample(),
                        None => return Err(format!("Missing args for operator {}", op)),
                    };
                    match &op[..] {
                        "-" => operands.push(MathValue::Number(-o)),
                        "!" => operands.push(Self::eval_fn("tgamma", &[o + 1.0])?),
                        _ => return Err(format!("Unknown UOp: {}", op)),
                    }
                }
                MathToken::Function(fname, arity) => {
                    if *arity > operands.len() {
                        return Err(format!("Missing args for function {}", fname));
                    }
                    let args: Vec<_> = operands.split_off(operands.len() - arity)
                        .into_iter()
                        .map(|arg| match arg {
                            MathValue::Number(n) => n,
                            MathValue::RandVar(x) => x.sample(),
                        }).collect();
                    operands.push(Self::eval_fn(fname, &args).or(Self::eval_rv(fname, &args))?)
                }
                _ => return Err(format!("Bad Token: {:?}", token)),
            }
        }
        operands.pop().ok_or(format!("Failed to eval: {:?}", rpn))
    }

    fn eval_fn(fname: &str, args: &[f64]) -> Result<MathValue, String> {
        Ok(MathValue::Number(match fname {
            "sin" if args.len() == 1 => args[0].sin(),
            "cos" if args.len() == 1 => args[0].cos(),
            "atan2" if args.len() == 2 => args[0].atan2(args[1]),
            "max" if !args.is_empty() => args.iter().fold(args[0], |a, &b| a.max(b)),
            "min" if !args.is_empty() => args.iter().fold(args[0], |a, &b| a.min(b)),
            "abs" if args.len() == 1 => args[0].abs(),
            "rand" if args.len() == 1 => rand::random::<f64>() * args[0],
            // Order not important
            "nCr" if args.len() == 2 => funcs::combinations(args[0], args[1])?,
            "nMCr" if args.len() == 2 => funcs::multicombinations(args[0], args[1])?,
            // Order is important
            "nPr" if args.len() == 2 => funcs::permutations(args[0], args[1])?,
            "nMPr" if args.len() == 2 => args[0].powf(args[1]),
            _ => return Err(format!("Unknown Function: {} with {} args", fname, args.len()))
        }))
    }

    fn eval_rv(dname: &str, args: &[f64]) -> Result<MathValue, String> {
        use rand_distr::*;
        Ok(MathValue::RandVar(match dname {
            "normal" if args.len() == 2 => Rc::new(Normal::new(args[0], args[1]).unwrap()),
            "uniform" if args.len() == 2 => Rc::new(Uniform::new(args[0], args[1])),
            "lognormal" if args.len() == 2 => Rc::new(LogNormal::new(args[0], args[1]).unwrap()),
            _ => return Err(format!("Unknown distribution: {} with {} args", dname, args.len()))
        }))
    }
}

mod funcs {
    use super::mathlink;
    pub fn combinations(n: f64, r: f64) -> Result<f64, String> {
        let tgamma = mathlink::link_fn("tgamma")?;
        Ok(tgamma(n + 1.0) / tgamma(r + 1.0) / tgamma(n - r + 1.0))
    }

    pub fn multicombinations(n: f64, r: f64) -> Result<f64, String> {
        let tgamma = mathlink::link_fn("tgamma")?;
        Ok(tgamma(n + r) / tgamma(r + 1.0) / tgamma(n))
    }

    pub fn permutations(n: f64, r: f64) -> Result<f64, String> {
        let tgamma = mathlink::link_fn("tgamma")?;
        Ok(tgamma(n + 1.0) / tgamma(n - r + 1.0))
    }
}

#[cfg(feature = "dynlink-eval")]
mod mathlink {
    use std::mem;
    pub fn link_fn(fname: &str) -> Result<fn(f64) -> f64, String> {
        match dylib::DynamicLibrary::open(None) {
            Ok(lib) => unsafe {
                match lib.symbol(fname) {
                    Ok(f) => Ok(mem::transmute::<*mut u8, fn(f64) -> f64>(f)),
                    Err(e) => Err(e),
                }
            },
            Err(e) => Err(e),
        }
    }
}

#[cfg(not(feature = "dynlink-eval"))]
mod mathlink {
    pub fn link_fn(fname: &str) -> Result<fn(f64) -> f64, String> {
        Err(format!(
            "Dynamic linking not enabled, unknown function: {}",
            fname
        ))
    }
}
