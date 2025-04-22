use crate::context::Context;
use crate::expr::{Expr, evaluate};
use crate::parser::parser;

fn eval(expr: Expr) -> Result<Expr, String> {
    evaluate(expr, &mut Context::new())
}

#[test]
fn arith_ops() -> Result<(), String> {
    let p = parser()?;
    assert_eq!(eval(p(r#"1 + 2"#)?)?, Expr::Number(3.0));
    assert_eq!(eval(p(r#"1 + 2 - 3"#)?)?, Expr::Number(0.0));
    assert_eq!(eval(p(r#"1 - 2 + 3"#)?)?, Expr::Number(2.0));
    assert_eq!(eval(p(r#"1 + 2 * 3"#)?)?, Expr::Number(7.0));
    assert_eq!(eval(p(r#"2 ^ 2 ^ 3"#)?)?, Expr::Number(256.0));
    assert_eq!(eval(p(r#"1 + 2 ^ 3"#)?)?, Expr::Number(9.0));
    assert_eq!(eval(p(r#"3 / 2 / 4"#)?)?, Expr::Number(0.375));
    assert_eq!(eval(p(r#"-3"#)?)?, Expr::Number(-3.0));
    assert_eq!(eval(p(r#"--3"#)?)?, Expr::Number(3.0));
    assert_eq!(eval(p(r#"4--3"#)?)?, Expr::Number(7.0));
    Ok(())
}

#[test]
fn set_delayed() -> Result<(), String> {
    let mut ctx = Context::new();
    let p = parser()?;
    assert_eq!(evaluate(p(r#"x := 1"#)?, &mut ctx)?, Expr::Number(1.0));
    assert_eq!(
        evaluate(p(r#"f := x + 1"#)?, &mut ctx)?,
        Expr::Head(
            Box::new(Expr::Symbol("Plus".into())),
            vec![Expr::Symbol("x".to_string()), Expr::Number(1.0)]
        )
    );
    assert_eq!(evaluate(p(r#"g = x + 1"#)?, &mut ctx)?, Expr::Number(2.0));
    assert_eq!(evaluate(p(r#"f"#)?, &mut ctx)?, Expr::Number(2.0));
    Ok(())
}
