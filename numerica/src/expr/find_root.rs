use super::Expr;
use crate::context::Context;
use crate::{find_root_vec, findroot};

struct VarSpec {
    sym: String,
    start: f64,
    range: Option<(f64, f64)>,
}

fn parse_varspec(varspec: Expr) -> Result<Vec<VarSpec>, String> {
    use super::Expr::*;
    // Normalize single and multiple variable specifications.
    // Single: {x0, x0_start, x0_range}, Multiple: {{x0, x0_start, x0_range}, ...}
    let varspec = match varspec {
        Head(h, spec) if *h == Symbol("List".into()) && spec.len() > 0 => match &spec[0] {
            // List of Lists implies multiple variables. Return the full spec.
            Head(h, _) if **h == Symbol("List".into()) => spec,
            // Single variable specification. Wrap in a list for uniformity.
            Symbol(_) => vec![Expr::from_head("List", spec)],
            o => return Err(format!("Unexpected var spec for FindRoot: {}", o)),
        },
        _ => return Err(format!("Unexpected var spec for FindRoot: {}", varspec)),
    };

    varspec
        .into_iter()
        .map(|vs| {
            let Head(_, vs) = vs else {
                return Err(format!("Unexpected var spec for FindRoot: {:?}", vs));
            };
            // Match individual var spec {x0, x0_start, x0_lo, x0_hi}
            match vs.as_slice() {
                [Symbol(sym), Number(start)] => Ok(VarSpec {
                    sym: sym.clone(),
                    start: *start,
                    range: None,
                }),
                [Symbol(sym), Number(start), Number(lo), Number(hi)] => Ok(VarSpec {
                    sym: sym.clone(),
                    start: *start,
                    range: Some((*lo, *hi)),
                }),
                _ => Err(format!("Unexpected var spec for FindRoot: {:?}", vs)),
            }
        })
        .collect::<Result<Vec<_>, _>>()
}

fn nonlinear_root(fexpr: Expr, varspec: VarSpec) -> Result<Vec<f64>, String> {
    use super::Expr::*;
    let ctx = &mut Context::new();
    // Wrap the expression we're finding the root of in a Function
    let fexpr = crate::evaluate(
        Expr::from_head("Function", vec![Symbol(varspec.sym), fexpr]),
        ctx,
    )?;
    let f = |xi: f64| match super::apply(fexpr.clone(), vec![Number(xi)], &mut Context::new()) {
        Ok(Number(x)) => Ok(x),
        o => Err(format!("FindRoot didn't return Number: {:?}", o)),
    };
    match varspec.range {
        Some((lo, hi)) => Ok(vec![findroot::regula_falsi(f, (lo, hi))?]),
        None => findroot::find_roots(f, varspec.start),
    }
}

fn nonlinear_system(fexprs: Vec<Expr>, varspecs: Vec<VarSpec>) -> Result<Vec<f64>, String> {
    use super::Expr::*;
    let ctx = &mut Context::new();
    let fexprs = fexprs
        .into_iter()
        .map(|fexpr| {
            crate::evaluate(
                Expr::from_head(
                    "Function",
                    vec![
                        Expr::from_head(
                            "List",
                            varspecs.iter().map(|vs| Symbol(vs.sym.clone())).collect(),
                        ),
                        fexpr,
                    ],
                ),
                ctx,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let funcs = fexprs
        .into_iter()
        .map(|fexpr| {
            move |xi: &Vec<f64>| match super::apply(
                fexpr.clone(),
                xi.iter().map(|&x| Number(x)).collect(),
                &mut Context::new(),
            ) {
                Ok(Number(x)) => Ok(x),
                o => Err(format!("FindRoot didn't return Number: {:?}", o)),
            }
        })
        .collect();

    find_root_vec(funcs, varspecs.iter().map(|vi| vi.start).collect())
}

pub(crate) fn eval_find_root(args: Vec<Expr>) -> Result<Expr, String> {
    let [fexpr, varspec]: [Expr; 2] = args
        .try_into()
        .map_err(|e| format!("FindRoot must have 2 arguments. {:?}", e))?;

    // Pull out variables specs to find roots for.
    let mut varspec = parse_varspec(varspec)?;

    // Adapt to single or multiple functions
    let mut fexpr = match fexpr {
        Expr::Head(h, a) if *h == Expr::Symbol("List".into()) => a,
        expr => vec![expr],
    };

    if varspec.len() != fexpr.len() {
        return Err(format!(
            "Findroot unknowns != func-count, {} vs {}",
            varspec.len(),
            fexpr.len()
        ));
    }

    if fexpr.len() == 1 {
        let roots = nonlinear_root(fexpr.swap_remove(0), varspec.swap_remove(0))?;
        return Ok(Expr::from_head(
            "List",
            roots.into_iter().map(|ri| Expr::Number(ri)).collect(),
        ));
    } else {
        let roots = nonlinear_system(fexpr, varspec)?;
        return Ok(Expr::from_head(
            "List",
            roots.into_iter().map(|ri| Expr::Number(ri)).collect(),
        ));
    }
}
