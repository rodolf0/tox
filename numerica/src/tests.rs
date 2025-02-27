use crate::context::Context;
use crate::expr::{eval_with_ctx, evaluate, Expr};
use crate::parser::parser;

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
fn replace_all() -> Result<(), String> {
    let p = parser()?;
    // Test ReplaceAll with single simple Rule
    let expr1 = r#"ReplaceAll[Hold[Plus[x, Times[2, x]]], Rule[x, 3]]"#;
    let rep_1rule = p(expr1)?;
    assert_eq!(
        evaluate(rep_1rule.clone())?,
        Expr::Expr(
            "Hold".to_string(),
            vec![Expr::Expr(
                "Plus".to_string(),
                vec![
                    Expr::Number(3.0),
                    Expr::Expr(
                        "Times".to_string(),
                        vec![Expr::Number(2.0), Expr::Number(3.0)]
                    )
                ]
            )]
        )
    );
    assert_eq!(
        evaluate(p(&format!("Evaluate[{}]", expr1))?)?,
        Expr::Number(9.0)
    );
    // Test ReplaceAll with a List[Rule]
    let rep_rule_list = p(r#"ReplaceAll[
            Plus[x, Times[2, x]],
            List[Rule[Times[2, x], 3], Rule[Plus[x, 3], 4]]
        ]"#)?;
    assert_eq!(evaluate(rep_rule_list)?, Expr::Number(4.0));

    // Test ReplaceAll with rule head replacement
    let rep_rule_head = p(r#"ReplaceAll[
            Plus[x, Times[2, x]],
            List[Rule[Times[2, x], 3], Rule[Plus, Times]]
        ]"#)?;
    assert_eq!(
        evaluate(rep_rule_head)?,
        Expr::Expr(
            "Times".to_string(),
            vec![Expr::Number(3.0), Expr::Symbol("x".to_string())]
        )
    );
    Ok(())
}

#[test]
fn rule_associativity() -> Result<(), String> {
    let p = parser()?;
    assert_eq!(
        evaluate(p(r#"x /. x -> z -> 3"#)?)?,
        Expr::Expr(
            "Rule".to_string(),
            vec![Expr::Symbol("z".to_string()), Expr::Number(3.0)]
        )
    );
    assert_eq!(
        evaluate(p(r#"x /. z -> x -> 3"#)?)?,
        Expr::Symbol("x".to_string())
    );
    Ok(())
}

#[test]
fn replace_ops() -> Result<(), String> {
    let p = parser()?;
    assert_eq!(
        evaluate(p(r#"x + y /. x -> 2"#)?)?,
        Expr::Expr(
            "Plus".to_string(),
            vec![Expr::Number(2.0), Expr::Symbol("y".to_string())]
        )
    );
    assert_eq!(
        evaluate(p(r#"x + y /. x -> z /. z -> 3"#)?)?,
        Expr::Expr(
            "Plus".to_string(),
            vec![Expr::Number(3.0), Expr::Symbol("y".to_string())]
        )
    );
    assert_eq!(
        evaluate(p(r#"x + y /. z -> 3 /. x -> z"#)?)?,
        Expr::Expr(
            "Plus".to_string(),
            vec![Expr::Symbol("z".to_string()), Expr::Symbol("y".to_string())]
        )
    );
    Ok(())
}

#[test]
fn arith_ops() -> Result<(), String> {
    let p = parser()?;
    assert_eq!(evaluate(p(r#"1 + 2"#)?)?, Expr::Number(3.0));
    assert_eq!(evaluate(p(r#"1 + 2 - 3"#)?)?, Expr::Number(0.0));
    assert_eq!(evaluate(p(r#"1 - 2 + 3"#)?)?, Expr::Number(2.0));
    assert_eq!(evaluate(p(r#"1 + 2 * 3"#)?)?, Expr::Number(7.0));
    assert_eq!(evaluate(p(r#"2 ^ 2 ^ 3"#)?)?, Expr::Number(256.0));
    assert_eq!(evaluate(p(r#"1 + 2 ^ 3"#)?)?, Expr::Number(9.0));
    assert_eq!(evaluate(p(r#"3 / 2 / 4"#)?)?, Expr::Number(0.375));
    assert_eq!(evaluate(p(r#"-3"#)?)?, Expr::Number(-3.0));
    assert_eq!(evaluate(p(r#"--3"#)?)?, Expr::Number(3.0));
    assert_eq!(evaluate(p(r#"4--3"#)?)?, Expr::Number(7.0));
    Ok(())
}

#[test]
fn sum_expr() -> Result<(), String> {
    let p = parser()?;
    assert_eq!(evaluate(p(r#"Sum[x^2, {x, 3}]"#)?)?, Expr::Number(14.0));
    assert_eq!(evaluate(p(r#"Sum[x^2, {x, 2, 4}]"#)?)?, Expr::Number(29.0));
    assert_eq!(
        evaluate(p(r#"Sum[x^i, {i, 4}]"#)?)?,
        Expr::Expr(
            "Sum".to_string(),
            vec![
                Expr::Expr(
                    "Power".to_string(),
                    vec![Expr::Symbol("x".to_string()), Expr::Symbol("i".to_string()),]
                ),
                Expr::Expr(
                    "List".to_string(),
                    vec![Expr::Symbol("i".to_string()), Expr::Number(4.0)]
                )
            ]
        )
    );
    assert_eq!(
        evaluate(p(r#"ReplaceAll[Sum[x^i, {i, 4}], x -> 2]"#)?)?,
        Expr::Number(1.0 + 2.0 + 4.0 + 8.0 + 16.0)
    );
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

#[test]
fn table() -> Result<(), String> {
    let p = parser()?;
    assert_eq!(
        evaluate(p(r#"Table[i, {i, 3}]"#)?)?,
        Expr::Expr(
            "List".to_string(),
            vec![Expr::Number(1.0), Expr::Number(2.0), Expr::Number(3.0),]
        )
    );
    assert_eq!(
        evaluate(p(r#"Table[i+j, {i, 2}, {j, 3}]"#)?)?,
        Expr::Expr(
            "List".to_string(),
            vec![
                Expr::Expr(
                    "List".to_string(),
                    vec![Expr::Number(2.0), Expr::Number(3.0), Expr::Number(4.0)]
                ),
                Expr::Expr(
                    "List".to_string(),
                    vec![Expr::Number(3.0), Expr::Number(4.0), Expr::Number(5.0)]
                ),
            ]
        )
    );
    assert_eq!(
        evaluate(p(r#"Table[i+j+k, {i, 2}, {j, 2+1}, {k, 2}]"#)?)?,
        Expr::Expr(
            "List".to_string(),
            vec![
                Expr::Expr(
                    "List".to_string(),
                    vec![
                        Expr::Expr(
                            "List".to_string(),
                            vec![Expr::Number(3.0), Expr::Number(4.0)]
                        ),
                        Expr::Expr(
                            "List".to_string(),
                            vec![Expr::Number(4.0), Expr::Number(5.0)]
                        ),
                        Expr::Expr(
                            "List".to_string(),
                            vec![Expr::Number(5.0), Expr::Number(6.0)]
                        ),
                    ]
                ),
                Expr::Expr(
                    "List".to_string(),
                    vec![
                        Expr::Expr(
                            "List".to_string(),
                            vec![Expr::Number(4.0), Expr::Number(5.0)]
                        ),
                        Expr::Expr(
                            "List".to_string(),
                            vec![Expr::Number(5.0), Expr::Number(6.0)]
                        ),
                        Expr::Expr(
                            "List".to_string(),
                            vec![Expr::Number(6.0), Expr::Number(7.0)]
                        ),
                    ]
                ),
            ]
        )
    );
    Ok(())
}
