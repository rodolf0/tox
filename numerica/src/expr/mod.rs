mod distribution;
mod find_root;
mod replace_all;
mod sum;
mod table;
mod times;
mod transcendental;

use distribution::{Distr, eval_normal_dist};
use find_root::eval_find_root;
use replace_all::{eval_replace_all, replace_all};
use table::eval_table;
use times::eval_times;

use core::fmt;
use std::rc::Rc;

use crate::context::Context;

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    // Expr2 { head: Box<Expr>, args: Vec<Expr> },
    Expr(String, Vec<Expr>),
    Symbol(String),
    Number(f64),
    Bool(bool),
    String(String),
    Distribution(Rc<Distr>),
    // DateTime(DateTime<Utc>),
    // Matrix(Matrix),
    // Quantity(f64, Dimension),
}

// Lowest number is highest precedence
fn precedence(e: &Expr) -> usize {
    match e {
        Expr::Number(_) => 0,
        Expr::Symbol(_) => 1,
        Expr::Expr(head, _) => match head.as_ref() {
            "List" => 3,
            "Sin" | "Cos" | "Exp" => 5,
            "Power" => 50,
            "Divide" => 60,
            "Times" => 65,
            "Plus" => 70,
            "Minus" => 75,
            _ => 1000,
        },
        _ => 1000,
    }
}

fn join_args(e: &Expr, sep: &str) -> String {
    let parent_p = precedence(e);
    let Expr::Expr(_, args) = e else {
        panic!("BUG: Tried to join_args for non Expr: {:?}", e);
    };
    args.iter()
        .map(|a| {
            if parent_p < precedence(a) {
                format!("({})", a.to_string())
            } else {
                a.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(sep)
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Expr::Symbol(s) => write!(f, "{}", s),
            Expr::Number(n) => write!(f, "{}", n),
            Expr::Expr(head, _) => match head.as_ref() {
                "Plus" => write!(f, "{}", join_args(self, " + ")),
                "Times" => write!(f, "{}", join_args(self, " * ")),
                "Minus" => write!(f, "{}", join_args(self, " - ")),
                "Divide" => write!(f, "{}", join_args(self, " / ")),
                "Power" => write!(f, "{}", join_args(self, " ^ ")),
                "List" => write!(f, "{{{}}}", join_args(self, ", ")),
                _ => write!(f, "{}[{}]", head, join_args(self, ", ")),
            },
            _ => write!(f, "{:?}", self),
        }
    }
}

pub fn evaluate(expr: Expr) -> Result<Expr, String> {
    eval_with_ctx(expr, &mut Context::new())
}

pub fn eval_with_ctx(expr: Expr, ctx: &mut Context) -> Result<Expr, String> {
    match expr {
        Expr::Expr(head, args) => match head.as_ref() {
            "Hold" => Ok(Expr::Expr(head, args)),
            "Evaluate" => {
                let [arg]: [Expr; 1] = args
                    .try_into()
                    .map_err(|e| format!("Evaluate expects single arg. {:?}", e))?;
                let rules = vec![(
                    Expr::Symbol("Hold".to_string()),
                    Expr::Symbol("Evaluate".to_string()),
                )];
                // TODO: this is a bit of a hack, should replace only heads
                eval_with_ctx(replace_all(arg, &rules)?, ctx)
            }
            "List" => Ok(Expr::Expr(
                head,
                args.into_iter()
                    .map(|a| eval_with_ctx(a, ctx))
                    .collect::<Result<_, _>>()?,
            )),
            "Rule" => {
                let [lhs, rhs]: [Expr; 2] = args
                    .try_into()
                    .map_err(|e| format!("Rule must have 2 arguments. {:?}", e))?;
                Ok(Expr::Expr(head, vec![lhs, eval_with_ctx(rhs, ctx)?]))
            }
            "ReplaceAll" => eval_replace_all(Expr::Expr(head, args), ctx),
            "Plus" => {
                // Flatten operations that are commutative and associative
                let mut numeric: Option<f64> = None;
                let mut new_args = Vec::new();
                for arg in args {
                    match eval_with_ctx(arg, ctx)? {
                        Expr::Number(n) => *numeric.get_or_insert(0.0) += n,
                        o => new_args.push(o),
                    }
                }
                if numeric.is_some_and(|n| n != 0.0) || new_args.len() == 0 {
                    new_args.insert(0, Expr::Number(*numeric.get_or_insert(0.0)));
                }
                if new_args.len() == 1 {
                    Ok(new_args.swap_remove(0))
                } else {
                    Ok(Expr::Expr(head, new_args))
                }
            }
            "Times" => eval_times(args, ctx),
            "Minus" | "Power" | "Divide" => {
                let [lhs, rhs]: [Expr; 2] = args
                    .try_into()
                    .map_err(|e| format!("{} must have 2 arguments. {:?}", head, e))?;
                let lhs = eval_with_ctx(lhs, ctx)?;
                let rhs = eval_with_ctx(rhs, ctx)?;
                Ok(match (lhs, rhs) {
                    (Expr::Number(lhs), Expr::Number(rhs)) => match head.as_ref() {
                        "Minus" => Expr::Number(lhs - rhs),
                        "Power" => Expr::Number(lhs.powf(rhs)),
                        "Divide" => Expr::Number(lhs / rhs),
                        _ => panic!("BUG: {} op not implemented", head),
                    },
                    (lhs, rhs) => Expr::Expr(head, vec![lhs, rhs]),
                })
            }
            "FindRoot" => eval_find_root(args, ctx),
            "Sum" => sum::eval_sum(args, ctx),
            "SetDelayed" | "Set" => {
                let [lhs, rhs]: [Expr; 2] = args
                    .try_into()
                    .map_err(|e| format!("Set(Delayed) must have 2 arguments. {:?}", e))?;
                let (lhs, rhs) = match lhs {
                    Expr::Symbol(lhs) if head == "Set" => (lhs, eval_with_ctx(rhs, ctx)?),
                    Expr::Symbol(lhs) if head == "SetDelayed" => (lhs, rhs),
                    _ => return Err(format!("Set(Delayed) lhs must be a symbol. {:?}", lhs)),
                };
                ctx.set(lhs, rhs.clone());
                Ok(rhs)
            }
            "Gamma" => transcendental::eval_gamma(args, ctx),
            "NormalDist" => eval_normal_dist(args, ctx),
            "Sin" => transcendental::eval_sin(args, ctx),
            "Cos" => transcendental::eval_cos(args, ctx),
            "Exp" => transcendental::eval_exp(args, ctx),
            "Table" => eval_table(args, ctx),
            otherhead => match ctx.get(otherhead) {
                Some(Expr::Expr(h, function_args)) if h == "Function" => {
                    // Destructure Function[{params}, body]]
                    let [params, body]: [Expr; 2] = function_args
                        .try_into()
                        .map_err(|e| format!("Function must have params and body. {:?}", e))?;
                    // Evaluate function call inputs
                    let evaled_args: Vec<_> = args
                        .into_iter()
                        .map(|ai| eval_with_ctx(ai, ctx))
                        .collect::<Result<_, _>>()?;
                    // Bind params to current values passed as args to this call
                    let bindings: Vec<_> = match params {
                        sym @ Expr::Symbol(_) => [sym].into_iter().zip(evaled_args).collect(),
                        Expr::Expr(h, syms) if h == "List" => {
                            if !syms.iter().all(|s| matches!(s, Expr::Symbol(_))) {
                                return Err(format!("Function params must be symbols: {:?}", syms));
                            }
                            syms.into_iter().zip(evaled_args).collect()
                        }
                        other => {
                            return Err(format!("Function params must be symbols: {}", other));
                        }
                    };
                    // Replace instances of function parameters in the callable and evaluate function
                    eval_with_ctx(replace_all(body, &bindings)?, ctx)
                }
                _ => Ok(Expr::Expr(head, args)),
            },
        },
        Expr::Symbol(ref sym) => match ctx.get(sym) {
            Some(expr_lookup) => Ok(eval_with_ctx(expr_lookup, ctx)?),
            None => Ok(expr),
        },
        Expr::Distribution(d) => Ok(Expr::Number(d.sample())),
        _ => Ok(expr),
    }
}
