mod find_root;
mod replace_all;
mod table;

use find_root::eval_find_root;
use replace_all::{eval_replace_all, replace_all};
use table::eval_table;

use core::fmt;
use rand_distr::Distribution;
use std::rc::Rc;

use crate::context::Context;

#[derive(Debug, PartialEq)]
pub enum Distr {
    Normal(rand_distr::Normal<f64>),
    Poisson(rand_distr::Poisson<f64>),
}

impl Distr {
    fn sample(&self) -> f64 {
        match self {
            Distr::Normal(d) => d.sample(&mut rand::rng()),
            Distr::Poisson(d) => d.sample(&mut rand::rng()),
        }
    }
}

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

fn distribute_op(lhs: Expr, rhs: Expr, op: &str, over: &str) -> Expr {
    match lhs {
        Expr::Expr(h, lhs) if h == over => match rhs {
            Expr::Expr(h, rhs) if h == over => Expr::Expr(
                over.to_string(),
                lhs.iter()
                    .flat_map(|lhsi| {
                        rhs.iter()
                            .map(|rhsi| distribute_op(lhsi.clone(), rhsi.clone(), op, over))
                    })
                    .collect(),
            ),
            rhs => Expr::Expr(
                over.to_string(),
                lhs.into_iter()
                    .map(|lhsi| distribute_op(lhsi, rhs.clone(), op, over))
                    .collect(),
            ),
        },
        lhs => match rhs {
            Expr::Expr(h, rhs) if h == over => Expr::Expr(
                over.to_string(),
                rhs.into_iter()
                    .map(|rhsi| distribute_op(lhs.clone(), rhsi, op, over))
                    .collect(),
            ),
            rhs => Expr::Expr(op.to_string(), vec![lhs, rhs]),
        },
    }
}

fn flatten(expr: Expr, op: &str) -> Expr {
    match expr {
        Expr::Expr(head, args) if head == op => Expr::Expr(
            head,
            args.into_iter()
                .flat_map(|ai| match ai {
                    Expr::Expr(h, a) if h == op => {
                        a.into_iter().map(|aj| flatten(aj, op)).collect()
                    }
                    other => vec![flatten(other, op)],
                })
                .collect(),
        ),
        Expr::Expr(head, args) => {
            Expr::Expr(head, args.into_iter().map(|ai| flatten(ai, op)).collect())
        }
        other => other,
    }
}

pub fn evaluate(expr: Expr) -> Result<Expr, String> {
    eval_with_ctx(expr, &mut Context::new())
}

pub fn eval_with_ctx(expr: Expr, ctx: &mut Context) -> Result<Expr, String> {
    match expr {
        Expr::Expr(head, mut args) => match head.as_ref() {
            "Hold" => {
                if args.len() != 1 {
                    Err(format!("Hold expects single arg. {:?}", args))
                } else {
                    Ok(Expr::Expr(head, args))
                }
            }
            "Evaluate" => {
                if args.len() != 1 {
                    Err(format!("Hold expects single arg. {:?}", args))
                } else {
                    let rules = vec![(
                        Expr::Symbol("Hold".to_string()),
                        Expr::Symbol("Evaluate".to_string()),
                    )];
                    // TODO: this is a bit of a hack, should replace only heads
                    eval_with_ctx(replace_all(args.swap_remove(0), &rules)?, ctx)
                }
            }
            "List" => {
                let evaled_args = args
                    .into_iter()
                    .map(|a| eval_with_ctx(a, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(Expr::Expr(head, evaled_args))
            }
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
            "Times" => {
                // distribute over List
                // 3 * {a ,b} => {3 * a, 3 * b}
                // {a, b} * 3 => {3 * a, 3 * b}
                // {a, b} * {x, y} => {a * x, a * y, b * x, b * y}
                // 3 * x => 3 * x
                // 3 * x * {a, b} => {3 * x * a, 3 * x * b}
                // 3 * {4, 5} => {12, 15}

                // Distribute Times over List
                let mut new_args = vec![eval_with_ctx(args.remove(0), ctx)?];
                for arg in args {
                    let rhs = eval_with_ctx(arg, ctx)?;
                    new_args = new_args
                        .into_iter()
                        .map(|lhsi| {
                            flatten(distribute_op(lhsi, rhs.clone(), "Times", "List"), "Times")
                        })
                        .collect();
                }
                // Peel off wrapping Times result of distributing Times over List to avoid inf recurse
                new_args = new_args
                    .into_iter()
                    .flat_map(|expr| match expr {
                        Expr::Expr(h, a) if h == "Times" => a,
                        o => vec![o],
                    })
                    .collect();
                // Run mmultiplication
                let mut numeric: Option<f64> = None;
                let mut new_args2 = Vec::new();
                for arg in new_args {
                    match eval_with_ctx(arg, ctx)? {
                        Expr::Number(n) => *numeric.get_or_insert(1.0) *= n,
                        o => new_args2.push(o),
                    }
                }
                if numeric.is_some_and(|n| n != 1.0) || new_args2.len() == 0 {
                    new_args2.insert(0, Expr::Number(numeric.unwrap()));
                }
                if new_args2.len() == 1 {
                    Ok(new_args2.swap_remove(0))
                } else {
                    Ok(Expr::Expr(head, new_args2))
                }
            }
            "Minus" | "Power" | "Divide" => {
                let [lhs_expr, rhs_expr]: [Expr; 2] = args
                    .try_into()
                    .map_err(|e| format!("{} expects 2 arguments {:?}", head, e))?;
                let lhs = match lhs_expr {
                    Expr::Number(_) => lhs_expr,
                    other => eval_with_ctx(other, ctx)?,
                };
                let rhs = match rhs_expr {
                    Expr::Number(_) => rhs_expr,
                    other => eval_with_ctx(other, ctx)?,
                };
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
            "Sum" => {
                let Some(Expr::Expr(list_head, var_args)) = args.get(1) else {
                    return Err(format!("Sum unexpected arg1: {:?}", args.get(1)));
                };
                if list_head != "List" {
                    return Err(format!("Sum arg1 must be a List: {:?}", list_head));
                }
                let (x, x0, xn) = match var_args.as_slice() {
                    [Expr::Symbol(x), Expr::Number(xn)] => (x.clone(), 0 as i32, *xn as i32),
                    [Expr::Symbol(x), Expr::Number(x0), Expr::Number(xn)] => {
                        (x.clone(), *x0 as i32, *xn as i32)
                    }
                    other => return Err(format!("Sum unexpected arg1: {:?}", other)),
                };
                let sum_expr = args.swap_remove(0);
                let Expr::Number(mut sum) = eval_with_ctx(
                    replace_all(
                        sum_expr.clone(),
                        &[(Expr::Symbol(x.clone()), Expr::Number(x0 as f64))],
                    )?,
                    ctx,
                )?
                else {
                    return Ok(Expr::Expr(
                        "Sum".to_string(),
                        vec![sum_expr, args.swap_remove(0)],
                    ));
                };
                for xi in (x0 + 1)..=xn {
                    sum += match eval_with_ctx(
                        replace_all(
                            sum_expr.clone(),
                            &[(Expr::Symbol(x.clone()), Expr::Number(xi as f64))],
                        )?,
                        ctx,
                    )? {
                        Expr::Number(s) => s,
                        other => panic!("BUG: Non-Number should have exited earlier: {:?}", other),
                    };
                }
                Ok(Expr::Number(sum))
            }
            "SetDelayed" | "Set" => {
                let [lhs, rhs]: [Expr; 2] = args
                    .try_into()
                    .map_err(|e| format!("Set(Delayed) must have 2 arguments. {:?}", e))?;
                let Expr::Symbol(sym) = lhs else {
                    return Err(format!("Set(Delayed) lhs must be a symbol. {:?}", lhs));
                };
                let rhs = if head == "Set" {
                    eval_with_ctx(rhs, ctx)?
                } else {
                    rhs
                };
                ctx.set(sym, rhs.clone());
                Ok(rhs)
            }
            "Gamma" => {
                if args.len() != 1 {
                    Err(format!("Gamma expects single arg. {:?}", args))
                } else {
                    match eval_with_ctx(args.swap_remove(0), ctx)? {
                        Expr::Number(n) => Ok(Expr::Number(crate::gamma(1.0 + n))),
                        other => Ok(Expr::Expr(head, vec![other])),
                    }
                }
            }
            "NormalDist" => {
                let [mu, sigma]: [f64; 2] = args
                    .into_iter()
                    .map(|a| match eval_with_ctx(a, ctx) {
                        Ok(Expr::Number(n)) => Ok(n),
                        Ok(other) => Err(format!("NormalDist params must be number. {:?}", other)),
                        Err(e) => Err(e),
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .map_err(|e| format!("NormalDist error: {:?}", e))?;
                Ok(Expr::Distribution(Rc::new(Distr::Normal(
                    rand_distr::Normal::new(mu, sigma).map_err(|e| e.to_string())?,
                ))))
            }
            "Sin" => match eval_with_ctx(args.swap_remove(0), ctx)? {
                Expr::Number(n) => Ok(Expr::Number(n.sin())),
                other => Ok(Expr::Expr(head, vec![other])),
            },
            "Cos" => match eval_with_ctx(args.swap_remove(0), ctx)? {
                Expr::Number(n) => Ok(Expr::Number(n.cos())),
                other => Ok(Expr::Expr(head, vec![other])),
            },
            "Exp" => match eval_with_ctx(args.swap_remove(0), ctx)? {
                Expr::Number(n) => Ok(Expr::Number(n.exp())),
                other => Ok(Expr::Expr(head, vec![other])),
            },
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
