use core::fmt;
use rand_distr::Distribution;
use std::rc::Rc;

use crate::context::Context;
use crate::{find_root_vec, findroot};

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
            "FindRoot" => {
                // Evaluate and pull out list of functions to work on.
                let fexpr: Vec<Expr> = match args.remove(0) {
                    Expr::Expr(h, a) if h == "List" => a
                        .into_iter()
                        .map(|fi| eval_with_ctx(fi, ctx))
                        .collect::<Result<_, _>>()?,
                    expr => vec![eval_with_ctx(expr, ctx)?],
                };
                // Pull out variables specs to find roots for.
                let varspec: Vec<_> = match args.remove(0) {
                    Expr::Expr(h, a) if h == "List" => a,
                    other => return Err(format!("Unexpected var spec for FindRoot: {:?}", other)),
                };
                let vars: Vec<(Expr, f64, Option<(f64, f64)>)> = match varspec.as_slice() {
                    [v @ Expr::Symbol(_), Expr::Number(start)] => vec![(v.clone(), *start, None)],
                    [v @ Expr::Symbol(_), Expr::Number(start), Expr::Number(lo), Expr::Number(hi)] =>
                    {
                        vec![(v.clone(), *start, Some((*lo, *hi)))]
                    }
                    o if o
                        .iter()
                        .all(|s| matches!(s, Expr::Expr(h, _) if h == "List")) =>
                    {
                        o.into_iter()
                            .map(|s| {
                                let Expr::Expr(_, varspec) = s else {
                                    unreachable!();
                                };
                                match varspec.as_slice() {
                                    [v @ Expr::Symbol(_), Expr::Number(start)] => {
                                        (v.clone(), *start, None)
                                    }
                                    _ => todo!(),
                                }
                            })
                            .collect()
                    }
                    _ => todo!(),
                };

                if vars.len() != fexpr.len() {
                    return Err(format!(
                        "Need same number of functions as unknowns, {} vs {}",
                        vars.len(),
                        fexpr.len()
                    ));
                }

                if fexpr.len() == 1 {
                    let f = |xi: f64| match replace_all(
                        fexpr[0].clone(),
                        &[(vars[0].0.clone(), Expr::Number(xi))],
                    )
                    .and_then(|expr| evaluate(expr))
                    {
                        Ok(Expr::Number(x)) => Ok(x),
                        err => Err(format!("FindRoot didn't return Number: {:?}", err)),
                    };
                    let x0 = vars[0].1;
                    let roots = if let Some((lo, hi)) = vars[0].2 {
                        vec![findroot::regula_falsi(f, (lo, hi))?]
                    } else {
                        findroot::find_roots(f, x0)?
                    };
                    return Ok(Expr::Expr(
                        "List".to_string(),
                        roots.into_iter().map(|ri| Expr::Number(ri)).collect(),
                    ));
                }

                let funcs: Vec<_> = fexpr
                    .into_iter()
                    .map(|f| {
                        let vars = &vars;
                        move |xi: &Vec<f64>| match replace_all(
                            f.clone(),
                            &vars
                                .iter()
                                .zip(xi)
                                .map(|(var, val)| (var.0.clone(), Expr::Number(*val)))
                                .collect::<Vec<_>>(),
                        )
                        .and_then(|expr| evaluate(expr))
                        {
                            Ok(Expr::Number(x)) => Ok(x),
                            err => Err(format!("FindRoot didn't return Number: {:?}", err)),
                        }
                    })
                    .collect();

                let roots = find_root_vec(funcs, vars.iter().map(|vi| vi.1).collect())?;
                Ok(Expr::Expr(
                    "List".to_string(),
                    roots.into_iter().map(|ri| Expr::Number(ri)).collect(),
                ))
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
            "Table" => {
                // first arg is the expression that will be evaluated for each table element
                let expr = args.remove(0);
                // Figure out iteration dimensions
                let idxs: Vec<(Expr, f64, f64, f64)> = args
                    .into_iter()
                    .map(|spec| eval_with_ctx(spec, ctx))
                    .map(|spec| match spec {
                        Ok(Expr::Expr(h, spec)) if h == "List" => match spec.as_slice() {
                            [i, Expr::Number(imax)] => Ok((i.clone(), 1.0, *imax, 1.0)),
                            [i, Expr::Number(imin), Expr::Number(imax)] => {
                                Ok((i.clone(), *imin, *imax, 1.0))
                            }
                            [i, Expr::Number(imin), Expr::Number(imax), Expr::Number(di)] => {
                                Ok((i.clone(), *imin, *imax, *di))
                            }
                            other => Err(format!("Table spec not supported. {:?}", other)),
                        },
                        other => Err(format!("Table spec not supported. {:?}", other)),
                    })
                    .collect::<Result<_, _>>()?;
                // Function to step the cursor across all dimensions. Returns bumped dimension
                fn cursor_step(
                    spec: &[(Expr, f64, f64, f64)],
                    cursor: Option<Vec<f64>>,
                ) -> Option<(usize, Vec<f64>)> {
                    // Init cursor
                    let Some(mut cursor) = cursor else {
                        return Some((0, spec.iter().map(|spec| spec.1).collect()));
                    };
                    // Increment cursor values, rippling carries as needed
                    let mut idx = cursor.len() - 1;
                    loop {
                        cursor[idx] += spec[idx].3; // Add step size
                        if cursor[idx] <= spec[idx].2 {
                            return Some((idx, cursor)); // No carry needed
                        }
                        cursor[idx] = spec[idx].1; // Reset to min
                        if idx == 0 {
                            return None; // Done if we carried past first position
                        }
                        idx -= 1; // Move to next position
                    }
                }

                // Generate the table
                let mut cursor = None;
                let mut table = Expr::Expr("List".to_string(), Vec::new());

                while let Some((bumped_dim, c)) = cursor_step(&idxs, cursor) {
                    let mut inserter = match table {
                        Expr::Expr(_, ref mut a) => a,
                        _ => panic!(),
                    };
                    for dim in 0..idxs.len() - 1 {
                        if dim >= bumped_dim {
                            inserter.push(Expr::Expr("List".to_string(), Vec::new()));
                        }
                        inserter = match inserter.last_mut().unwrap() {
                            Expr::Expr(h, ref mut a) if h == "List" => a,
                            _ => panic!(),
                        };
                    }

                    let rexpr = replace_all(
                        expr.clone(),
                        &idxs
                            .iter()
                            .zip(&c)
                            .map(|(spec, ci)| (spec.0.clone(), Expr::Number(*ci)))
                            .collect::<Vec<_>>(),
                    )?;
                    inserter.push(eval_with_ctx(rexpr, ctx)?);
                    cursor = Some(c); // get next iteration
                }
                Ok(table)
            }
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
