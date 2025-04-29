use crate::context::Context;
use crate::expr::Expr;

fn eval(expr: &str) -> Result<Expr, String> {
    eval_ctx(expr, &mut Context::new())
}

fn eval_ctx(expr: &str, ctx: &mut Context) -> Result<Expr, String> {
    use crate::expr::evaluate;
    use crate::parser::parser;
    evaluate(parser()?(expr)?, ctx)
}

#[test]
fn set_delayed() -> Result<(), String> {
    let mut ctx = Context::new();
    assert_eq!(eval_ctx(r#"x := 1"#, &mut ctx)?, Expr::Number(1.0));
    assert_eq!(
        eval_ctx(r#"f := x + 1"#, &mut ctx)?,
        Expr::Head(
            Box::new(Expr::Symbol("Plus".into())),
            vec![Expr::Symbol("x".into()), Expr::Number(1.0)]
        )
    );
    assert_eq!(eval_ctx(r#"g = x + 1"#, &mut ctx)?, Expr::Number(2.0));
    assert_eq!(eval_ctx(r#"f"#, &mut ctx)?, Expr::Number(2.0));
    Ok(())
}

#[test]
fn composite_expr() -> Result<(), String> {
    assert_eq!(
        eval(r#"ReplaceAll[Times, Rule[Times, Plus]][3, 4]"#)?,
        Expr::Number(7.0)
    );
    Ok(())
}

#[test]
fn empty_arglist() -> Result<(), String> {
    let rand_num = eval(r#"NormalDist[0, 2][]"#)?;
    assert!(matches!(rand_num, Expr::Number(_)));
    Ok(())
}

#[test]
fn functions() -> Result<(), String> {
    assert_eq!(eval(r#"Function[x, x + 1][3]"#)?, Expr::Number(4.0));

    // Bind a function to a variable and call it
    let mut ctx = Context::new();
    assert_eq!(
        eval_ctx(r#"f = Function[y, y * 2]"#, &mut ctx)?,
        Expr::Function(
            vec!["y".into()],
            Box::new(Expr::from_head(
                "Times",
                vec![Expr::Symbol("y".into()), Expr::Number(2.0)]
            ))
        )
    );
    assert_eq!(eval_ctx(r#"f[7]"#, &mut ctx)?, Expr::Number(14.0));
    Ok(())
}
