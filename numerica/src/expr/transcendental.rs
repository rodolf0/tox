use super::{Expr, evaluate};
use crate::context::Context;

pub fn eval_gamma(args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    let [arg]: [Expr; 1] = args
        .try_into()
        .map_err(|e| format!("Expected single arg. {:?}", e))?;
    // TODO: arg is already evaluated (same for all others)
    match evaluate(arg, ctx)? {
        Expr::Number(n) => Ok(Expr::Number(crate::gamma(1.0 + n))),
        other => Ok(Expr::Head(
            Box::new(Expr::Symbol("Gamma".into())),
            vec![other],
        )),
    }
}

pub fn eval_sin(args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    let [arg]: [Expr; 1] = args
        .try_into()
        .map_err(|e| format!("Expected single arg. {:?}", e))?;
    match evaluate(arg, ctx)? {
        Expr::Number(n) => Ok(Expr::Number(n.sin())),
        other => Ok(Expr::Head(
            Box::new(Expr::Symbol("Sin".into())),
            vec![other],
        )),
    }
}

pub fn eval_cos(args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    let [arg]: [Expr; 1] = args
        .try_into()
        .map_err(|e| format!("Expected single arg. {:?}", e))?;
    match evaluate(arg, ctx)? {
        Expr::Number(n) => Ok(Expr::Number(n.cos())),
        other => Ok(Expr::Head(
            Box::new(Expr::Symbol("Cos".into())),
            vec![other],
        )),
    }
}

pub fn eval_exp(args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    let [arg]: [Expr; 1] = args
        .try_into()
        .map_err(|e| format!("Expected single arg. {:?}", e))?;
    match evaluate(arg, ctx)? {
        Expr::Number(n) => Ok(Expr::Number(n.exp())),
        other => Ok(Expr::Head(
            Box::new(Expr::Symbol("Exp".into())),
            vec![other],
        )),
    }
}
