use super::replace_all::replace_all;
use super::{Expr, evaluate};
use crate::context::Context;

pub fn eval_table(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    // first arg is the expression that will be evaluated for each table element
    let expr = args.remove(0);
    // Figure out iteration dimensions
    let idxs: Vec<(Expr, f64, f64, f64)> = args
        .into_iter()
        .map(|spec| evaluate(spec, ctx))
        .map(|spec| match spec {
            Ok(Expr::Head(h, spec)) if *h == Expr::Symbol("List".into()) => match spec.as_slice() {
                [i, Expr::Number(imax)] => Ok((i.clone(), 1.0, *imax, 1.0)),
                [i, Expr::Number(imin), Expr::Number(imax)] => Ok((i.clone(), *imin, *imax, 1.0)),
                [i, Expr::Number(imin), Expr::Number(imax), Expr::Number(di)] => {
                    Ok((i.clone(), *imin, *imax, *di))
                }
                other => Err(format!("Table spec not supported. {:?}", other)),
            },
            other => Err(format!("Table spec not supported. {:?}", other)),
        })
        .collect::<Result<_, _>>()?;
    // Function to step the cursor across all dimensions. Returns bumped dimension
    fn cursor_step(
        spec: &[(Expr, f64, f64, f64)],
        cursor: Option<Vec<f64>>,
    ) -> Option<(usize, Vec<f64>)> {
        // Init cursor
        let Some(mut cursor) = cursor else {
            return Some((0, spec.iter().map(|spec| spec.1).collect()));
        };
        // Increment cursor values, rippling carries as needed
        let mut idx = cursor.len() - 1;
        loop {
            cursor[idx] += spec[idx].3; // Add step size
            if cursor[idx] <= spec[idx].2 {
                return Some((idx, cursor)); // No carry needed
            }
            cursor[idx] = spec[idx].1; // Reset to min
            if idx == 0 {
                return None; // Done if we carried past first position
            }
            idx -= 1; // Move to next position
        }
    }

    // Generate the table
    let mut cursor = None;
    let mut table = Expr::from_head("List", Vec::new());

    while let Some((bumped_dim, c)) = cursor_step(&idxs, cursor) {
        let mut inserter = match table {
            Expr::Head(_, ref mut a) => a,
            _ => panic!(),
        };
        for dim in 0..idxs.len() - 1 {
            if dim >= bumped_dim {
                inserter.push(Expr::from_head("List", Vec::new()));
            }
            inserter = match inserter.last_mut().unwrap() {
                Expr::Head(h, a) if **h == Expr::Symbol("List".into()) => a,
                _ => panic!(),
            };
        }

        let rexpr = replace_all(
            expr.clone(),
            &idxs
                .iter()
                .zip(&c)
                .map(|(spec, ci)| (spec.0.clone(), Expr::Number(*ci)))
                .collect::<Vec<_>>(),
        )?;
        inserter.push(evaluate(rexpr, ctx)?);
        cursor = Some(c); // get next iteration
    }
    Ok(table)
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
    fn table() -> Result<(), String> {
        assert_eq!(
            eval(r#"Table[i, {i, 3}]"#)?,
            Expr::from_head(
                "List",
                vec![Expr::Number(1.0), Expr::Number(2.0), Expr::Number(3.0),]
            )
        );
        assert_eq!(
            eval(r#"Table[i+j, {i, 2}, {j, 3}]"#)?,
            Expr::from_head(
                "List",
                vec![
                    Expr::from_head(
                        "List",
                        vec![Expr::Number(2.0), Expr::Number(3.0), Expr::Number(4.0)]
                    ),
                    Expr::from_head(
                        "List",
                        vec![Expr::Number(3.0), Expr::Number(4.0), Expr::Number(5.0)]
                    ),
                ]
            )
        );
        assert_eq!(
            eval(r#"Table[i+j+k, {i, 2}, {j, 2+1}, {k, 2}]"#)?,
            Expr::from_head(
                "List",
                vec![
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head("List", vec![Expr::Number(3.0), Expr::Number(4.0)]),
                            Expr::from_head("List", vec![Expr::Number(4.0), Expr::Number(5.0)]),
                            Expr::from_head("List", vec![Expr::Number(5.0), Expr::Number(6.0)]),
                        ]
                    ),
                    Expr::from_head(
                        "List",
                        vec![
                            Expr::from_head("List", vec![Expr::Number(4.0), Expr::Number(5.0)]),
                            Expr::from_head("List", vec![Expr::Number(5.0), Expr::Number(6.0)]),
                            Expr::from_head("List", vec![Expr::Number(6.0), Expr::Number(7.0)]),
                        ]
                    ),
                ]
            )
        );
        Ok(())
    }
}
