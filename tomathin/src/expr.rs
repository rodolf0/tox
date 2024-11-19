use crate::findroot;
use crate::parser::Expr;

pub fn evaluate(expr: Expr) -> Result<Expr, String> {
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
                    evaluate(replace_all(args.swap_remove(0), &rules)?)
                }
            }
            "List" => {
                let evaled_args = args
                    .into_iter()
                    .map(|a| evaluate(a))
                    .collect::<Result<_, _>>()?;
                Ok(Expr::Expr(head, evaled_args))
            }
            "Rule" => {
                let [lhs, rhs]: [Expr; 2] = args
                    .try_into()
                    .map_err(|e| format!("Rule must have 2 arguments. {:?}", e))?;
                Ok(Expr::Expr(head, vec![lhs, evaluate(rhs)?]))
            }
            "ReplaceAll" => eval_replace_all(Expr::Expr(head, args)),
            "Plus" | "Times" => {
                let mut numeric: Option<f64> = None;
                let mut new_args = Vec::new();
                for arg in args {
                    match evaluate(arg)? {
                        Expr::Number(n) if head == "Plus" => *numeric.get_or_insert(0.0) += n,
                        Expr::Number(n) if head == "Times" => *numeric.get_or_insert(1.0) *= n,
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
            "Minus" | "Power" | "Divide" => {
                let [lhs_expr, rhs_expr]: [Expr; 2] = args
                    .try_into()
                    .map_err(|e| format!("{} expects 2 arguments {:?}", head, e))?;
                let lhs = match lhs_expr {
                    Expr::Number(_) => lhs_expr,
                    other => evaluate(other)?,
                };
                let rhs = match rhs_expr {
                    Expr::Number(_) => rhs_expr,
                    other => evaluate(other)?,
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
                let fexpr = evaluate(args.swap_remove(0))?;
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
                let Expr::Number(mut sum) = evaluate(replace_all(
                    sum_expr.clone(),
                    &[(Expr::Symbol(x.clone()), Expr::Number(x0 as f64))],
                )?)?
                else {
                    return Ok(Expr::Expr(
                        "Sum".to_string(),
                        vec![sum_expr, args.swap_remove(0)],
                    ));
                };
                for xi in (x0 + 1)..=xn {
                    sum += match evaluate(replace_all(
                        sum_expr.clone(),
                        &[(Expr::Symbol(x.clone()), Expr::Number(xi as f64))],
                    )?)? {
                        Expr::Number(s) => s,
                        other => panic!("BUG: Non-Number should have exited earlier: {:?}", other),
                    };
                }
                Ok(Expr::Number(sum))
            }
            other => panic!("{} head not implemented", other),
        },
        // Nothing specific on atomic expressions
        _ => Ok(expr),
    }
}

pub fn eval_replace_all(expr: Expr) -> Result<Expr, String> {
    // Eval of replace_all is outside of eval because evaluation of expr is deferred
    match expr {
        Expr::Expr(head, args) if head == "ReplaceAll" => {
            let [expr, rules]: [Expr; 2] = args
                .try_into()
                .map_err(|e| format!("ReplaceAll must have 2 arguments. {:?}", e))?;
            evaluate(replace_all(
                // Nested evaluation of replace_all without eval of expr
                eval_replace_all(expr)?,
                eval_rules(rules)?.as_slice(),
            )?)
        }
        other => Ok(other),
    }
}

fn eval_rules(rules: Expr) -> Result<Vec<(Expr, Expr)>, String> {
    // Evaluate rules and check they result in Rule or List[Rule]
    match evaluate(rules)? {
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
