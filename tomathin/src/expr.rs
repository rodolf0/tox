use core::fmt;

use crate::context::Context;
use crate::findroot;
use crate::parser::Expr;

// Lowest number is highest precedence
fn precedence(e: &Expr) -> usize {
    match e {
        Expr::Number(_) => 0,
        Expr::Symbol(_) => 1,
        Expr::Expr(head, _) => match head.as_ref() {
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

                let mut new_args = vec![args.remove(0)];
                for arg in args {
                    match eval_with_ctx(arg, ctx)? {
                        rhs @ Expr::Number(rhsn) => {
                            new_args = new_args
                                .into_iter()
                                .map(|argi| match argi {
                                    Expr::Number(lhs) => Expr::Number(lhs * rhsn),
                                    lhs => distribute_op(lhs, rhs.clone(), "Times", "List"),
                                })
                                .collect();
                        }
                        rhs => {
                            new_args = new_args
                                .into_iter()
                                .map(|lhsi| distribute_op(lhsi, rhs.clone(), "Times", "List"))
                                .collect();
                        }
                    }
                }
                if new_args.len() == 1 {
                    Ok(new_args.swap_remove(0))
                } else {
                    Ok(Expr::Expr(head, new_args))
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
            "FindRoot" => {
                let Some(Expr::Expr(list_head, var_args)) = args.get(1) else {
                    return Err(format!("FindRoot unexpected arg1: {:?}", args.get(1)));
                };
                let (x, x0) = match (list_head, var_args.as_slice()) {
                    (h, [Expr::Symbol(x), Expr::Number(x0)]) if h == "List" => (x.clone(), *x0),
                    other => return Err(format!("FindRoot unexpected arg1: {:?}", other)),
                };
                let fexpr = eval_with_ctx(args.swap_remove(0), ctx)?;
                // NOTE: not super performant because of iteration of replace + eval
                let f = |xi: f64| match evaluate(replace_all(
                    fexpr.clone(),
                    &[(Expr::Symbol(x.clone()), Expr::Number(xi))],
                )?)? {
                    Expr::Number(x1) => Ok(x1),
                    other => Err(format!(
                        "FindRoot expr evaluation didn't return Number: {:?}",
                        other
                    )),
                };
                let root = findroot::find_root(f, x0)?;
                Ok(Expr::Number(root))
            }
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
            other => panic!("{} head not implemented", other),
        },
        Expr::Symbol(ref sym) => match ctx.get(sym) {
            Some(expr_lookup) => Ok(eval_with_ctx(expr_lookup, ctx)?),
            None => Ok(expr),
        },
        _ => Ok(expr),
    }
}

pub fn eval_replace_all(expr: Expr, ctx: &mut Context) -> Result<Expr, String> {
    // Eval of replace_all is outside of eval because evaluation of expr is deferred
    match expr {
        Expr::Expr(head, args) if head == "ReplaceAll" => {
            let [expr, rules]: [Expr; 2] = args
                .try_into()
                .map_err(|e| format!("ReplaceAll must have 2 arguments. {:?}", e))?;
            eval_with_ctx(
                replace_all(
                    // Nested evaluation of replace_all without eval of expr
                    eval_replace_all(expr, ctx)?,
                    eval_rules(rules, ctx)?.as_slice(),
                )?,
                ctx,
            )
        }
        other => Ok(other),
    }
}

fn eval_rules(rules: Expr, ctx: &mut Context) -> Result<Vec<(Expr, Expr)>, String> {
    // eval_with_ctx rules and check they result in Rule or List[Rule]
    match eval_with_ctx(rules, ctx)? {
        Expr::Expr(head, args) if head == "Rule" => {
            let [lhs, rhs]: [Expr; 2] = args
                .try_into()
                .map_err(|e| format!("Rule must have 2 arguments. {:?}", e))?;
            Ok(vec![(lhs, rhs)])
        }
        Expr::Expr(h, args) if h == "List" => args
            .into_iter()
            .map(|rule| match rule {
                Expr::Expr(head, args) if head == "Rule" => {
                    let [lhs, rhs]: [Expr; 2] = args
                        .try_into()
                        .map_err(|e| format!("Rule must have 2 arguments. {:?}", e))?;
                    Ok((lhs, rhs))
                }
                other => Err(format!("Expected all args to be Rule: {:?}", other)),
            })
            .collect(),
        other => Err(format!("ReplaceAll unexpected arg: '{:?}'", other)),
    }
}

// ReplaceAll[x, Rule[x, 3]]
// ReplaceAll[List[1, 2, 3], Rule[List, FindRoot]]
fn replace_all(expr: Expr, rules: &[(Expr, Expr)]) -> Result<Expr, String> {
    match expr {
        // for each sub-expression apply the replacement
        Expr::Expr(head, args) => {
            // First execute replace_all on subexpressions.
            let replaced_expr = Expr::Expr(
                head,
                args.into_iter()
                    .map(|a| replace_all(a, rules))
                    .collect::<Result<_, _>>()?,
            );
            // Check if update expression matches a replacement too.
            let replaced_expr = rules
                .iter()
                .filter(|(lhs, _)| replaced_expr == *lhs)
                .next()
                .map(|(_, rhs)| rhs.clone())
                .unwrap_or(replaced_expr);
            // Check if a rule is a head re-write
            Ok(match replaced_expr {
                Expr::Expr(head, args) => {
                    for r in rules {
                        if let (Expr::Symbol(lhs), Expr::Symbol(rhs)) = r {
                            if *lhs == head {
                                return Ok(Expr::Expr(rhs.clone(), args));
                            }
                        }
                    }
                    Expr::Expr(head, args)
                }
                expr => expr,
            })
        }
        atom => Ok(rules
            .iter()
            .filter(|(lhs, _)| atom == *lhs)
            .next()
            .map(|(_, rhs)| rhs.clone())
            .unwrap_or(atom)),
    }
}
