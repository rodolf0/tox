use crate::context::Context;
use crate::expr::Expr;

// Parse iteration arguments, Eg: {var, 0, end} or {var, start, end}
fn parse_sum_args(sum_args: &Expr) -> Result<(String, i32, i32), String> {
    use super::Expr::*;
    match sum_args {
        Head(h, args) if **h == Symbol("List".into()) => match args.as_slice() {
            [Symbol(x), Number(xn)] => Ok((x.clone(), 0 as i32, *xn as i32)),
            [Symbol(x), Number(x0), Number(xn)] => Ok((x.clone(), *x0 as i32, *xn as i32)),
            other => Err(format!("Sum unexpected arg1: {:?}", other)),
        },
        other => Err(format!("Sum unexpected arg1: {:?}", other)),
    }
}

pub(crate) fn eval_sum(args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    let [sum_expr, sum_args]: [Expr; 2] = args
        .try_into()
        .map_err(|e| format!("Sum expected: {{expr, args}}. Got {:?}", e))?;
    let (x, x0, xn) = parse_sum_args(&sum_args)?;
    // Build sum function out of sum expression and iteration variable
    let sum_func = super::evaluate(
        Expr::from_head("Function", vec![Expr::Symbol(x), sum_expr.clone()]),
        ctx,
    )?;
    let sum = (x0..=xn).try_fold(0.0, |sum, xi| {
        match super::apply(sum_func.clone(), vec![Expr::Number(xi as f64)], ctx) {
            Ok(Expr::Number(n)) => Ok(sum + n),
            Ok(other) => Err(Ok(other)), // Short-circuit eval but no failure
            Err(err) => Err(Err(err)),
        }
    });
    match sum {
        Ok(sum) => Ok(Expr::Number(sum)),
        Err(Ok(_)) => Ok(Expr::from_head("Sum", vec![sum_expr, sum_args])),
        Err(Err(e)) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use crate::context::Context;
    use crate::expr::{Expr, evaluate};
    use crate::parser::parser;

    fn eval(expr: Expr) -> Result<Expr, String> {
        evaluate(expr, &mut Context::new())
    }

    #[test]
    fn sum_expr() -> Result<(), String> {
        let p = parser()?;
        assert_eq!(eval(p(r#"Sum[x^2, {x, 3}]"#)?)?, Expr::Number(14.0));
        assert_eq!(eval(p(r#"Sum[x^2, {x, 2, 4}]"#)?)?, Expr::Number(29.0));
        assert_eq!(
            eval(p(r#"Sum[x^i, {i, 4}]"#)?)?,
            Expr::from_head(
                "Sum",
                vec![
                    Expr::from_head(
                        "Power",
                        vec![Expr::Symbol("x".into()), Expr::Symbol("i".into())]
                    ),
                    Expr::from_head("List", vec![Expr::Symbol("i".into()), Expr::Number(4.0)])
                ]
            )
        );
        assert_eq!(
            eval(p(r#"ReplaceAll[Sum[x^i, {i, 4}], x -> 2]"#)?)?,
            Expr::Number(1.0 + 2.0 + 4.0 + 8.0 + 16.0)
        );
        Ok(())
    }
}
