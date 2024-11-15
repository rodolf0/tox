use crate::findroot;
use crate::parser::Expr;

pub fn evaluate(expr: Expr) -> Result<Expr, String> {
    match expr {
        Expr::Expr(head, mut args) => match head.as_ref() {
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
            "ReplaceAll" => {
                let [expr, rules]: [Expr; 2] = args
                    .try_into()
                    .map_err(|e| format!("ReplaceAll must have 2 arguments. {:?}", e))?;
                replace_all(expr, eval_rules(rules)?.as_slice())
            }
            "Plus" => {
                let mut numeric: f64 = 0.0;
                let mut others = Vec::new();
                for a in args {
                    match a {
                        Expr::Number(n) => numeric += n,
                        other => {
                            let maybe_n = evaluate(other)?;
                            if let Expr::Number(n) = maybe_n {
                                numeric += n;
                            } else {
                                others.push(maybe_n);
                            }
                        }
                    }
                }
                others.push(Expr::Number(numeric));
                if others.len() == 1 {
                    Ok(others.swap_remove(0))
                } else {
                    Ok(Expr::Expr(head, others))
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
            "Times" => {
                let mut numeric: f64 = 1.0;
                let mut others = Vec::new();
                for a in args {
                    match a {
                        Expr::Number(n) => numeric *= n,
                        other => {
                            let maybe_n = evaluate(other)?;
                            if let Expr::Number(n) = maybe_n {
                                numeric *= n;
                            } else {
                                others.push(maybe_n);
                            }
                        }
                    }
                }
                others.push(Expr::Number(numeric));
                if others.len() == 1 {
                    Ok(others.swap_remove(0))
                } else {
                    Ok(Expr::Expr(head, others))
                }
            }
            "FindRoot" => {
                let Expr::Number(x0) = evaluate(args.swap_remove(1))? else {
                    return Err("FindRoot requires x0 to be a number".to_string());
                };
                let fexpr = evaluate(args.swap_remove(0))?;
                let f = |x: f64| match evaluate(replace_all(
                    fexpr.clone(),
                    &[(Expr::Symbol("x".to_string()), Expr::Number(x))],
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
            other => panic!("{} head not implemented", other),
        },
        // Nothing specific on atomic expressions
        _ => Ok(expr),
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
