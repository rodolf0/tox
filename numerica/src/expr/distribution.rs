use super::Expr;
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

pub(crate) fn eval_normal_dist(args: Vec<Expr>) -> Result<Expr, String> {
    let [mu, sigma]: [f64; 2] = args
        .into_iter()
        .map(|a| match a {
            Expr::Number(n) => Ok(n),
            other => Err(format!("NormalDist params must be number. {:?}", other)),
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|e| format!("NormalDist error: {:?}", e))?;
    Ok(Expr::Distribution(Rc::new(Distr::Normal(
        rand_distr::Normal::new(mu, sigma).map_err(|e| e.to_string())?,
    ))))
}

pub(crate) fn eval_unsure(args: Vec<Expr>) -> Result<Expr, String> {
    let [low, high]: [Expr; 2] = args
        .try_into()
        .map_err(|e| format!("Unsure needs 2 arguments. {:?}", e))?;
    let (Expr::Number(low), Expr::Number(high)) = (low, high) else {
        return Err(format!("Unsure needs a numbers for interval."));
    };
    let mu = Expr::Number((high + low) / 2.0);
    let sigma = Expr::Number((high - low).abs() / 3.92); // 2x z-score 95%
    eval_normal_dist(vec![mu, sigma])
}

pub fn is_stochastic(expr: &Expr) -> bool {
    match expr {
        Expr::Distribution(_) => true,
        Expr::Head(h, args) => is_stochastic(&*h) || args.iter().any(is_stochastic),
        Expr::Function(_, body) => is_stochastic(body),
        _ => false,
    }
}

fn sample_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Distribution(d) => Expr::Number(d.sample()),
        Expr::Head(h, args) => Expr::Head(
            Box::new(sample_expr(h.as_ref())),
            args.iter().map(sample_expr).collect(),
        ),
        Expr::Function(p, body) => Expr::Function(p.clone(), Box::new(sample_expr(body))),
        o => o.clone(),
    }
}

pub(crate) fn eval_sample(args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    let [expr]: [Expr; 1] = args
        .try_into()
        .map_err(|e| format!("Sample must have an expr. {:?}", e))?;
    // expr has already been evaluated, here we pick samples from nested
    // distributions and then re-evaluate expr with concrete values.
    crate::evaluate(sample_expr(&expr), ctx)
}

pub(crate) fn eval_histogram(args: Vec<Expr>, ctx: &mut Context) -> Result<Expr, String> {
    let [expr, nsamples, nbuckets]: [Expr; 3] = args
        .try_into()
        .map_err(|e| format!("Histogram needs expr, num-samples, num-buckets. {:?}", e))?;
    let Expr::Number(nsamples) = nsamples else {
        return Err(format!("Histogram num-samples must be a number."));
    };
    let Expr::Number(nbuckets) = nbuckets else {
        return Err(format!("Histogram num-buckets must be a number."));
    };
    // expr has already been evaluated, here we pick samples from nested
    // distributions and then re-evaluate expr with concrete values.
    let samples = (0..nsamples as u32)
        .map(|_| crate::evaluate(sample_expr(&expr), ctx))
        .collect::<Result<Vec<_>, _>>()?;

    let mut max = f64::MIN;
    let mut min = f64::MAX;
    for sample in &samples {
        match sample {
            Expr::Number(n) => {
                max = max.max(*n);
                min = min.min(*n);
            }
            _ => return Err(format!("Histogram samples must be numbers.")),
        }
    }
    let bucket_width = (max - min) / nbuckets;
    let mut histogram = vec![0.0; nbuckets as usize];
    for sample in &samples {
        let Expr::Number(sample) = sample else {
            unreachable!()
        };
        let idx = ((sample - min) / bucket_width) as usize;
        let idx = idx.min(histogram.len() - 1);
        histogram[idx] += 1.0;
    }

    Ok(Expr::from_head(
        "List",
        histogram
            .into_iter()
            .enumerate()
            .map(|(idx, n)| {
                Expr::from_head(
                    "List",
                    vec![
                        Expr::Number(min + (0.5 + (idx as f64)) * bucket_width),
                        Expr::Number(n),
                    ],
                )
            })
            .collect(),
    ))
}
