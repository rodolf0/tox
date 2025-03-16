use super::{eval_with_ctx, Expr};
use crate::context::Context;
use rand_distr::Distribution;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum Distr {
    Normal(rand_distr::Normal<f64>),
    Poisson(rand_distr::Poisson<f64>),
}

impl Distr {
    pub fn sample(&self) -> f64 {
        match self {
            Distr::Normal(d) => d.sample(&mut rand::rng()),
            Distr::Poisson(d) => d.sample(&mut rand::rng()),
        }
    }
}

pub fn eval_normal_dist(args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    let [mu, sigma]: [f64; 2] = args
        .into_iter()
        .map(|a| match eval_with_ctx(a, ctx) {
            Ok(Expr::Number(n)) => Ok(n),
            Ok(other) => Err(format!("NormalDist params must be number. {:?}", other)),
            Err(e) => Err(e),
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|e| format!("NormalDist error: {:?}", e))?;
    Ok(Expr::Distribution(Rc::new(Distr::Normal(
        rand_distr::Normal::new(mu, sigma).map_err(|e| e.to_string())?,
    ))))
}
