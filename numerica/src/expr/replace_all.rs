use super::{Expr, eval_with_ctx};
use crate::context::Context;

pub fn eval_replace_all(expr: Expr, ctx: &mut Context) -> Result<Expr, String> {
    // Eval of replace_all is outside of eval because evaluation of expr is deferred
    match expr {
        Expr::Expr(head, args) if head == "ReplaceAll" => {
            let [expr, rules]: [Expr; 2] = args
                .try_into()
                .map_err(|e| format!("ReplaceAll must have 2 arguments. {:?}", e))?;
            eval_with_ctx(
                replace_all(
                    // Nested evaluation of replace_all without eval of expr
                    eval_replace_all(expr, ctx)?,
                    eval_rules(rules, ctx)?.as_slice(),
                )?,
                ctx,
            )
        }
        other => Ok(other),
    }
}

// ReplaceAll[x, Rule[x, 3]]
// ReplaceAll[List[1, 2, 3], Rule[List, FindRoot]]
pub fn replace_all(expr: Expr, rules: &[(Expr, Expr)]) -> Result<Expr, String> {
    match expr {
        // for each sub-expression apply the replacement
        Expr::Expr(head, args) => {
            // First execute replace_all on subexpressions.
            let replaced_expr = Expr::Expr(
                head,
                args.into_iter()
                    .map(|a| replace_all(a, rules))
                    .collect::<Result<_, _>>()?,
            );
            // Check if update expression matches a replacement too.
            let replaced_expr = rules
                .iter()
                .filter(|(lhs, _)| replaced_expr == *lhs)
                .next()
                .map(|(_, rhs)| rhs.clone())
                .unwrap_or(replaced_expr);
            // Check if a rule is a head re-write
            Ok(match replaced_expr {
                Expr::Expr(head, args) => {
                    for r in rules {
                        if let (Expr::Symbol(lhs), Expr::Symbol(rhs)) = r {
                            if *lhs == head {
                                return Ok(Expr::Expr(rhs.clone(), args));
                            }
                        }
                    }
                    Expr::Expr(head, args)
                }
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

fn eval_rules(rules: Expr, ctx: &mut Context) -> Result<Vec<(Expr, Expr)>, String> {
    // eval_with_ctx rules and check they result in Rule or List[Rule]
    match eval_with_ctx(rules, ctx)? {
        Expr::Expr(head, args) if head == "Rule" => {
            let [lhs, rhs]: [Expr; 2] = args
                .try_into()
                .map_err(|e| format!("Rule must have 2 arguments. {:?}", e))?;
            Ok(vec![(lhs, rhs)])
        }
        Expr::Expr(h, args) if h == "List" => args
            .into_iter()
            .map(|rule| match rule {
                Expr::Expr(head, args) if head == "Rule" => {
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
    use crate::context::Context;
    use crate::expr::{Expr, eval_with_ctx};
    use crate::parser::parser;

    fn eval(expr: Expr) -> Result<Expr, String> {
        eval_with_ctx(expr, &mut Context::new())
    }

    #[test]
    fn replace_all() -> Result<(), String> {
        let p = parser()?;
        // Test ReplaceAll with single simple Rule
        let expr1 = r#"ReplaceAll[Hold[Plus[x, Times[2, x]]], Rule[x, 3]]"#;
        let rep_1rule = p(expr1)?;
        assert_eq!(
            eval(rep_1rule.clone())?,
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
            eval(p(&format!("Evaluate[{}]", expr1))?)?,
            Expr::Number(9.0)
        );
        // Test ReplaceAll with a List[Rule]
        let rep_rule_list = p(r#"ReplaceAll[
                Plus[x, Times[2, x]],
                List[Rule[Times[2, x], 3], Rule[Plus[x, 3], 4]]
            ]"#)?;
        assert_eq!(eval(rep_rule_list)?, Expr::Number(4.0));

        // Test ReplaceAll with rule head replacement
        let rep_rule_head = p(r#"ReplaceAll[
                Plus[x, Times[2, x]],
                List[Rule[Times[2, x], 3], Rule[Plus, Times]]
            ]"#)?;
        assert_eq!(
            eval(rep_rule_head)?,
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
            eval(p(r#"x /. x -> z -> 3"#)?)?,
            Expr::Expr(
                "Rule".to_string(),
                vec![Expr::Symbol("z".to_string()), Expr::Number(3.0)]
            )
        );
        assert_eq!(
            eval(p(r#"x /. z -> x -> 3"#)?)?,
            Expr::Symbol("x".to_string())
        );
        Ok(())
    }

    #[test]
    fn replace_ops() -> Result<(), String> {
        let p = parser()?;
        assert_eq!(
            eval(p(r#"x + y /. x -> 2"#)?)?,
            Expr::Expr(
                "Plus".to_string(),
                vec![Expr::Number(2.0), Expr::Symbol("y".to_string())]
            )
        );
        assert_eq!(
            eval(p(r#"x + y /. x -> z /. z -> 3"#)?)?,
            Expr::Expr(
                "Plus".to_string(),
                vec![Expr::Number(3.0), Expr::Symbol("y".to_string())]
            )
        );
        assert_eq!(
            eval(p(r#"x + y /. z -> 3 /. x -> z"#)?)?,
            Expr::Expr(
                "Plus".to_string(),
                vec![Expr::Symbol("z".to_string()), Expr::Symbol("y".to_string())]
            )
        );
        Ok(())
    }
}
