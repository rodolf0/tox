use crate::expr::evaluate;
use crate::parser::Expr;

fn parse_expr(input: &str) -> Expr {
    let parser = crate::parser::parser().unwrap();
    let tok = crate::tokenizer::Tokenizer::new(input.chars());
    parser(tok).unwrap()
}

#[test]
fn basic_expr_parsing() {
    let parsed = parse_expr(r#"FindRoot[Sum[360, Sum[a, b]], List["1, 2, 3"]]"#);
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
        ],
    );
    assert_eq!(parsed, expected);
}

#[test]
fn replace_all() -> Result<(), String> {
    // Test ReplaceAll with single simple Rule
    let rep_1rule = parse_expr(r#"ReplaceAll[Plus[x, Times[2, x]], Rule[x, 3]]"#);
    assert_eq!(
        evaluate(rep_1rule)?,
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
    // Test ReplaceAll with a List[Rule]
    let rep_rule_list = parse_expr(
        r#"ReplaceAll[
            Plus[x, Times[2, x]],
            List[Rule[Times[2, x], 3], Rule[Plus[x, 3], 4]]
        ]"#,
    );
    assert_eq!(evaluate(rep_rule_list)?, Expr::Number(4.0));

    // Test ReplaceAll with rule head replacement
    let rep_rule_head = parse_expr(
        r#"ReplaceAll[
            Plus[x, Times[2, x]],
            List[Rule[Times[2, x], 3], Rule[Plus, Times]]
        ]"#,
    );
    assert_eq!(
        evaluate(rep_rule_head)?,
        Expr::Expr(
            "Times".to_string(),
            vec![Expr::Symbol("x".to_string()), Expr::Number(3.0),]
        )
    );

    Ok(())
}
