use std::ops::{Index, IndexMut};

#[derive(Debug)]
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
        }
    }

    pub fn row(&self, row: usize) -> MatrixView {
        MatrixView {
            m: self,
            offset: self.cols * row,
            stride: 1,
            limit: self.cols,
        }
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        assert!(row < self.rows, "Index out of bounds");
        assert!(col < self.cols, "Index out of bounds");
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
        &self.m.data[self.offset + self.stride * idx]
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
    // pub fn dot(&self, other: MatrixView) -> f64 {
    //     assert!(
    //         self.limit == other.limit,
    //         "Dot requires views of equal length"
    //     );
    //     self.into_iter()
    //         .zip(other.into_iter())
    //         .map(|(s, o)| s * o)
    //         .sum::<f64>()
    // }

    // pub fn norm(&self) -> f64 {
    //     self.into_iter().map(|n| n * n).sum::<f64>().sqrt()
    // }

    pub fn vec(&self) -> Vec<f64> {
        self.into_iter().cloned().collect()
    }
}

fn proj(v: &Vec<f64>, u: MatrixView) -> Vec<f64> {
    let scale = dot(v, &u) / dot(&u, &u);
    u.into_iter().map(|ui| ui * scale).collect()
}

// fn dot(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
//     a.iter().zip(b.iter()).map(|(ai, bi)| ai * bi).sum::<f64>()
// }

fn dot<'a>(a: impl IntoIterator<Item = &'a f64>, b: impl IntoIterator<Item = &'a f64>) -> f64 {
    a.into_iter()
        .zip(b.into_iter())
        .map(|(ai, bi)| ai * bi)
        .sum::<f64>()
}

fn norm<'a>(a: impl IntoIterator<Item = &'a f64>) -> f64 {
    a.into_iter().map(|a| a * a).sum::<f64>().sqrt()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix() {
        let m = Matrix::from_rows(vec![
            vec![10.0, -1.0, 2.0, 0.0],
            vec![-1.0, 11.0, -1.0, 3.0],
            vec![2.0, -1.0, 10.0, -1.0],
            vec![0.0, 3.0, -1.0, 8.0],
        ]);
        println!("{:?}", m);
        let c2 = m.col(3);
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
}
