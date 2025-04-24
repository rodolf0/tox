use super::replace_all::replace_all;
use super::{Expr, evaluate};
use crate::context::Context;
use crate::{find_root_vec, findroot};

pub(crate) fn eval_find_root(args: Vec<Expr>) -> Result<Expr, String> {
    let [fexpr, varspec]: [Expr; 2] = args
        .try_into()
        .map_err(|e| format!("FindRoot must have 2 arguments. {:?}", e))?;

    // Adapt to single or multiple functions
    let fexpr = match fexpr {
        Expr::Head(h, a) if *h == Expr::Symbol("List".into()) => a,
        expr => vec![expr],
    };

    // Pull out variables specs to find roots for.
    let varspec: Vec<_> = match varspec {
        Expr::Head(h, a) if *h == Expr::Symbol("List".into()) => a,
        o => return Err(format!("Unexpected var spec for FindRoot: {:?}", o)),
    };

    let vars: Vec<(Expr, f64, Option<(f64, f64)>)> = match varspec.as_slice() {
        [v @ Expr::Symbol(_), Expr::Number(start)] => vec![(v.clone(), *start, None)],
        [
            v @ Expr::Symbol(_),
            Expr::Number(start),
            Expr::Number(lo),
            Expr::Number(hi),
        ] => {
            vec![(v.clone(), *start, Some((*lo, *hi)))]
        }
        // TODO: clean this up
        vspec
            if vspec
                .iter()
                .all(|s| matches!(s, Expr::Head(h, _) if **h == Expr::Symbol("List".into()))) =>
        {
            vspec
                .into_iter()
                .map(|s| {
                    let Expr::Head(_, varspec) = s else {
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

    // TODO move this to aanother function
    if fexpr.len() == 1 {
        let f =
            |xi: f64| match replace_all(fexpr[0].clone(), &[(vars[0].0.clone(), Expr::Number(xi))])
                .and_then(|expr| evaluate(expr, &mut Context::new()))
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
        return Ok(Expr::from_head(
            "List",
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
            .and_then(|expr| evaluate(expr, &mut Context::new()))
            {
                Ok(Expr::Number(x)) => Ok(x),
                err => Err(format!("FindRoot didn't return Number: {:?}", err)),
            }
        })
        .collect();

    let roots = find_root_vec(funcs, vars.iter().map(|vi| vi.1).collect())?;
    Ok(Expr::from_head(
        "List",
        roots.into_iter().map(|ri| Expr::Number(ri)).collect(),
    ))
}
