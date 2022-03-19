use crate::parser::RPNExpr;
use lexers::MathToken;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;


pub trait RandomVariable {
    fn eval(&self) -> f64;
}

impl<D: rand::distributions::Distribution<f64>> RandomVariable for D {
    fn eval(&self) -> f64 {
        self.sample(&mut rand::thread_rng())
    }
}

pub enum MathOp {
    Number(f64),
    Variable(String, MathContext),
    RandVar(Box<dyn RandomVariable>),
    Function(Box<dyn Fn() -> f64>),
}

impl RandomVariable for MathOp {
    fn eval(&self) -> f64 {
        match self {
            MathOp::Number(n) => *n,
            MathOp::RandVar(r) => r.eval(),
            MathOp::Function(f) => f(),
            MathOp::Variable(v, cx) => match cx.0.borrow().get(v) {
                Some(rv) => rv.eval(),
                None => panic!("Variable {} not in context", v),
            },
        }
    }
}

#[derive(Clone)]
pub struct MathContext(Rc<RefCell<HashMap<String, MathOp>>>);

impl MathContext {
    pub fn new() -> MathContext {
        use std::f64::consts;
        let mut cx = HashMap::new();
        cx.insert("pi".to_string(), MathOp::Number(consts::PI));
        cx.insert("e".to_string(), MathOp::Number(consts::E));
        MathContext(Rc::new(RefCell::new(cx)))
    }

    pub fn setvar(&self, name: &str, value: MathOp) {
        self.0.borrow_mut().insert(name.to_string(), value);
    }

    pub fn eval(&self, rpn: &RPNExpr) -> Result<f64, String> {
        let mut operands = Vec::new();

        for token in &rpn.0 {
            match token {
                MathToken::Number(num) => operands.push(*num),
                MathToken::Variable(ref v) => operands.push(
                    match self.0.borrow().get(v) {
                        Some(mathop) => mathop.eval(),
                        None => return Err(format!("Unknown Variable: {}", v)),
                    }
                ),
                MathToken::BOp(op) => {
                    let rhs = operands.pop().ok_or("Missing operands")?;
                    let lhs = operands.pop().ok_or("Missing operands")?;
                    operands.push(match &op[..] {
                        "+" => lhs + rhs,
                        "-" => lhs - rhs,
                        "*" => lhs * rhs,
                        "/" => lhs / rhs,
                        "%" => lhs % rhs,
                        "^" | "**" => lhs.powf(rhs),
                        _ => return Err(format!("Unknown BOp: {}", op)),
                    });
                }
                MathToken::UOp(op) => {
                    let arg = operands.pop().ok_or("Missing operands")?;
                    operands.push(match &op[..] {
                        "-" => -arg,
                        "!" => libm::tgamma(arg + 1.0),
                        _ => return Err(format!("Unknown UOp: {}", op)),
                    });
                }
                MathToken::Function(fname, arity) => {
                    if *arity > operands.len() {
                        return Err(format!("Missing args for function {}", fname));
                    }
                    let args: Vec<_> = operands.split_off(operands.len() - arity);
                    operands.push(
                        eval_fn(fname, &args).or_else::<String, _>(
                            |_| Ok(build_rv(fname, &args)?.eval()))?);
                }
                _ => return Err(format!("Unexpected token for RPN eval: {:?}", token)),
            }
        }
        operands.pop().ok_or(format!("Failed to eval RPN: {:?}", rpn))
    }

    pub fn compile(&self, rpn: &RPNExpr) -> Result<MathOp, String> {
        let mut stack = Vec::new();
        for token in &rpn.0 {
            match token {
                MathToken::Number(n) => stack.push(MathOp::Number(*n)),
                MathToken::Variable(v) => stack.push(
                    MathOp::Variable(v.to_string(), self.clone())),
                MathToken::BOp(op) => {
                    let op = op.clone();
                    let rhs = stack.pop().ok_or("Missing operands")?;
                    let lhs = stack.pop().ok_or("Missing operands")?;
                    match (lhs, rhs) {
                        (MathOp::Number(lhs), MathOp::Number(rhs)) => stack.push(
                            MathOp::Number(match &op[..] {
                                "+" => lhs + rhs,
                                "-" => lhs - rhs,
                                "*" => lhs * rhs,
                                "/" => lhs / rhs,
                                "%" => lhs % rhs,
                                "^" | "**" => lhs.powf(rhs),
                                _ => return Err(format!("Unknown BOp: {}", op)),
                            })),
                        (lhs, rhs) => stack.push(
                            MathOp::Function(Box::new(move || match &op[..] {
                                "+" => lhs.eval() + rhs.eval(),
                                "-" => lhs.eval() - rhs.eval(),
                                "*" => lhs.eval() * rhs.eval(),
                                "/" => lhs.eval() / rhs.eval(),
                                "%" => lhs.eval() % rhs.eval(),
                                "^" | "**" => lhs.eval().powf(rhs.eval()),
                                _ => panic!("Unknown BOp: {}", op)
                            })))
                    }
                }
                MathToken::UOp(op) => {
                    let op = op.clone();
                    match stack.pop().ok_or("Missing operands")? {
                        MathOp::Number(arg) => stack.push(
                            MathOp::Number(match &op[..] {
                                "-" => -arg,
                                "!" => libm::tgamma(arg + 1.0),
                                _ => return Err(format!("Unknown UOp: {}", op)),
                            })),
                        arg => stack.push(
                            MathOp::Function(Box::new(move || match &op[..] {
                                "-" => -arg.eval(),
                                "!" => libm::tgamma(arg.eval() + 1.0),
                                _ => panic!("Unknown UOp: {}", op)
                            }))
                        )
                    }
                }
                MathToken::Function(fname, arity) => {
                    if *arity > stack.len() {
                        return Err(format!("Missing args for function {}", fname));
                    }
                    let args: Vec<_> = stack.split_off(stack.len() - arity);
                    // All arguments to function call are numbers (static)
                    if args.iter().all(|a| matches!(a, MathOp::Number(_))) {
                        // All args are numbers, use .eval to extract f64s
                        let args: Vec<_> = args.iter().map(|v| v.eval()).collect();
                        if let Ok(rv) = build_rv(fname, &args) {
                            stack.push(MathOp::RandVar(rv));
                        } else {
                            stack.push(MathOp::Number(eval_fn(fname, &args)?));
                        }
                    } else {
                        // Args for function call determined at runtime.
                        let fname = fname.clone();
                        stack.push(MathOp::Function(Box::new(move || {
                            let args: Vec<_> = args.iter().map(|v| v.eval()).collect();
                            if let Ok(rv) = build_rv(&fname, &args) {
                                rv.eval()
                            } else {
                                eval_fn(&fname, &args).unwrap()
                            }
                        })));
                    }
                }
                _ => return Err(format!("Unexpected token for RPN compile: {:?}", token)),
            }
        }
        assert_eq!(stack.len(), 1);
        Ok(stack.pop().ok_or("Failed to compile RPNExpr")?)
    }
}

fn eval_fn(fname: &str, args: &[f64]) -> Result<f64, String> {
    Ok(match fname {
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
    })
}

fn build_rv(dname: &str, args: &[f64]) -> Result<Box<dyn RandomVariable>, String> {
    use rand_distr::*;
    Ok(match dname {
        "normal" if args.len() == 2 => Box::new(Normal::new(args[0], args[1]).unwrap()),
        "uniform" if args.len() == 2 => Box::new(Uniform::new(args[0], args[1])),
        "lognormal" if args.len() == 2 => Box::new(LogNormal::new(args[0], args[1]).unwrap()),
        _ => return Err(format!("Unknown distribution: {} with {} args", dname, args.len()))
    })
}

mod funcs {
    pub fn combinations(n: f64, r: f64) -> Result<f64, String> {
        use libm::tgamma;
        Ok(tgamma(n + 1.0) / tgamma(r + 1.0) / tgamma(n - r + 1.0))
    }

    pub fn multicombinations(n: f64, r: f64) -> Result<f64, String> {
        use libm::tgamma;
        Ok(tgamma(n + r) / tgamma(r + 1.0) / tgamma(n))
    }

    pub fn permutations(n: f64, r: f64) -> Result<f64, String> {
        use libm::tgamma;
        Ok(tgamma(n + 1.0) / tgamma(n - r + 1.0))
    }
}
