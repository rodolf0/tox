pub struct ProductIterator {
    idx: Vec<usize>,
    lengths: Vec<usize>,
}

impl ProductIterator {
    pub fn new(lengths: Vec<usize>) -> Self {
        ProductIterator {
            idx: Vec::new(),
            lengths,
        }
    }
}

impl Iterator for ProductIterator {
    // Tuple of output dimensions
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx.len() != self.lengths.len() {
            self.idx = vec![0; self.lengths.len()];
            return Some(self.idx.clone());
        }
        for id in (0..self.lengths.len()).rev() {
            if self.idx[id] + 1 >= self.lengths[id] {
                self.idx[id] = 0;
            } else {
                self.idx[id] += 1;
                return Some(self.idx.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_iterator() {
        let items: Vec<_> = ProductIterator::new(vec![3, 2, 3]).collect();
        assert_eq!(
            items,
            vec![
                vec![0, 0, 0],
                vec![0, 0, 1],
                vec![0, 0, 2],
                vec![0, 1, 0],
                vec![0, 1, 1],
                vec![0, 1, 2],
                vec![1, 0, 0],
                vec![1, 0, 1],
                vec![1, 0, 2],
                vec![1, 1, 0],
                vec![1, 1, 1],
                vec![1, 1, 2],
                vec![2, 0, 0],
                vec![2, 0, 1],
                vec![2, 0, 2],
                vec![2, 1, 0],
                vec![2, 1, 1],
                vec![2, 1, 2]
            ]
        );
    }
}
