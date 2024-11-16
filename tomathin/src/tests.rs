use crate::expr::evaluate;
use crate::parser::{parser, Expr};

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
    let rep_1rule = p(r#"ReplaceAll[Hold[Plus[x, Times[2, x]]], Rule[x, 3]]"#)?;
    assert_eq!(
        evaluate(rep_1rule.clone())?,
        Expr::Expr(
            "Plus".to_string(),
            vec![
                Expr::Number(3.0),
                Expr::Expr(
                    "Times".to_string(),
                    vec![Expr::Number(2.0), Expr::Number(3.0)]
                )
            ]
        )
    );
    assert_eq!(evaluate(evaluate(rep_1rule)?)?, Expr::Number(9.0));
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
            vec![Expr::Symbol("x".to_string()), Expr::Number(3.0),]
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
    Ok(())
}
