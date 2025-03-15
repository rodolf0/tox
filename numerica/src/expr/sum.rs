use super::replace_all::replace_all;
use super::{eval_with_ctx, Expr};
use crate::context::Context;

pub fn eval_sum(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
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
