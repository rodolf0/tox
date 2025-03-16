use super::{eval_with_ctx, Expr};
use crate::context::Context;

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

pub fn eval_times(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
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
            .map(|lhsi| flatten(distribute_op(lhsi, rhs.clone(), "Times", "List"), "Times"))
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
        Ok(Expr::Expr("Times".to_string(), new_args2))
    }
}
