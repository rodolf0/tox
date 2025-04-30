use crate::expr::Expr;

fn fold_bin_op_commutative(
    args: Vec<Expr>,
    reduce_op: fn(f64, f64) -> f64,
    lassoc: bool,
) -> (Option<f64>, Option<Vec<Expr>>) {
    let reduce = |acc: (Option<f64>, Option<Vec<Expr>>), x| {
        let (num_acc, other_acc) = acc;
        match x {
            Expr::Number(n) => match num_acc {
                None => (Some(n), other_acc),
                Some(lhs) => (Some(reduce_op(lhs, n)), other_acc),
            },
            other => match other_acc {
                None => (num_acc, Some(vec![other])),
                Some(lhs) => (
                    num_acc,
                    Some(lhs.into_iter().chain(std::iter::once(other)).collect()),
                ),
            },
        }
    };
    if lassoc {
        args.into_iter().fold((None, None), reduce)
    } else {
        args.into_iter().rfold((None, None), reduce)
    }
}

fn eval_commutative_binop(
    args: Vec<Expr>,
    head_name: &str,
    reduce_op: fn(f64, f64) -> f64,
    lassoc: bool,
) -> Result<Expr, String> {
    match fold_bin_op_commutative(args, reduce_op, lassoc) {
        (None, None) => Err(format!("Missing arguments for {}", head_name)),
        (Some(n), None) => Ok(Expr::Number(n)),
        (None, Some(exprs)) => Ok(Expr::from_head(head_name, exprs)),
        (Some(num), Some(exprs)) => Ok(Expr::from_head(
            head_name,
            std::iter::once(Expr::Number(num))
                .chain(exprs.into_iter())
                .collect(),
        )),
    }
}

fn eval_non_commutative_binop(
    args: Vec<Expr>,
    head_name: &str,
    reduce_op: fn(f64, f64) -> f64,
) -> Result<Expr, String> {
    let [lhs, rhs]: [Expr; 2] = args
        .try_into()
        .map_err(|e| format!("{} must have 2 arguments. {:?}", head_name, e))?;
    match (lhs, rhs) {
        (Expr::Number(lhs), Expr::Number(rhs)) => Ok(Expr::Number(reduce_op(lhs, rhs))),
        (lhs, rhs) => Ok(Expr::from_head(head_name, vec![lhs, rhs])),
    }
}

pub(crate) fn eval_plus(args: Vec<Expr>) -> Result<Expr, String> {
    eval_commutative_binop(args, "Plus", |acc, x| acc + x, true)
}

pub(crate) fn eval_minus(args: Vec<Expr>) -> Result<Expr, String> {
    eval_non_commutative_binop(args, "Minus", |l, r| l - r)
}

pub(crate) fn eval_divide(args: Vec<Expr>) -> Result<Expr, String> {
    eval_non_commutative_binop(args, "Divide", |l, r| l / r)
}

pub(crate) fn eval_times(args: Vec<Expr>) -> Result<Expr, String> {
    eval_commutative_binop(args, "Times", |acc, x| acc * x, true)
}

pub(crate) fn eval_power(args: Vec<Expr>) -> Result<Expr, String> {
    eval_non_commutative_binop(args, "Power", |l, r| l.powf(r))
}

#[cfg(test)]
mod tests {
    use crate::context::Context;
    use crate::expr::Expr;

    fn eval(expr: &str) -> Result<Expr, String> {
        use crate::expr::evaluate;
        use crate::parser::parser;
        evaluate(parser()?(expr)?, &mut Context::new())
    }

    #[test]
    fn arith_ops() -> Result<(), String> {
        assert_eq!(eval(r#"1 + 2"#)?, Expr::Number(3.0));
        assert_eq!(eval(r#"1 + 2 - 3"#)?, Expr::Number(0.0));
        assert_eq!(eval(r#"1 - 2 + 3"#)?, Expr::Number(2.0));
        assert_eq!(eval(r#"1 - 2 - 3"#)?, Expr::Number(-4.0));
        assert_eq!(eval(r#"1 + 2 * 3"#)?, Expr::Number(7.0));
        assert_eq!(eval(r#"2 ^ 2 ^ 3"#)?, Expr::Number(256.0));
        assert_eq!(eval(r#"1 + 2 ^ 3"#)?, Expr::Number(9.0));
        assert_eq!(eval(r#"3 / 2 / 4"#)?, Expr::Number(0.375));
        assert_eq!(eval(r#"-3"#)?, Expr::Number(-3.0));
        assert_eq!(eval(r#"--3"#)?, Expr::Number(3.0));
        assert_eq!(eval(r#"4--3"#)?, Expr::Number(7.0));
        assert_eq!(eval(r#"-4--3"#)?, Expr::Number(-1.0));
        assert_eq!(eval(r#"-4-3"#)?, Expr::Number(-7.0));
        Ok(())
    }

    #[test]
    fn arith_ops_commutative() -> Result<(), String> {
        // Check there's no re-ordering leading to a different expression
        assert_eq!(
            eval(r#"x ^ 2"#)?,
            Expr::from_head(
                "Power",
                vec![Expr::Symbol("x".to_string()), Expr::Number(2.0)]
            )
        );
        Ok(())
    }
}
