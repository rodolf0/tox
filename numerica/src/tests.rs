use crate::context::Context;
use crate::expr::{Expr, eval_with_ctx};
use crate::parser::parser;

fn eval(expr: Expr) -> Result<Expr, String> {
    eval_with_ctx(expr, &mut Context::new())
}

#[test]
fn parse_basic_expr() -> Result<(), String> {
    let input = r#"FindRoot[Sum[360, Sum[a, b]], List["1, 2, 3"], {x, 2}]"#;
    let expected = Expr::Expr(
        "FindRoot".to_string(),
        vec![
            Expr::Expr(
                "Sum".to_string(),
                vec![
                    Expr::Number(360.0),
                    Expr::Expr(
                        "Sum".to_string(),
                        vec![Expr::Symbol("a".to_string()), Expr::Symbol("b".to_string())],
                    ),
                ],
            ),
            Expr::Expr(
                "List".to_string(),
                vec![Expr::String("1, 2, 3".to_string())],
            ),
            Expr::Expr(
                "List".to_string(),
                vec![Expr::Symbol("x".to_string()), Expr::Number(2.0)],
            ),
        ],
    );
    assert_eq!(parser()?(input)?, expected);
    Ok(())
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
    assert_eq!(eval_with_ctx(p(r#"x := 1"#)?, &mut ctx)?, Expr::Number(1.0));
    assert_eq!(
        eval_with_ctx(p(r#"f := x + 1"#)?, &mut ctx)?,
        Expr::Expr(
            "Plus".to_string(),
            vec![Expr::Symbol("x".to_string()), Expr::Number(1.0)]
        )
    );
    assert_eq!(
        eval_with_ctx(p(r#"g = x + 1"#)?, &mut ctx)?,
        Expr::Number(2.0)
    );
    assert_eq!(eval_with_ctx(p(r#"f"#)?, &mut ctx)?, Expr::Number(2.0));
    Ok(())
}
