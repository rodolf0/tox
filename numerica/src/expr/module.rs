use super::Expr;
use crate::context::Context;

fn replace_temp_sym(expr: Expr, from: &str, to: &str) -> Expr {
    match expr {
        // Recursively apply replacement for arguments
        Expr::Head(head, args) => Expr::Head(
            head,
            args.into_iter()
                .map(|e| replace_temp_sym(e, from, to))
                .collect(),
        ),
        Expr::Symbol(x) if x == from => Expr::Symbol(to.into()),
        expr => expr,
    }
}

fn rand16() -> u16 {
    (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
        ^ 0x5a5a) as u16
}

fn parse_locals(locals: &[Expr]) -> Result<Vec<String>, String> {
    locals
        .iter()
        .map(|s| match s {
            Expr::Symbol(sym) => Ok(sym.clone()),
            // Allow Set/SetDelayed expressions in locals
            // NOTE: can possible be factored out for With head
            Expr::Head(h, args)
                if **h == Expr::Symbol("Set".into())
                    || **h == Expr::Symbol("SetDelayed".into()) =>
            {
                match args.get(0) {
                    Some(Expr::Symbol(lhs)) => Ok(lhs.clone()),
                    o => Err(format!(
                        "Malformed Set(Delayed) expr within Module locals. {:?}",
                        o
                    )),
                }
            }
            _ => Err(format!("Module locals must be Symbol/Set(Delayed): {}", s)),
        })
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) fn eval_module(args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    let [locals, expr]: [Expr; 2] = args
        .try_into()
        .map_err(|e| format!("Module requires a list of locals and an expr. {:?}", e))?;
    let locals = match locals {
        Expr::Head(h, syms) if *h == Expr::Symbol("List".into()) => Ok(syms),
        o => Err(format!("Module locals need to be a List. Got {:?}", o)),
    }?;
    let local_syms = parse_locals(&locals)?;
    // Rename each local variable to var__<random>
    let renamed_syms: Vec<_> = local_syms
        .iter()
        .map(|l| format!("{}__{}", l, rand16()))
        .collect();
    // Re-write the Module body to use the new symbol names
    let expr = local_syms
        .iter()
        .zip(&renamed_syms)
        .fold(expr, |reexpr, (local, rename)| {
            replace_temp_sym(reexpr, local, &rename)
        });
    // Re-write the locals asignment expressions to use new names
    let locals: Vec<_> = locals
        .into_iter()
        .map(|local_expr| {
            local_syms
                .iter()
                .zip(&renamed_syms)
                .fold(local_expr, |relocal, (local, rename)| {
                    replace_temp_sym(relocal, local, &rename)
                })
        })
        .collect();

    // Emulate CompoundExpression of locals + body
    // Body hasn't been evaluated yet. Module has HoldAll attr.
    // Now that we've got values for all parameters, we can evaluate.
    locals
        .into_iter()
        .map(|e| super::evaluate(e, ctx))
        .collect::<Result<Vec<_>, _>>()?;
    super::evaluate(expr, ctx)
}
