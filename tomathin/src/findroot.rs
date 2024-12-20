use crate::matrix::{dot_product, qr_decompose, Matrix};

pub fn explore_domain(
    f: impl Fn(f64) -> Result<f64, String>,
    (x0, xf): (f64, f64),
    divs: usize,
) -> Result<Vec<(f64, f64)>, String> {
    _explore_domain(&f, (x0, xf), divs, 4, 10)
}

fn _explore_domain(
    f: &impl Fn(f64) -> Result<f64, String>,
    (a, b): (f64, f64),
    divs: usize,
    max_division_depth: usize,
    nested_div_factor: usize,
) -> Result<Vec<(f64, f64)>, String> {
    // Sample the function at 100 points and check sign chnages hinting roots

    if a >= b || divs < 2 {
        return Err(format!("Empty explore interval {}-{}/{}", a, b, divs));
    }

    let dx = (b - a) / divs as f64;
    let points = (0..=divs)
        .map(|slot| {
            let x = a + dx * slot as f64;
            f(x).and_then(|fx| Ok((x, fx)))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut root_brackets = Vec::new();
    for w in points.windows(2) {
        let &[(x0, fx0), (x1, fx1)] = w else {
            unreachable!()
        };
        if fx1.signum() != fx0.signum() {
            root_brackets.push((x0, x1));
        }
    }

    if root_brackets.is_empty() && max_division_depth > 0 {
        _explore_domain(
            f,
            (a, b),
            10 * divs,
            max_division_depth - 1,
            nested_div_factor,
        )
    } else {
        Ok(root_brackets)
    }
}

pub fn find_root(f: impl Fn(f64) -> Result<f64, String>, x0: f64) -> Result<f64, String> {
    let esqrt = f64::EPSILON.sqrt();
    let tolerance = 1.0e-12;
    let mut x = x0;
    for _ in 0..100 {
        let h = esqrt * (x.abs() + 1.0); // keep h meaningful across scales
        let fx = f(x)?;
        if fx.abs() < tolerance {
            return Ok(x);
        }
        let f_central_diff = (f(x + h)? - f(x - h)?) / 2.0 / h;
        // Nudge x a bit if stuck at an extema (f'(x) is 0)
        if f_central_diff.abs() < 1.0e-10 {
            x += 1.0e-3;
            continue;
        }
        x = x - fx / f_central_diff;
    }
    Err(format!("Didn't converge, x={}", x))
}

pub fn regula_falsi(
    f: impl Fn(f64) -> Result<f64, String>,
    (mut a, mut b): (f64, f64),
) -> Result<f64, String> {
    let tolerance = 1.0e-12;
    let (mut fa, mut fb) = (f(a)?, f(b)?);
    let mut last_fx_sign = 0.0;
    for _ in 0..1000 {
        // NOTE: this update form instead of (b-a)/(fb-fa) avoids precision
        // loss // as 'a' and 'b' become close. fa, fb are opposite signs.
        let x = (a * fb - b * fa) / (fb - fa);
        let fx = f(x)?;
        if x.is_nan() || fx.is_nan() {
            return Err(format!(
                "Didn't converge: a={}, fa={}, b={}, fb={}, x={}, fx={}",
                a, fa, b, fb, x, fx
            ));
        }
        if fx.abs() < tolerance {
            return Ok(x);
        }
        if fa.signum() == fx.signum() {
            (a, fa) = (x, fx);
            // illinios variant: avoid endpoint stagnation
            // The 1/2 factor is not arbitrary, guarantees superlinear convergence.
            if fx.signum() == last_fx_sign {
                fb /= 2.0;
            }
        } else {
            (b, fb) = (x, fx);
            // illinios variant: avoid endpoint stagnation
            if fx.signum() == last_fx_sign {
                fa /= 2.0;
            }
        }
        last_fx_sign = fx.signum();
    }
    Err(format!("Didn't converge: Maxed iterations x={}", a))
}

pub fn bisection(
    f: impl Fn(f64) -> Result<f64, String>,
    (mut a, mut b): (f64, f64),
) -> Result<f64, String> {
    let tolerance = 1.0e-12;
    let (mut fa, mut fb) = (f(a)?, f(b)?);
    if fa.signum() == fb.signum() {
        return Err(format!(
            "Bisection requires points with opposing image, fa*fb={}",
            fa * fb
        ));
    }
    for _ in 0..1000 {
        let x = (a + b) / 2.0;
        let fx = f(x)?;
        if fx.abs() < tolerance {
            return Ok(x);
        }
        if fx.signum() == fa.signum() {
            (a, fa) = (x, fx);
        } else if fx.signum() == fb.signum() {
            (b, fb) = (x, fx);
        } else {
            break; // eg: NaN
        }
    }
    Err(format!("Didn't converge, x={}", a))
}

pub fn gauss_seidel(a: Vec<Vec<f64>>, b: Vec<f64>) -> Result<Vec<f64>, String> {
    gauss_seidel_impl(a, b, 1000, 1.0e-12)
}

fn gauss_seidel_impl(
    a: Vec<Vec<f64>>,
    b: Vec<f64>,
    max_iter: usize,
    tolerance: f64,
) -> Result<Vec<f64>, String> {
    let mut x = vec![0.0; b.len()];

    let mut best_row = Vec::new();
    // index a rows by largest diagonal weight
    for ii in 0..a.len() {
        let best = a
            .iter()
            .enumerate()
            .map(|(i, ai)| {
                (
                    i,
                    ai[ii].abs()
                        - ai.iter()
                            .enumerate()
                            .map(|(j, aij)| if ii != j { aij.abs() } else { 0.0 })
                            .sum::<f64>(),
                )
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        best_row.push(best.0);
    }

    for _ in 0..max_iter {
        let x_before = x.clone();
        for (c, r) in best_row.iter().enumerate() {
            let o: f64 = a[*r]
                .iter()
                .enumerate()
                .map(|(j, aij)| if j != c { aij * x[j] } else { 0.0 })
                .sum();
            x[c] = (b[*r] - o) / a[*r][c];
        }
        // Check for convergence
        let inf_norm = x_before
            .iter()
            .enumerate()
            .map(|(i, xb)| (xb - x[i]).abs())
            .reduce(f64::max)
            .unwrap();
        if inf_norm < tolerance {
            return Ok(x);
        }
    }
    Err("Didn't converge".to_string())
}

pub fn find_root_vec(
    f: Vec<impl Fn(&Vec<f64>) -> Result<f64, String>>,
    x0: Vec<f64>,
) -> Result<Vec<f64>, String> {
    let esqrt = f64::EPSILON.sqrt();
    let tolerance = 1.0e-12;

    let mut x = x0.clone();
    for _ in 0..100 {
        let mut jacobian = Vec::new();
        for r in 0..x.len() {
            let h = esqrt * (x[r].abs() + 1.0); // keep h meaningful across scales
            let mut jacobian_r = Vec::new();
            let mut x_m_h = x.clone();
            let mut x_p_h = x.clone();
            x_m_h[r] -= h;
            x_p_h[r] += h;
            for fc in &f {
                jacobian_r.push((fc(&x_p_h)? - fc(&x_m_h)?) / 2.0 / h);
            }
            jacobian.push(jacobian_r);
        }

        let b = f.iter().map(|fi| -fi(&x).unwrap()).collect();
        // let dx = gauss_seidel_impl(jacobian, b, 100, 1.0e-3)?;
        let dx = nsolve(Matrix::from_rows(jacobian), b);

        x = x
            .into_iter()
            .enumerate()
            .map(|(i, xi)| xi + dx[i])
            .collect();
        // Check for convergence
        let inf_norm = f
            .iter()
            .map(|fi| fi(&x).unwrap().abs())
            .reduce(f64::max)
            .unwrap();
        if inf_norm < tolerance {
            return Ok(x);
        }
    }
    Err("Didn't converge".to_string())
}

pub fn nsolve(a: Matrix, b: Vec<f64>) -> Vec<f64> {
    // orthogonalize a via gram schmidt
    // let q_t = gram_schmidt_orthonorm(&a).transpose();
    // let r = matmul(&q_t, &a);
    let (q, r) = qr_decompose(&a);
    let q_t = q.transpose();
    let c: Vec<_> = (0..q_t.num_rows())
        .map(|r| dot_product(&*q_t.row(r), &b))
        .collect();
    // we'll have as many unknowns as a as columns
    // if the system is under-determined though some will be left at 0 (c isn't that large)
    let mut x = vec![0.0; a.num_cols()];
    let xsize = std::cmp::min(a.num_rows(), a.num_cols());

    // r11 r12 r13  x1  c1
    //   0 r22 r23  x2  c2
    //   0   0 r33  x3  c3
    //
    // r33 * x3 = c3                         => x3 = c3 / r33
    // r22 * x2 + r23 * x3 = c2              => x2 = (c2 - r23 * x3) / r22
    // r11 * x1 + r12 * x2 + r13 * x3 = c1   => x1 = (c1 - r12 * x2 - r13 * x3) / r11

    for n in (0..xsize).rev() {
        x[n] = (c[n] - dot_product(&r.row(n)[n + 1..], &x[n + 1..])) / r[(n, n)];
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: &Vec<f64>, b: &Vec<f64>) {
        let e = 1.0e-9;
        assert_eq!(a.len(), b.len());
        assert!(
            a.iter()
                .zip(b.iter())
                .map(|(a, b)| (a - b).abs() < e)
                .all(|d| d),
            "a: {:?}, b: {:?}",
            a,
            b
        );
    }

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
    }

    #[test]
    fn test_gauss_seidel() -> Result<(), String> {
        let x = gauss_seidel(vec![vec![16.0, 3.0], vec![7.0, -11.0]], vec![11.0, 13.0])?;
        approx_eq(&x, &vec![160.0 / 197.0, -131.0 / 197.0]);

        let x = gauss_seidel(
            vec![
                vec![10.0, -1.0, 2.0, 0.0],
                vec![-1.0, 11.0, -1.0, 3.0],
                vec![2.0, -1.0, 10.0, -1.0],
                vec![0.0, 3.0, -1.0, 8.0],
            ],
            vec![6.0, 25.0, -11.0, 15.0],
        )?;
        approx_eq(&x, &vec![1.0, 2.0, -1.0, 1.0]);

        let x = gauss_seidel(vec![vec![0.2, 1.1], vec![2.2, 0.1]], vec![2.78, 0.89])?;
        approx_eq(&x, &vec![0.29208333333336917, 2.47416666666666]);
        Ok(())
    }

    #[test]
    fn test_find_root_vec() {
        let f = vec![
            |x: &Vec<f64>| Ok(x[0].powi(2) + x[1].powi(2) - 4.0),
            |x: &Vec<f64>| Ok(x[0] * x[1] - 1.0),
        ];
        let x = find_root_vec(f, vec![0.1, 1.1]).unwrap();
        approx_eq(&x, &vec![-1.9318516525782186, -0.5176380902051412]);
    }

    #[test]
    fn test_find_root_vec2() {
        let f = vec![|x: &Vec<f64>| Ok((x[0] - 2.0).exp() - x[1]), |x: &Vec<
            f64,
        >| {
            Ok(x[1].powi(2) - x[0])
        }];
        let x = find_root_vec(f, vec![1.0, 1.0]).unwrap();
        approx_eq(&x, &vec![0.0190260161037140, 0.137934825565243]);
    }

    #[test]
    fn test_nsolve() {
        let x = nsolve(
            Matrix::from_rows(vec![vec![16.0, 3.0], vec![7.0, -11.0]]),
            vec![11.0, 13.0],
        );
        approx_eq(&x, &vec![160.0 / 197.0, -131.0 / 197.0]);

        let x = nsolve(
            Matrix::from_rows(vec![
                vec![10.0, -1.0, 2.0, 0.0],
                vec![-1.0, 11.0, -1.0, 3.0],
                vec![2.0, -1.0, 10.0, -1.0],
                vec![0.0, 3.0, -1.0, 8.0],
            ]),
            vec![6.0, 25.0, -11.0, 15.0],
        );
        approx_eq(&x, &vec![1.0, 2.0, -1.0, 1.0]);

        let x = nsolve(
            Matrix::from_rows(vec![
                vec![10.0, -1.0, 2.0, 0.0],
                vec![-1.0, 11.0, -1.0, 3.0],
                vec![2.0, -1.0, 10.0, -1.0],
            ]),
            vec![6.0, 25.0, -11.0],
        );
        println!("{:?}", x);

        let x = nsolve(
            Matrix::from_rows(vec![vec![0.2, 1.1], vec![2.2, 0.1]]),
            vec![2.78, 0.89],
        );
        approx_eq(&x, &vec![0.29208333333336917, 2.47416666666666]);

        let x = nsolve(
            Matrix::from_iter([1.0, 2.0, 3.0, 4.0, 5.0], 5, 1),
            vec![2.0, 5.0, 3.0, 8.0, 7.0],
        );
        println!("{:?}", x);

        let x = nsolve(
            Matrix::from_iter([1.0, 1.0, 2.0, 1.0, 3.0, 1.0, 4.0, 1.0, 5.0, 1.0], 5, 2),
            vec![2.0, 5.0, 3.0, 8.0, 7.0],
        );
        println!("{:?}", x);
    }

    #[test]
    fn test_explore_domain() {
        let f = |x: f64| {
            let mut sum = -1000.0;
            for i in 1..=5 {
                sum += 100.0 / (1.0 + x).powi(i)
            }
            Ok(sum)
        };
        let brackets = explore_domain(f, (-1000.0, 1000.0), 10).unwrap();
        assert_eq!(brackets.len(), 2);
        let (x0, x1) = brackets[0];
        assert!(regula_falsi(f, (x0, x1)).is_err());
        let (x0, x1) = brackets[1];
        let root = regula_falsi(f, (x0, x1)).unwrap();
        approx_eq(&vec![root], &vec![-0.1940185201887317]);
    }
}
