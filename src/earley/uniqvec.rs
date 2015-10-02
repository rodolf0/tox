use std::collections::HashSet;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::Index;
use std::rc::Rc;


pub struct UniqedVec<T> {
    order: Vec<Rc<T>>,
    dedup: HashSet<Rc<T>>,
}

impl<T: Hash + Eq> UniqedVec<T> {
    pub fn new() -> UniqedVec<T> {
        UniqedVec{order: Vec::new(), dedup: HashSet::new()}
    }

    pub fn push(&mut self, item: T) {
        let val = Rc::new(item);
        if !self.dedup.contains(&val) {
            self.order.push(val.clone());
            self.dedup.insert(val);
        }
    }

    pub fn len(&self) -> usize {
        self.dedup.len()
    }
}

impl<T: Hash + Eq> Index<usize> for UniqedVec<T> {
    type Output = T;
    fn index<'b>(&'b self, idx: usize) -> &'b T {
        self.order.index(idx)
    }
}

impl<T: Hash + Eq> Extend<T> for UniqedVec<T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<T: Hash + Eq> FromIterator<T> for UniqedVec<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut uniquedvec = UniqedVec::new();
        uniquedvec.extend(iter.into_iter());
        uniquedvec
    }
}

/*
impl<'a, T: Hash + Eq> IntoIterator for &'a UniqedVec<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;
    fn into_iter(self) -> slice::Iter<'a, T> { self.order.into_iter() }
}
*/
