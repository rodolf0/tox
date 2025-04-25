use numerica::{Context, Expr};

pub fn plot_histogram(expr: &Expr, ctx: &mut Context) -> Result<(), String> {
    const SAMPLES: f64 = 50000.0;
    const BINS: f64 = 31.0;
    let r = numerica::evaluate(
        numerica::Expr::from_head(
            "Histogram",
            vec![expr.clone(), Expr::Number(SAMPLES), Expr::Number(BINS)],
        ),
        ctx,
    )?;
    // pull out the list
    let Expr::Head(_, buckets) = r else {
        panic!("Expected Expr::Head('List', ...)");
    };
    let mut max_freq: f64 = 0.0;
    for bucket in &buckets {
        let Expr::Head(_, bucket_def) = bucket else {
            panic!("Expected Expr::Head('List', ...)");
        };
        let Expr::Number(freq) = bucket_def[1] else {
            panic!("Expected Expr::Number(freq)");
        };
        max_freq = max_freq.max(freq);
    }
    for bucket in buckets {
        let Expr::Head(_, bucket_def) = bucket else {
            panic!("Expected Expr::Head('List', ...)");
        };
        let Expr::Number(center) = bucket_def[0] else {
            panic!("Expected Expr::Number(center)");
        };
        let Expr::Number(freq) = bucket_def[1] else {
            panic!("Expected Expr::Number(freq)");
        };
        println!(
            "{:10.2} {}",
            center,
            "\u{2b24}".repeat((50.0 * freq / max_freq) as usize)
        );
    }
    Ok(())
}
