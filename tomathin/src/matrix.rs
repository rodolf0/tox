use std::borrow::Cow;
use std::ops::{Index, IndexMut, Mul, Range, Sub};

#[derive(Debug, Clone)]
pub struct Matrix<'a> {
    data: Cow<'a, [f64]>,
    shape: (usize, usize),  // rows, cols
    stride: (usize, usize), // move between rows, cols
    offset: (usize, usize),
}

impl<'a> Matrix<'a> {
    pub fn from_rows(rows: Vec<Vec<f64>>) -> Self {
        assert!(rows.len() != 0 && rows[0].len() != 0, "Empty rows or cols");
        let n_rows = rows.len();
        let n_cols = rows[0].len();
        Matrix {
            data: Cow::from(rows.into_iter().flat_map(|v| v).collect::<Vec<_>>()),
            shape: (n_rows, n_cols),
            stride: (n_cols, 1),
            offset: (0, 0),
        }
    }

    pub fn col(&self, col: usize) -> Matrix {
        self.get(0..self.shape.0, col..col + 1)
    }

    pub fn row(&self, row: usize) -> Matrix {
        self.get(row..row + 1, 0..self.shape.1)
    }

    pub fn eye(n: usize) -> Matrix<'a> {
        Matrix {
            data: Cow::from(
                (0..n * n)
                    .map(|i| if i % (n + 1) == 0 { 1.0 } else { 0.0 })
                    .collect::<Vec<_>>(),
            ),
            shape: (n, n),
            stride: (n, 1),
            offset: (0, 0),
        }
    }
}

impl<'a> Matrix<'a> {
    pub fn get(&self, row: Range<usize>, col: Range<usize>) -> Matrix {
        assert!(
            row.end <= self.shape.0 && col.end <= self.shape.1,
            "Out of range row={}/{}, col={}/{}",
            row.end,
            self.shape.0,
            col.end,
            self.shape.1
        );
        Matrix {
            data: Cow::Borrowed(&self.data),
            shape: (row.end - row.start, col.end - col.start),
            stride: self.stride,
            offset: (self.offset.0 + row.start, self.offset.1 + col.start),
        }
    }

    pub fn to_owned<'b>(&'a self) -> Matrix<'b> {
        Matrix {
            data: Cow::Owned(self.into_iter().cloned().collect()),
            offset: (0, 0),
            shape: self.shape,
            stride: (self.shape.1, 1),
        }
    }

    pub fn transpose(&self) -> Matrix {
        Matrix {
            data: Cow::Borrowed(&self.data),
            shape: (self.shape.1, self.shape.0),
            stride: (self.stride.1, self.stride.0),
            offset: (self.offset.1, self.offset.0),
        }
    }
}

impl<'a> Index<(usize, usize)> for Matrix<'a> {
    type Output = f64;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        let idx = (self.offset.0 + row) * self.stride.0 + (self.offset.1 + col) * self.stride.1;
        &self.data[idx]
    }
}

impl<'a> IndexMut<(usize, usize)> for Matrix<'a> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        let idx = (self.offset.0 + row) * self.stride.0 + (self.offset.1 + col) * self.stride.1;
        &mut self.data.to_mut()[idx]
    }
}

impl<'a> IntoIterator for &'a Matrix<'a> {
    type Item = &'a f64;
    type IntoIter = Box<dyn Iterator<Item = &'a f64> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new((0..self.shape.0).flat_map(|r| {
            self.data
                .iter()
                .skip((self.offset.0 + r) * self.stride.0 + self.offset.1 * self.stride.1)
                .step_by(self.stride.1)
                .take(self.shape.1)
        }))
    }
}

impl<'a> Mul<Matrix<'_>> for Matrix<'a> {
    type Output = Matrix<'a>;

    fn mul(self, rhs: Matrix) -> Matrix<'a> {
        matmul(&self, &rhs)
    }
}

impl<'a> Sub<Matrix<'_>> for Matrix<'a> {
    type Output = Matrix<'a>;

    fn sub(self, rhs: Matrix) -> Matrix<'a> {
        assert_eq!(self.shape, rhs.shape, "Can't Sub matrix of different size");
        Matrix {
            data: Cow::Owned(
                self.data
                    .iter()
                    .zip(&*rhs.data)
                    .map(|(l, r)| l - r)
                    .collect(),
            ),
            shape: (self.shape.0, rhs.shape.1),
            stride: (rhs.shape.1, 1),
            offset: (0, 0),
        }
    }
}

fn proj(v: &Vec<f64>, u: Matrix) -> Vec<f64> {
    let scale = dot(v, &u) / dot(&u, &u);
    u.into_iter().map(|ui| ui * scale).collect()
}

fn dot<'a>(a: impl IntoIterator<Item = &'a f64>, b: impl IntoIterator<Item = &'a f64>) -> f64 {
    a.into_iter()
        .zip(b.into_iter())
        .map(|(ai, bi)| ai * bi)
        .sum::<f64>()
}

fn outer<'a>(a: &[f64], b: &[f64]) -> Matrix<'a> {
    Matrix {
        data: Cow::Owned(
            (0..a.len())
                .flat_map(|r| (0..b.len()).map(move |c| a[r] * b[c]))
                .collect(),
        ),
        shape: (a.len(), b.len()),
        stride: (b.len(), 1),
        offset: (0, 0),
    }
}

fn norm<'a>(a: impl IntoIterator<Item = &'a f64>) -> f64 {
    a.into_iter().map(|a| a * a).sum::<f64>().sqrt()
}

fn matmul<'a>(a: &Matrix, b: &Matrix) -> Matrix<'a> {
    assert_eq!(
        a.shape.1, b.shape.0,
        "Matrix inproperly sized for matmul {}-{}",
        a.shape.1, b.shape.0
    );
    Matrix {
        data: Cow::Owned(
            (0..a.shape.0)
                .flat_map(|ra| (0..b.shape.1).map(move |cb| dot(&a.row(ra), &b.col(cb))))
                .collect(),
        ),
        shape: (a.shape.0, b.shape.1),
        stride: (b.shape.1, 1),
        offset: (0, 0),
    }
}

// Column-space of m describes a space
// When just care about column-space (collection vectors)
// Get a set of vectors that span the same space. I don't really care directions just the space they span, so I'll orthogonalize
// Modified gram Schmidt for better numerical stability
pub fn gram_schmidt_orthonorm(mut m: Matrix) -> Matrix {
    for k in 0..m.shape.1 {
        // take col-k vector remove components shared with other bases
        let uk = (0..k).fold(
            m.col(k).into_iter().cloned().collect::<Vec<_>>(),
            |uk, j| {
                uk.iter()
                    .zip(&proj(&uk, m.col(j)))
                    .map(|(u, vp)| u - vp)
                    .collect()
            },
        );
        // normalize vector
        for r in 0..m.shape.0 {
            m[(r, k)] = uk[r] / dot(&uk, &uk).sqrt();
        }
    }
    m
}

// A householder reflection finds 'v' reflection hyper-plane (perpendicular vector)
// so that vector 'x' when reflected over 'v' is colinear with standard basis e1.
// Find 'v' so that x - 2 * proj-v(x) == ||x|| * e1
pub fn householder_reflector<'a>(x: impl IntoIterator<Item = &'a f64>) -> Vec<f64> {
    let mut v: Vec<_> = x.into_iter().cloned().collect();
    // of possible reflections (signum) choose the largest ||v|| to minimize error.
    v[0] = v[0] + v[0].signum() * v.iter().map(|xi| xi * xi).sum::<f64>().sqrt();
    println!("u = {:?}", v);
    let norm_v = v.iter().map(|vi| vi * vi).sum::<f64>().sqrt();
    v.iter().map(|vi| vi / norm_v).collect()
}

pub fn qr_decompose<'b>(a: &Matrix) -> (Matrix<'b>, Matrix<'b>) {
    // Decompose Amxn -> Qmxm Rmxn
    // for underdeterined systems (infinite solutions) cap r's cols
    let r_cols = std::cmp::min(a.shape.0, a.shape.1);
    let mut r = a.get(0..a.shape.0, 0..r_cols).to_owned();
    let q_size = a.shape.0; // Q is always square.
    let mut q = Matrix::eye(q_size);
    // Apply reflactors to a for each column to derive r
    for c in 0..r.shape.1 {
        let rj = r.get(c..r.shape.0, c..c + 1); // column vector from diagonal to bottom
        let v = householder_reflector(&rj);

        // Apply the reflector to the A sub-matrices resulting in R
        for j in c..r.shape.1 {
            let vdotr = dot(&v, &r.get(c..r.shape.0, j..j + 1));
            // modify r in place iterating from diagonal to end of row
            for i in c..r.shape.0 {
                r[(i, j)] = r[(i, j)] - 2.0 * v[i - c] * vdotr;
            }
        }
        println!("r = {:?}", r);

        // Q should be formed by applying Hi in reverse order.
        // Since Q is symetric orthogonal. Q.t = Qk ... Q2 Q1, Q = Q1 Q2 ... Qk
        // We can build Q in the forward pass and return its transpose
        for j in 0..q.shape.1 {
            let vdotq = dot(&v, &q.get(c..q.shape.0, j..j + 1));
            // modify q in place iterating from diagonal to end of row
            for i in c..q.shape.0 {
                q[(i, j)] = q[(i, j)] - 2.0 * v[i - c] * vdotq;
            }
        }
        println!("q = {:?}", q);
    }
    (q.transpose().to_owned(), r)
}

pub fn nsolve(a: Matrix, b: Vec<f64>) -> Vec<f64> {
    // orthogonalize a via gram schmidt
    // let q_t = gram_schmidt_orthonorm(a.to_owned()).transpose().to_owned();
    // let r = matmul(&q_t, &a);
    let (q, r) = qr_decompose(&a);
    println!("A = {:?}", a);
    println!("Q = {:?}", q);
    println!("R = {:?}", r);
    println!("Q R = {:?}", matmul(&q, &r));
    println!("Q Q.t= {:?}", matmul(&q, &q.transpose()));
    let q_t = q.transpose();
    let c: Vec<_> = (0..q_t.shape.0).map(|r| dot(&q_t.row(r), &b)).collect();
    // we'll have as many unknowns as a as columns
    // if the system is under-determined though some will be left at 0 (c isn't that large)
    let mut x = vec![0.0; a.shape.1];
    let xsize = std::cmp::min(a.shape.0, a.shape.1);

    // r11 r12 r13  x1  c1
    //   0 r22 r23  x2  c2
    //   0   0 r33  x3  c3
    //
    // r33 * x3 = c3                         => x3 = c3 / r33
    // r22 * x2 + r23 * x3 = c2              => x2 = (c2 - r23 * x3) / r22
    // r11 * x1 + r12 * x2 + r13 * x3 = c1   => x1 = (c1 - r12 * x2 - r13 * x3) / r11

    for n in (0..xsize).rev() {
        x[n] = (c[n] - dot(&r.get(n..n + 1, n + 1..r.shape.1), &x[n + 1..])) / r[(n, n)];
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
    fn test_tensor() {
        let m = Matrix::from_rows(vec![
            vec![10.0, -1.0, 3.0, 0.0],
            vec![-1.0, 11.0, -4.0, 3.0],
            vec![2.0, -1.0, 10.0, -1.0],
            vec![0.0, 3.0, -1.0, 8.0],
        ]);
        // index basic ranges
        assert_eq!(*m.col(2).to_owned().data, [3.0, -4.0, 10.0, -1.0]);
        assert_eq!(*m.get(1..2, 0..4).to_owned().data, [-1.0, 11.0, -4.0, 3.0]);
        assert_eq!(
            *m.get(1..3, 1..4).to_owned().data,
            [11.0, -4.0, 3.0, -1.0, 10.0, -1.0]
        );
        assert_eq!(
            *m.get(0..2, 1..4).to_owned().data,
            [-1.0, 3.0, 0.0, 11.0, -4.0, 3.0]
        );
        // index after index
        assert_eq!(
            *m.get(1..3, 1..4).get(1..2, 1..3).to_owned().data,
            [10.0, -1.0],
        );
        // transpose
        assert_eq!(
            *m.get(1..3, 1..4).transpose().to_owned().data,
            [11.0, -1.0, -4.0, 10.0, 3.0, -1.0],
        );
        assert_eq!(
            *m.get(1..4, 1..4)
                .transpose()
                .get(0..2, 1..3)
                .to_owned()
                .data,
            [-1.0, 3.0, 10.0, -1.0],
        );
        assert_eq!(
            *m.get(1..4, 1..4)
                .transpose()
                .get(0..2, 1..3)
                .transpose()
                .to_owned()
                .data,
            [-1.0, 10.0, 3.0, -1.0],
        );
        // indexing
        assert_eq!(
            m.get(1..4, 1..4).transpose().get(0..2, 1..3).transpose()[(1, 0)],
            3.0
        );
        assert_eq!(m.get(1..4, 1..4).transpose()[(1, 0)], -4.0);
    }

    #[test]
    fn test_orthonorm() {
        let m = Matrix::from_rows(vec![
            vec![-1.0, 11.0, -1.0],
            vec![2.0, -20.0, 2.0],
            vec![0.0, 3.0, -1.0],
        ]);
        let onm = gram_schmidt_orthonorm(m);
        println!("gs: {:?}", onm);
        for i in 0..onm.shape.1 {
            println!("norm {}={}", i, norm(&onm.col(i)));
            for j in 0..onm.shape.1 {
                println!("dot {}*{}: {}", i, j, dot(&onm.col(i), &onm.col(j)));
            }
        }
    }

    #[test]
    fn test_qr_decompose() {
        let a = Matrix::from_rows(vec![
            vec![12.0, -51.0, 4.0],
            vec![6.0, 167.0, -68.0],
            vec![-4.0, 24.0, -41.0],
        ]);
        let (q, r) = qr_decompose(&a);
        println!("A = {:?}", a);
        println!("Q = {:?}", q);
        println!("R = {:?}", r);
        println!("Q R = {:?}", matmul(&q, &r));
        println!("Q Q.t= {:?}", matmul(&q, &q.transpose()));
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
            Matrix {
                data: Cow::Owned(vec![1.0, 2.0, 3.0, 4.0, 5.0]),
                shape: (5, 1),
                stride: (1, 1),
                offset: (0, 0),
            },
            vec![2.0, 5.0, 3.0, 8.0, 7.0],
        );
        println!("{:?}", x);

        let x = nsolve(
            Matrix {
                data: Cow::Owned(vec![1.0, 1.0, 2.0, 1.0, 3.0, 1.0, 4.0, 1.0, 5.0, 1.0]),
                shape: (5, 2),
                stride: (2, 1),
                offset: (0, 0),
            },
            vec![2.0, 5.0, 3.0, 8.0, 7.0],
        );
        println!("{:?}", x);
    }

    #[test]
    fn test_householder() {
        let x = vec![4.0, 1.0, -2.0, 2.0];
        let v = householder_reflector(&x);
        println!("v = {:?}", v);

        let udotv = dot(&v, &x);
        println!("udotv = {:?}", udotv);
        let hx: Vec<_> = x
            .iter()
            .zip(&v)
            .map(|(xi, vi)| xi - 2.0 * vi * udotv)
            .collect();
        println!("hx = {:?}", hx);

        let vtv = dot(&v, &v);
        println!("vtv = {:?}", vtv);

        let o = outer(&v, &v);
        println!("o = {:?}", o);

        let oo = &o;
        let h = Matrix {
            data: Cow::Owned(
                (0..o.shape.0)
                    .flat_map(|r| {
                        (0..o.shape.1).map(move |c| {
                            let eye = if r == c { 1.0 } else { 0.0 };
                            eye - 2.0 * oo[(r, c)] // / vtv
                        })
                    })
                    .collect(),
            ),
            shape: o.shape,
            stride: o.stride,
            offset: (0, 0),
        };
        println!("h = {:?}", h);
        let col1 = vec![
            dot(&h.row(0), &x),
            dot(&h.row(1), &x),
            dot(&h.row(2), &x),
            dot(&h.row(3), &x),
        ];
        println!("a1 = {:?}", col1);
        approx_eq(&col1, &vec![-norm(&x), 0.0, 0.0, 0.0]);
    }
}
