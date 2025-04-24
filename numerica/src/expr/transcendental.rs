use super::Expr;

fn apply_op(args: Vec<Expr>, op_name: &str, op: fn(f64) -> f64) -> Result<Expr, String> {
    let [arg]: [Expr; 1] = args
        .try_into()
        .map_err(|e| format!("Expected single arg. {:?}", e))?;
    match arg {
        Expr::Number(n) => Ok(Expr::Number(op(n))),
        // Handle list of args, apply op to each
        Expr::Head(h, a) if *h == Expr::Symbol("List".into()) => {
            let args = a
                .into_iter()
                .map(|x| match x {
                    Expr::Number(n) => Expr::Number(op(n)),
                    o => Expr::from_head(op_name, vec![o]),
                })
                .collect();
            Ok(Expr::from_head("List", args))
        }
        o => Ok(Expr::from_head(op_name, vec![o])),
    }
}

fn gamma(x: f64) -> f64 {
    #[link(name = "m")]
    unsafe extern "C" {
        fn tgamma(x: f64) -> f64;
    }
    unsafe { tgamma(x) }
}

pub(crate) fn eval_gamma(args: Vec<Expr>) -> Result<Expr, String> {
    apply_op(args, "Gamma", gamma)
}

pub(crate) fn eval_sin(args: Vec<Expr>) -> Result<Expr, String> {
    apply_op(args, "Sin", |x| x.sin())
}

pub(crate) fn eval_cos(args: Vec<Expr>) -> Result<Expr, String> {
    apply_op(args, "Cos", |x| x.cos())
}

pub(crate) fn eval_exp(args: Vec<Expr>) -> Result<Expr, String> {
    apply_op(args, "Exp", |x| x.exp())
}
