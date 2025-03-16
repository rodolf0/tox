use super::{eval_with_ctx, Expr};
use crate::context::Context;

pub fn eval_gamma(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    if args.len() != 1 {
        Err(format!("Gamma expects single arg. {:?}", args))
    } else {
        match eval_with_ctx(args.swap_remove(0), ctx)? {
            Expr::Number(n) => Ok(Expr::Number(crate::gamma(1.0 + n))),
            other => Ok(Expr::Expr("Gamma".to_string(), vec![other])),
        }
    }
}

pub fn eval_sin(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    match eval_with_ctx(args.swap_remove(0), ctx)? {
        Expr::Number(n) => Ok(Expr::Number(n.sin())),
        other => Ok(Expr::Expr("Sin".to_string(), vec![other])),
    }
}

pub fn eval_cos(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    match eval_with_ctx(args.swap_remove(0), ctx)? {
        Expr::Number(n) => Ok(Expr::Number(n.cos())),
        other => Ok(Expr::Expr("Cos".to_string(), vec![other])),
    }
}

pub fn eval_exp(mut args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    match eval_with_ctx(args.swap_remove(0), ctx)? {
        Expr::Number(n) => Ok(Expr::Number(n.exp())),
        other => Ok(Expr::Expr("Exp".to_string(), vec![other])),
    }
}
