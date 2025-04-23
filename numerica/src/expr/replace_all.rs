use super::Expr;

pub(crate) fn eval_replace_all(args: Vec<Expr>) -> Result<Expr, String> {
    let [expr, rules]: [Expr; 2] = args
        .try_into()
        .map_err(|e| format!("ReplaceAll must have 2 arguments. {:?}", e))?;
    replace_all(expr, unpack_rules(rules)?.as_slice())
}

// ReplaceAll[x, Rule[x, 3]]
// ReplaceAll[List[1, 2, 3], Rule[List, FindRoot]]
pub(crate) fn replace_all(expr: Expr, rules: &[(Expr, Expr)]) -> Result<Expr, String> {
    match expr {
        Expr::Head(head, args) => {
            // Recursively apply replacement for arguments
            let replaced_expr = Expr::Head(
                head,
                args.into_iter()
                    .map(|a| replace_all(a, rules))
                    .collect::<Result<_, _>>()?,
            );
            // Check if update expression matches a replacement too.
            // TODO: this needs to run in a loop until it the result doesn't change
            let replaced_expr = rules
                .iter()
                .filter(|(lhs, _)| replaced_expr == *lhs)
                .next()
                .map(|(_, rhs)| rhs.clone())
                .unwrap_or(replaced_expr);
            // Check if a rule is a head re-write
            Ok(match replaced_expr {
                Expr::Head(head, args) if matches!(*head, Expr::Symbol(_)) => {
                    let Expr::Symbol(head) = *head else {
                        unreachable!()
                    };
                    let mut replaced_expr =
                        Expr::Head(Box::new(Expr::Symbol(head.clone())), args.clone());
                    for r in rules {
                        // Resolved heads will be Symbols, so filter LHS to just Symbols
                        if let (Expr::Symbol(lhs), rhs) = r {
                            if *lhs == head {
                                replaced_expr = Expr::Head(Box::new(rhs.clone()), args.clone());
                            }
                        }
                    }
                    replaced_expr
                }
                // Leave head alone
                expr => expr,
            })
        }
        atom => Ok(rules
            .iter()
            .filter(|(lhs, _)| atom == *lhs)
            .next()
            .map(|(_, rhs)| rhs.clone())
            .unwrap_or(atom)),
    }
}

fn unpack_rules(rules: Expr) -> Result<Vec<(Expr, Expr)>, String> {
    // Check they result in Rule or List[Rule]
    match rules {
        Expr::Head(head, args) if *head == Expr::Symbol("Rule".into()) => {
            let [lhs, rhs]: [Expr; 2] = args
                .try_into()
                .map_err(|e| format!("Rule must have 2 arguments. {:?}", e))?;
            Ok(vec![(lhs, rhs)])
        }
        Expr::Head(h, args) if *h == Expr::Symbol("List".into()) => args
            .into_iter()
            .map(|rule| match rule {
                Expr::Head(head, args) if *head == Expr::Symbol("Rule".into()) => {
                    let [lhs, rhs]: [Expr; 2] = args
                        .try_into()
                        .map_err(|e| format!("Rule must have 2 arguments. {:?}", e))?;
                    Ok((lhs, rhs))
                }
                other => Err(format!("Expected all args to be Rule: {:?}", other)),
            })
            .collect(),
        other => Err(format!("ReplaceAll unexpected arg: '{:?}'", other)),
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::Expr;

    fn eval(expr: &str) -> Result<Expr, String> {
        use crate::context::Context;
        use crate::expr::evaluate;
        use crate::parser::parser;
        evaluate(parser()?(expr)?, &mut Context::new())
    }

    #[test]
    fn single_rule() -> Result<(), String> {
        use crate::expr::Expr::*;
        // ReplaceAll[Hold[Plus[x, Times[2, x]]], Rule[x, 3]]
        let expr = vec![
            Head(
                Box::new(Symbol("Hold".into())),
                vec![Head(
                    Box::new(Symbol("Plus".into())),
                    vec![
                        Symbol("x".into()),
                        Head(
                            Box::new(Symbol("Times".into())),
                            vec![Number(2.0), Symbol("x".into())],
                        ),
                    ],
                )],
            ),
            Head(
                Box::new(Symbol("Rule".into())),
                vec![Symbol("x".into()), Number(3.0)],
            ),
        ];
        assert_eq!(
            super::eval_replace_all(expr)?,
            Head(
                Box::new(Symbol("Hold".into())),
                vec![Head(
                    Box::new(Symbol("Plus".into())),
                    vec![
                        Number(3.0),
                        Head(
                            Box::new(Symbol("Times".into())),
                            vec![Number(2.0), Number(3.0)]
                        )
                    ]
                )]
            )
        );
        Ok(())
    }

    #[test]
    fn multiple_rules() -> Result<(), String> {
        // Test ReplaceAll with a List[Rule]
        let r = eval(
            r#"ReplaceAll[
                Plus[x, Times[2, x]],
                List[Rule[Times[2, x], 3], Rule[Plus[x, 3], 4]]
            ]"#,
        )?;
        assert_eq!(r, Expr::Number(4.0));
        Ok(())
    }

    #[test]
    fn head_replacement() -> Result<(), String> {
        // Test ReplaceAll with rule head replacement
        let r = eval(
            r#"ReplaceAll[
                Plus[x, Times[2, x]],
                List[Rule[Times[2, x], 3], Rule[Plus, Times]]
            ]"#,
        )?;
        assert_eq!(
            r,
            Expr::Head(
                Box::new(Expr::Symbol("Times".into())),
                vec![Expr::Number(3.0), Expr::Symbol("x".into())]
            )
        );
        Ok(())
    }

    #[test]
    fn nested_replace() -> Result<(), String> {
        let r = eval(
            r#"
            ReplaceAll[
                Plus[x,
                  Times[2, ReplaceAll[Times[z, y], Rule[Times[z, y], x]]]
                ],
                Rule[Times[2, x], 3]
            ]"#,
        )?;
        assert_eq!(
            r,
            Expr::Head(
                Box::new(Expr::Symbol("Plus".into())),
                vec![Expr::Number(3.0), Expr::Symbol("x".into())]
            )
        );
        Ok(())
    }

    #[test]
    fn rule_associativity() -> Result<(), String> {
        assert_eq!(
            eval(r#"x /. x -> z -> 3"#)?,
            Expr::Head(
                Box::new(Expr::Symbol("Rule".into())),
                vec![Expr::Symbol("z".into()), Expr::Number(3.0)]
            )
        );
        assert_eq!(eval(r#"x /. z -> x -> 3"#)?, Expr::Symbol("x".into()));
        Ok(())
    }

    #[test]
    fn replace_ops() -> Result<(), String> {
        assert_eq!(
            eval(r#"x + y /. x -> 2"#)?,
            Expr::Head(
                Box::new(Expr::Symbol("Plus".into())),
                vec![Expr::Number(2.0), Expr::Symbol("y".into())]
            )
        );
        assert_eq!(
            eval(r#"x + y /. x -> z /. z -> 3"#)?,
            Expr::Head(
                Box::new(Expr::Symbol("Plus".into())),
                vec![Expr::Number(3.0), Expr::Symbol("y".into())]
            )
        );
        assert_eq!(
            eval(r#"x + y /. z -> 3 /. x -> z"#)?,
            Expr::Head(
                Box::new(Expr::Symbol("Plus".into())),
                vec![Expr::Symbol("z".into()), Expr::Symbol("y".into())]
            )
        );
        Ok(())
    }
}
