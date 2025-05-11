use super::Expr;
use crate::context::Context;
use crate::itertools::TableIterator;

pub(crate) fn eval_table(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    // first arg is the expression that will be evaluated for each table element
    let table_expr = args.remove(0);
    let table_idxs = args
        .into_iter()
        .map(|s| parse_index_spec(s))
        .collect::<Result<Vec<_>, _>>()?;
    // Evaluate the expression on each table value
    let table_expr = Expr::Function(
        table_idxs.iter().map(|i| i.var.clone()).collect(),
        Box::new(table_expr),
    );

    // Deduce the shape of the final table
    let table_shape = table_idxs
        .iter()
        .map(|s| (((s.end - s.start) / s.step).floor() + 1.0) as usize)
        .collect();

    // Iterate overa all indices generating the flat table
    let flat_table = TableIterator::new(
        table_idxs
            .into_iter()
            .map(|s| (s.start, s.end, s.step))
            .collect(),
    )
    .map(|idxs| {
        // Evaluate function at this input table indices
        super::apply(
            table_expr.clone(),
            idxs.into_iter().map(|i| Expr::Number(i)).collect(),
            ctx,
        )
    })
    .collect::<Result<Vec<_>, _>>()?;

    super::listops::reshape(&Expr::Symbol("List".into()), flat_table, table_shape)
}

struct IdxRange {
    var: String,
    start: f64,
    end: f64,
    step: f64,
}

fn parse_index_spec(spec: Expr) -> Result<IdxRange, String> {
    match spec {
        Expr::Head(h, spec) if *h == Expr::Symbol("List".into()) => match spec.as_slice() {
            [Expr::Symbol(s), Expr::Number(imax)] => Ok(IdxRange {
                var: s.clone(),
                start: 1.0,
                end: *imax,
                step: 1.0,
            }),
            [Expr::Symbol(s), Expr::Number(imin), Expr::Number(imax)] => Ok(IdxRange {
                var: s.clone(),
                start: *imin,
                end: *imax,
                step: 1.0,
            }),
            [
                Expr::Symbol(s),
                Expr::Number(imin),
                Expr::Number(imax),
                Expr::Number(di),
            ] => Ok(IdxRange {
                var: s.clone(),
                start: *imin,
                end: *imax,
                step: *di,
            }),
            other => Err(format!("Table spec not supported. {:?}", other)),
        },
        other => Err(format!("Table spec not supported. {:?}", other)),
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
