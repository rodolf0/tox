use crate::Expr;

fn convert(t: crate::parser::T) -> crate::Expr {
    use crate::parser::T;
    use crate::Expr;
    match t {
        T::Expr(h, args) => {
            let mut cargs = Vec::new();
            for a in args {
                cargs.push(convert(a));
            }
            Expr::Expr(h, cargs)
        }
        T::Symbol(x) => Expr::Symbol(x),
        T::String(s) => Expr::String(s),
        T::Number(n) => Expr::Number(n),
        other => panic!("convert failed on '{:?}'", other),
    }
}

fn parse_expr(input: &str) -> Expr {
    let parser = crate::parser::parser().unwrap();
    let tok = crate::tokenizer::Tokenizer::new(input.chars());
    let mut parse_out = parser(tok).unwrap();
    assert_eq!(parse_out.len(), 1);
    convert(parse_out.remove(0))
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
        crate::evaluate(rep_1rule)?,
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
    assert_eq!(crate::evaluate(rep_rule_list)?, Expr::Number(4.0));

    // Test ReplaceAll with rule head replacement
    let rep_rule_head = parse_expr(
        r#"ReplaceAll[
            Plus[x, Times[2, x]],
            List[Rule[Times[2, x], 3], Rule[Plus, Times]]
        ]"#,
    );
    assert_eq!(
        crate::evaluate(rep_rule_head)?,
        Expr::Expr(
            "Times".to_string(),
            vec![Expr::Symbol("x".to_string()), Expr::Number(3.0),]
        )
    );

    Ok(())
}
