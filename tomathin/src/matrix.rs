use std::borrow::Cow;
use std::ops::{Index, IndexMut, Mul, Range, Sub};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Matrix {
    data: Rc<[f64]>,
    shape: (usize, usize),  // rows, cols
    stride: (usize, usize), // move between rows, cols
    offset: (usize, usize),
}

impl<'a> Matrix {
    pub fn from_rows(rows: Vec<Vec<f64>>) -> Self {
        assert!(rows.len() != 0 && rows[0].len() != 0, "Empty rows or cols");
        let n_rows = rows.len();
        let n_cols = rows[0].len();
        Matrix {
            data: Rc::from_iter(rows.into_iter().flat_map(|v| v)),
            shape: (n_rows, n_cols),
            stride: (n_cols, 1),
            offset: (0, 0),
        }
    }

    pub fn col(&self, col: usize) -> Cow<'_, [f64]> {
        // Moving between rows needs to be contiguous in memory
        if self.stride.0 == 1 {
            let idx = self.offset.0 * self.stride.0 + (self.offset.1 + col) * self.stride.1;
            return Cow::Borrowed(&self.data[idx..idx + self.shape.0]);
        }
        Cow::Owned(
            self.get(0..self.shape.0, col..col + 1)
                .into_iter()
                .cloned()
                .collect(),
        )
    }

    pub fn row(&self, row: usize) -> Cow<'_, [f64]> {
        // Moving between columns needs to be contiguous in memory
        if self.stride.1 == 1 {
            let idx = (self.offset.0 + row) * self.stride.0 + self.offset.1 * self.stride.1;
            return Cow::Borrowed(&self.data[idx..idx + self.shape.1]);
        }
        Cow::Owned(
            self.get(row..row + 1, 0..self.shape.1)
                .into_iter()
                .cloned()
                .collect(),
        )
    }

    pub fn eye(n: usize) -> Matrix {
        Matrix {
            data: Rc::from_iter((0..n * n).map(|i| if i % (n + 1) == 0 { 1.0 } else { 0.0 })),
            shape: (n, n),
            stride: (n, 1),
            offset: (0, 0),
        }
    }
}

impl Matrix {
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
            data: self.data.clone(),
            shape: (row.end - row.start, col.end - col.start),
            stride: self.stride,
            offset: (self.offset.0 + row.start, self.offset.1 + col.start),
        }
    }

    pub fn coalesce(self) -> Matrix {
        if self.offset == (0, 0) && self.stride == (self.shape.1, 1) {
            return self;
        }
        Matrix {
            data: Rc::from_iter(self.into_iter().cloned()),
            offset: (0, 0),
            shape: self.shape,
            stride: (self.shape.1, 1),
        }
    }

    pub fn transpose(&self) -> Matrix {
        Matrix {
            data: self.data.clone(),
            shape: (self.shape.1, self.shape.0),
            stride: (self.stride.1, self.stride.0),
            offset: (self.offset.1, self.offset.0),
        }
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        let idx = (self.offset.0 + row) * self.stride.0 + (self.offset.1 + col) * self.stride.1;
        &self.data[idx]
    }
}

impl<'a> IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        let idx = (self.offset.0 + row) * self.stride.0 + (self.offset.1 + col) * self.stride.1;
        Rc::make_mut(&mut self.data).index_mut(idx)
    }
}

impl<'a> IntoIterator for &'a Matrix {
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

impl Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Matrix) -> Matrix {
        matmul(&self, &rhs)
    }
}

impl Sub<Matrix> for Matrix {
    type Output = Matrix;

    fn sub(self, rhs: Matrix) -> Matrix {
        assert_eq!(self.shape, rhs.shape, "Can't Sub matrix of different size");
        Matrix {
            data: Rc::from_iter(self.data.iter().zip(&*rhs.data).map(|(l, r)| l - r)),
            shape: (self.shape.0, rhs.shape.1),
            stride: (rhs.shape.1, 1),
            offset: (0, 0),
        }
    }
}

fn project(v: &[f64], u: &[f64]) -> Vec<f64> {
    let scale = dot(v, u) / dot(u, u);
    u.into_iter().map(|ui| ui * scale).collect()
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(ai, bi)| ai * bi).sum::<f64>()
}

fn outer<'a>(a: &[f64], b: &[f64]) -> Matrix {
    Matrix {
        data: Rc::from_iter((0..a.len()).flat_map(|r| (0..b.len()).map(move |c| a[r] * b[c]))),
        shape: (a.len(), b.len()),
        stride: (b.len(), 1),
        offset: (0, 0),
    }
}

fn norm<'a>(a: &[f64]) -> f64 {
    a.iter().map(|a| a * a).sum::<f64>().sqrt()
}

fn matmul<'a>(a: &Matrix, b: &Matrix) -> Matrix {
    assert_eq!(
        a.shape.1, b.shape.0,
        "Matrix inproperly sized for matmul {}-{}",
        a.shape.1, b.shape.0
    );
    Matrix {
        data: Rc::from_iter(
            (0..a.shape.0)
                .flat_map(|ra| (0..b.shape.1).map(move |cb| dot(&*a.row(ra), &*b.col(cb)))),
        ),
        shape: (a.shape.0, b.shape.1),
        stride: (b.shape.1, 1),
        offset: (0, 0),
    }
}

// Column-space of m describes a span of space.
// Orthogonalization will span the same space but with orthogonal vectors.
// Modified gram Schmidt for better numerical stability
pub fn gram_schmidt_orthonorm(a: &Matrix) -> Matrix {
    let mut m = a.clone();
    for k in 0..m.shape.1 {
        // take col-k vector remove components shared with other bases
        let uk = (0..k).fold(
            m.col(k).into_iter().cloned().collect::<Vec<_>>(),
            |uk, j| {
                uk.iter()
                    .zip(&project(&uk, &m.col(j)))
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
pub fn householder_reflector<'a>(x: &[f64]) -> Vec<f64> {
    let mut v: Vec<_> = x.iter().cloned().collect();
    // of possible reflections (signum) choose the largest ||v|| to minimize error.
    v[0] = v[0] + v[0].signum() * v.iter().map(|xi| xi * xi).sum::<f64>().sqrt();
    let norm_v = v.iter().map(|vi| vi * vi).sum::<f64>().sqrt();
    v.iter().map(|vi| vi / norm_v).collect()
}

pub fn qr_decompose(a: &Matrix) -> (Matrix, Matrix) {
    // Decompose Amxn -> Qmxm Rmxn
    // for underdeterined systems (infinite solutions) cap r's cols
    let r_cols = std::cmp::min(a.shape.0, a.shape.1);
    let mut r = a.get(0..a.shape.0, 0..r_cols);
    let q_size = a.shape.0; // Q is always square.
    let mut q = Matrix::eye(q_size);
    // Apply reflactors to a for each column to derive r
    for c in 0..r.shape.1 {
        let rj = &r.col(c)[c..]; // column vector from diagonal to bottom
        let v = householder_reflector(rj);

        // Apply the reflector to the A sub-matrices resulting in R
        for j in c..r.shape.1 {
            // let vdotr = dot(&v, &r.get(c..r.shape.0, j..j + 1));
            let vdotr = dot(&v, &r.col(j)[c..]);
            // modify r in place iterating from diagonal to end of row
            for i in c..r.shape.0 {
                r[(i, j)] = r[(i, j)] - 2.0 * v[i - c] * vdotr;
            }
        }
        // Q should be formed by applying Hi in reverse order.
        // Since Q is symetric orthogonal. Q.t = Qk ... Q2 Q1, Q = Q1 Q2 ... Qk
        // We can build Q in the forward pass and return its transpose
        for j in 0..q.shape.1 {
            let vdotq = dot(&v, &q.col(j)[c..]);
            // modify q in place iterating from diagonal to end of row
            for i in c..q.shape.0 {
                q[(i, j)] = q[(i, j)] - 2.0 * v[i - c] * vdotq;
            }
        }
    }
    (q.transpose(), r)
}

pub fn nsolve(a: Matrix, b: Vec<f64>) -> Vec<f64> {
    // orthogonalize a via gram schmidt
    // let q_t = gram_schmidt_orthonorm(&a).transpose();
    // let r = matmul(&q_t, &a);
    let (q, r) = qr_decompose(&a);
    let q_t = q.transpose();
    let c: Vec<_> = (0..q_t.shape.0).map(|r| dot(&*q_t.row(r), &b)).collect();
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
        x[n] = (c[n] - dot(&r.row(n)[n + 1..], &x[n + 1..])) / r[(n, n)];
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
    fn test_matrix_slices() {
        let m = Matrix::from_rows(vec![
            vec![10.0, -1.0, 3.0, 0.0],
            vec![-1.0, 11.0, -4.0, 3.0],
            vec![2.0, -1.0, 10.0, -1.0],
            vec![0.0, 3.0, -1.0, 8.0],
        ]);
        let row2 = m.row(2);
        assert_eq!(*row2, [2.0, -1.0, 10.0, -1.0]);
        unsafe {
            assert_eq!(m.data.as_ptr().add(8), row2.as_ptr());
        }
        let m_t = m.transpose();
        let col2 = m_t.row(2);
        assert_eq!(*col2, [3.0, -4.0, 10.0, -1.0]);
        unsafe {
            let m_end_bound = m.data.as_ptr().add(m.data.len());
            assert!(!(col2.as_ptr() >= m.data.as_ptr() && col2.as_ptr() <= m_end_bound))
        }
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
        assert_eq!(*m.col(2), [3.0, -4.0, 10.0, -1.0]);
        assert_eq!(*m.get(1..2, 0..4).coalesce().data, [-1.0, 11.0, -4.0, 3.0]);
        assert_eq!(
            *m.get(1..3, 1..4).coalesce().data,
            [11.0, -4.0, 3.0, -1.0, 10.0, -1.0]
        );
        assert_eq!(
            *m.get(0..2, 1..4).coalesce().data,
            [-1.0, 3.0, 0.0, 11.0, -4.0, 3.0]
        );
        // index after index
        assert_eq!(
            *m.get(1..3, 1..4).get(1..2, 1..3).coalesce().data,
            [10.0, -1.0],
        );
        // transpose
        assert_eq!(
            *m.get(1..3, 1..4).transpose().coalesce().data,
            [11.0, -1.0, -4.0, 10.0, 3.0, -1.0],
        );
        assert_eq!(
            *m.get(1..4, 1..4)
                .transpose()
                .get(0..2, 1..3)
                .coalesce()
                .data,
            [-1.0, 3.0, 10.0, -1.0],
        );
        assert_eq!(
            *m.get(1..4, 1..4)
                .transpose()
                .get(0..2, 1..3)
                .transpose()
                .coalesce()
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
        let onm = gram_schmidt_orthonorm(&m);
        println!("gs: {:?}", onm);
        for i in 0..onm.shape.1 {
            println!("norm {}={}", i, norm(&*onm.col(i)));
            for j in 0..onm.shape.1 {
                println!("dot {}*{}: {}", i, j, dot(&*onm.col(i), &*onm.col(j)));
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
                data: Rc::from([1.0, 2.0, 3.0, 4.0, 5.0]),
                shape: (5, 1),
                stride: (1, 1),
                offset: (0, 0),
            },
            vec![2.0, 5.0, 3.0, 8.0, 7.0],
        );
        println!("{:?}", x);

        let x = nsolve(
            Matrix {
                data: Rc::from([1.0, 1.0, 2.0, 1.0, 3.0, 1.0, 4.0, 1.0, 5.0, 1.0]),
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

        let vdotx = dot(&v, &x);
        let hx: Vec<_> = x
            .iter()
            .zip(&v)
            .map(|(xi, vi)| xi - 2.0 * vi * vdotx)
            .collect();
        println!("hx = {:?}", hx);

        let o = outer(&v, &v);
        println!("o = {:?}", o);

        let oo = &o;
        let h = Matrix {
            data: Rc::from_iter((0..o.shape.0).flat_map(|r| {
                (0..o.shape.1).map(move |c| {
                    let eye = if r == c { 1.0 } else { 0.0 };
                    eye - 2.0 * oo[(r, c)]
                })
            })),
            shape: o.shape,
            stride: o.stride,
            offset: (0, 0),
        };
        println!("h = {:?}", h);
        let col1 = vec![
            dot(&*h.row(0), &x),
            dot(&*h.row(1), &x),
            dot(&*h.row(2), &x),
            dot(&*h.row(3), &x),
        ];
        println!("a1 = {:?}", col1);
        approx_eq(&col1, &vec![-norm(&x), 0.0, 0.0, 0.0]);
    }
}
