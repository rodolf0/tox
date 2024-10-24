pub fn find_root(f: impl Fn(f64) -> Result<f64, String>, x0: f64) -> Result<f64, String> {
    let h = 1.0e-5;
    let tolerance = 1.0e-8;
    let mut x = x0;
    for _ in 1..100 {
        let fx = f(x)?;
        if fx.abs() < tolerance {
            return Ok(x);
        }
        let f_central_diff = (f(x + h)? - f(x - h)?) / 2.0 / h;
        x = x - fx / f_central_diff;
    }
    Err("Didn't converge".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_root() {
        let f = |x: f64| {
            let mut sum = -360.0;
            for i in 0..4 {
                sum += 75.0 / (1.0 + x).powi(i)
            }
            Ok(sum)
        };
        println!("{}", find_root(f, 0.3).unwrap());
        println!("{}", find_root(f, 3.3).unwrap());
        println!("{}", find_root(f, -300.3).unwrap());
    }
}
