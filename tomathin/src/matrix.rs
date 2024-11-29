use std::borrow::Cow;
use std::ops::{Index, IndexMut, Range};

#[derive(Debug, Clone)]
pub struct Tensor<'a> {
    data: Cow<'a, [f64]>,
    shape: (usize, usize),  // rows, cols
    stride: (usize, usize), // move between rows, cols
    offset: (usize, usize),
}

impl<'a> Tensor<'a> {
    pub fn from_rows(rows: Vec<Vec<f64>>) -> Self {
        assert!(rows.len() != 0 && rows[0].len() != 0, "Empty rows or cols");
        let n_rows = rows.len();
        let n_cols = rows[0].len();
        Tensor {
            data: Cow::from(rows.into_iter().flat_map(|v| v).collect::<Vec<_>>()),
            shape: (n_rows, n_cols),
            stride: (n_cols, 1),
            offset: (0, 0),
        }
    }

    pub fn col(&self, col: usize) -> Tensor {
        Tensor {
            // offset data view to column
            data: Cow::Borrowed(&self.data),
            shape: (self.shape.0, 1), // rows x 1
            stride: self.stride,
            offset: (0, col),
        }
    }
}

impl<'a> Tensor<'a> {
    pub fn get(&self, row: Range<usize>, col: Range<usize>) -> Tensor {
        Tensor {
            data: Cow::Borrowed(&self.data),
            shape: (row.end - row.start, col.end - col.start),
            stride: self.stride,
            offset: (self.offset.0 + row.start, self.offset.1 + col.start),
        }
    }

    pub fn to_owned(&self) -> Tensor {
        Tensor {
            data: Cow::Owned(
                (0..self.shape.0)
                    .flat_map(|r| {
                        self.data
                            .iter()
                            .skip(
                                (self.offset.0 + r) * self.stride.0 + self.offset.1 * self.stride.1,
                            )
                            .step_by(self.stride.1)
                            .take(self.shape.1)
                    })
                    .cloned()
                    .collect(),
            ),
            offset: (0, 0),
            shape: self.shape,
            stride: (self.shape.1, 1),
        }
    }

    pub fn transpose(&self) -> Tensor {
        Tensor {
            data: Cow::Borrowed(&self.data),
            shape: (self.shape.1, self.shape.0),
            stride: (self.stride.1, self.stride.0),
            offset: (self.offset.1, self.offset.0),
        }
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        let idx = (self.offset.0 + row) * self.stride.0 + (self.offset.1 + col) * self.stride.1);
        self.data[idx]
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn test_tensor() {
        let m = Tensor::from_rows(vec![
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
    }
}

#[derive(Debug, Clone)]
pub struct Matrix {
    data: Vec<f64>,
    cols: usize,
    rows: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct MatrixView<'a> {
    m: &'a Matrix,
    offset: usize,
    stride: usize,
    limit: usize,
    skip: usize,
}

impl Matrix {
    pub fn from_rows(rows: Vec<Vec<f64>>) -> Self {
        assert!(rows.len() != 0 && rows[0].len() != 0, "Empty rows or cols");
        let n_rows = rows.len();
        let n_cols = rows[0].len();
        Matrix {
            data: rows.into_iter().flat_map(|v| v).collect(),
            rows: n_rows,
            cols: n_cols,
        }
    }

    pub fn col(&self, col: usize) -> MatrixView {
        MatrixView {
            m: self,
            offset: col % self.cols,
            stride: self.cols,
            limit: self.rows,
            skip: 0,
        }
    }

    pub fn row(&self, row: usize) -> MatrixView {
        MatrixView {
            m: self,
            offset: self.cols * row,
            stride: 1,
            limit: self.cols,
            skip: 0,
        }
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        assert!(row < self.rows, "Index out of bounds {} {}", row, self.rows);
        assert!(col < self.cols, "Index out of bounds {} {}", col, self.cols);
        &self.data[self.cols * row + col]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        assert!(row < self.rows, "Index out of bounds");
        assert!(col < self.cols, "Index out of bounds");
        &mut self.data[self.cols * row + col]
    }
}

impl<'a> Index<usize> for MatrixView<'a> {
    type Output = f64;

    fn index(&self, idx: usize) -> &Self::Output {
        assert!(idx < self.limit, "Index out of bounds");
        &self.m.data[self.offset + self.stride * (idx + self.skip)]
    }
}

impl<'a> MatrixView<'a> {
    pub fn range(&self, range: Range<usize>) -> MatrixView<'a> {
        let start = std::cmp::min(self.limit, range.start);
        MatrixView {
            m: &self.m,
            offset: self.offset,
            stride: self.stride,
            limit: self.limit - start,
            skip: start,
        }
    }
}

impl<'a> IntoIterator for &'a MatrixView<'a> {
    type Item = &'a f64;
    type IntoIter = MatrixViewIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MatrixViewIterator { mv: &self, idx: 0 }
    }
}

pub struct MatrixViewIterator<'a> {
    mv: &'a MatrixView<'a>,
    idx: usize,
}

impl<'a> Iterator for MatrixViewIterator<'a> {
    type Item = &'a f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.mv.limit {
            self.idx += 1;
            return Some(&self.mv[self.idx - 1]);
        }
        None
    }
}

impl<'a> MatrixView<'a> {
    pub fn vec(&self) -> Vec<f64> {
        self.into_iter().cloned().collect()
    }
}

fn proj(v: &Vec<f64>, u: MatrixView) -> Vec<f64> {
    let scale = dot(v, &u) / dot(&u, &u);
    u.into_iter().map(|ui| ui * scale).collect()
}

fn dot<'a>(a: impl IntoIterator<Item = &'a f64>, b: impl IntoIterator<Item = &'a f64>) -> f64 {
    a.into_iter()
        .zip(b.into_iter())
        .map(|(ai, bi)| ai * bi)
        .sum::<f64>()
}

fn outer<'a>(a: &[f64], b: &[f64]) -> Matrix {
    Matrix {
        data: (0..a.len())
            .flat_map(|r| (0..b.len()).map(move |c| a[r] * b[c]))
            .collect(),
        rows: a.len(),
        cols: b.len(),
    }
}

fn norm<'a>(a: impl IntoIterator<Item = &'a f64>) -> f64 {
    a.into_iter().map(|a| a * a).sum::<f64>().sqrt()
}

// fn matmap<'a, ReduceFn>(a: &Matrix, b: &Matrix, reduce: ReduceFn) -> Matrix
// where
//     ReduceFn: Fn(Iterator<Item = &'a f64>, Iterator<Item = &'a f64>) -> f64,
// {
//     Matrix {
//         data: (0..a.rows)
//             .flat_map(|ra| {
//                 (0..b.cols).map(move |cb| reduce(a.row(ra).into_iter(), &b.col(cb).into_iter()))
//             })
//             .collect(),
//         rows: a.rows,
//         cols: b.cols,
//     }
// }

fn transpose(a: Matrix) -> Matrix {
    Matrix {
        data: (0..a.cols)
            .flat_map(|c| a.col(c).into_iter().cloned().collect::<Vec<_>>())
            .collect(),
        rows: a.cols,
        cols: a.rows,
    }
}

fn matmul(a: &Matrix, b: &Matrix) -> Matrix {
    Matrix {
        data: (0..a.rows)
            .flat_map(|ra| (0..b.cols).map(move |cb| dot(&a.row(ra), &b.col(cb))))
            .collect(),
        rows: a.rows,
        cols: b.cols,
    }
}

// Column-space of m describes a space
// When just care about column-space (collection vectors)
// Get a set of vectors that span the same space. I don't really care directions just the space they span, so I'll orthogonalize
// Modified gram Schmidt for better numerical stability
pub fn gram_schmidt_orthonorm(mut m: Matrix) -> Matrix {
    for k in 0..m.cols {
        // take col-k vector remove components shared with other bases
        let uk = (0..k).fold(m.col(k).vec(), |uk, j| {
            uk.iter()
                .zip(proj(&uk, m.col(j)).into_iter())
                .map(|(u, vp)| u - vp)
                .collect()
        });
        // normalize vector
        for r in 0..m.rows {
            m[(r, k)] = uk[r] / dot(&uk, &uk).sqrt();
        }
    }
    m
}

// A householder reflection finds 'v' reflection hyper-plane (perpendicular vector)
// so that // vector 'x' when reflected over 'v' is colinear with standard basis e1.
// Find 'v' so that x - 2 * proj-v(x) == ||x|| * e1
pub fn householder_reflector(x: &[f64]) -> Vec<f64> {
    let mut v = x.to_owned();
    // of possible reflections (signum) choose the largest v to minimize error.
    v[0] = x[0] + x[0].signum() * x.iter().map(|xi| xi * xi).sum::<f64>().sqrt();
    let norm_v = v.iter().map(|vi| vi * vi).sum::<f64>().sqrt();
    v.iter().map(|vi| vi / norm_v).collect()
}

pub fn nsolve(a: Matrix, b: Vec<f64>) -> Vec<f64> {
    // orthogonalize a via gram schmidt
    let a = &a;
    let q = &gram_schmidt_orthonorm(a.clone());
    // let r = matmul(&transpose(q.clone()), a);
    let r = Matrix {
        data: (0..a.rows)
            .flat_map(|r| {
                (0..a.cols).map(move |c| {
                    if c >= r {
                        dot(&a.col(c), &q.col(r))
                    } else {
                        0.0
                    }
                })
            })
            .collect(),
        cols: a.cols,
        rows: a.rows,
    };
    let c: Vec<_> = (0..q.rows).map(|r| dot(&q.col(r), &b)).collect();
    // we'll have as many unknowns as a as columns
    // if the system is under-determined though some will be left at 0 (c isn't that large)
    let mut x = vec![0.0; a.cols];
    let xsize = std::cmp::min(a.cols, a.rows);

    // r00 r01 r02 r03  x0  c0
    //   0 r11 r12 r13  x1  c1
    //   0   0 r22 r23  x2  c2
    //   0   0   0 r33  x3  c3

    // r33 * x3 = c3                         => x3 = c3 / r33
    // r22 * x2 + r23 * x3 = c2              => x2 = (c2 - r23 * x3) / r22
    // r11 * x1 + r12 * x2 + r13 * x3 = c1   => x1 = (c1 - r12 * x2 - r13 * x3) / r11

    for n in (0..xsize).rev() {
        x[n] = (c[n] - dot(&r.row(n).range(n + 1..r.cols), &x[n + 1..])) / r[(n, n)];
    }
    x
}

pub fn eigen_values(a: Matrix) {
    let mut a = a.clone();
    for _ in 0..100 {
        let q = gram_schmidt_orthonorm(a.clone());
        let r = matmul(&a, &q);
        a = matmul(&r, &q);
    }
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
    fn test_matrix() {
        let m = Matrix::from_rows(vec![
            vec![10.0, -1.0, 2.0, 0.0],
            vec![-1.0, 11.0, -4.0, 3.0],
            vec![2.0, -1.0, 10.0, -1.0],
            vec![0.0, 3.0, -1.0, 8.0],
        ]);
        println!("{:?}", m);
        let c2 = m.col(2);
        println!("{:?} {:?} {:?} {:?}", c2[0], c2[1], c2[2], c2[3]);

        let r2 = m.row(2);
        println!("{:?} {:?} {:?} {:?}", r2[0], r2[1], r2[2], r2[3]);

        println!("dot: {:?}", dot(&c2, &r2));

        let onm = gram_schmidt_orthonorm(m);
        println!("gs: {:?}", onm);
        for i in 0..onm.cols {
            println!("norm {}={}", i, norm(&onm.col(i)));
            for j in 0..onm.cols {
                println!("dot {}*{}: {}", i, j, dot(&onm.col(i), &onm.col(j)));
            }
        }
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
        for i in 0..onm.cols {
            println!("norm {}={}", i, norm(&onm.col(i)));
            for j in 0..onm.cols {
                println!("dot {}*{}: {}", i, j, dot(&onm.col(i), &onm.col(j)));
            }
        }
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
                data: vec![1.0, 2.0, 3.0, 4.0, 5.0],
                cols: 1,
                rows: 5,
            },
            vec![2.0, 5.0, 3.0, 8.0, 7.0],
        );
        println!("{:?}", x);

        let x = nsolve(
            Matrix {
                data: vec![1.0, 1.0, 2.0, 1.0, 3.0, 1.0, 4.0, 1.0, 5.0, 1.0],
                cols: 2,
                rows: 5,
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
            data: (0..o.rows)
                .flat_map(|r| {
                    (0..o.cols).map(move |c| {
                        let eye = if r == c { 1.0 } else { 0.0 };
                        eye - 2.0 * oo[(r, c)] // / vtv
                    })
                })
                .collect(),
            rows: o.rows,
            cols: o.cols,
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
