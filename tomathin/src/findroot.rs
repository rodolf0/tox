use crate::matrix::{dot_product, qr_decompose, Matrix};

pub fn find_root(f: impl Fn(f64) -> Result<f64, String>, x0: f64) -> Result<f64, String> {
    let esqrt = f64::EPSILON.sqrt();
    let tolerance = 1.0e-9;
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
    let h = 1.0e-5;
    let tolerance = 1.0e-12;

    let mut x = x0.clone();
    for _ in 0..100 {
        let mut jacobian = Vec::new();
        for r in 0..x.len() {
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
}
