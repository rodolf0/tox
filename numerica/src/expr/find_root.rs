use super::replace_all::replace_all;
use super::{eval_with_ctx, Expr};
use crate::context::Context;
use crate::{find_root_vec, findroot};

pub fn eval_find_root(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    // Evaluate and pull out list of functions to work on.
    let fexpr: Vec<Expr> = match args.remove(0) {
        Expr::Expr(h, a) if h == "List" => a
            .into_iter()
            .map(|fi| eval_with_ctx(fi, ctx))
            .collect::<Result<_, _>>()?,
        expr => vec![eval_with_ctx(expr, ctx)?],
    };
    // Pull out variables specs to find roots for.
    let varspec: Vec<_> = match args.remove(0) {
        Expr::Expr(h, a) if h == "List" => a,
        other => return Err(format!("Unexpected var spec for FindRoot: {:?}", other)),
    };
    let vars: Vec<(Expr, f64, Option<(f64, f64)>)> = match varspec.as_slice() {
        [v @ Expr::Symbol(_), Expr::Number(start)] => vec![(v.clone(), *start, None)],
        [v @ Expr::Symbol(_), Expr::Number(start), Expr::Number(lo), Expr::Number(hi)] => {
            vec![(v.clone(), *start, Some((*lo, *hi)))]
        }
        o if o
            .iter()
            .all(|s| matches!(s, Expr::Expr(h, _) if h == "List")) =>
        {
            o.into_iter()
                .map(|s| {
                    let Expr::Expr(_, varspec) = s else {
                        unreachable!();
                    };
                    match varspec.as_slice() {
                        [v @ Expr::Symbol(_), Expr::Number(start)] => (v.clone(), *start, None),
                        _ => todo!(),
                    }
                })
                .collect()
        }
        _ => todo!(),
    };

    if vars.len() != fexpr.len() {
        return Err(format!(
            "Need same number of functions as unknowns, {} vs {}",
            vars.len(),
            fexpr.len()
        ));
    }

    if fexpr.len() == 1 {
        let f =
            |xi: f64| match replace_all(fexpr[0].clone(), &[(vars[0].0.clone(), Expr::Number(xi))])
                .and_then(|expr| eval_with_ctx(expr, &mut Context::new()))
            {
                Ok(Expr::Number(x)) => Ok(x),
                err => Err(format!("FindRoot didn't return Number: {:?}", err)),
            };
        let x0 = vars[0].1;
        let roots = if let Some((lo, hi)) = vars[0].2 {
            vec![findroot::regula_falsi(f, (lo, hi))?]
        } else {
            findroot::find_roots(f, x0)?
        };
        return Ok(Expr::Expr(
            "List".to_string(),
            roots.into_iter().map(|ri| Expr::Number(ri)).collect(),
        ));
    }

    let funcs: Vec<_> = fexpr
        .into_iter()
        .map(|f| {
            let vars = &vars;
            move |xi: &Vec<f64>| match replace_all(
                f.clone(),
                &vars
                    .iter()
                    .zip(xi)
                    .map(|(var, val)| (var.0.clone(), Expr::Number(*val)))
                    .collect::<Vec<_>>(),
            )
            .and_then(|expr| eval_with_ctx(expr, &mut Context::new()))
            {
                Ok(Expr::Number(x)) => Ok(x),
                err => Err(format!("FindRoot didn't return Number: {:?}", err)),
            }
        })
        .collect();

    let roots = find_root_vec(funcs, vars.iter().map(|vi| vi.1).collect())?;
    Ok(Expr::Expr(
        "List".to_string(),
        roots.into_iter().map(|ri| Expr::Number(ri)).collect(),
    ))
}
