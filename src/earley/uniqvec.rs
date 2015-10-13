use std::collections::HashSet;
use std::hash::Hash;
use std::iter::FromIterator;
use std::ops::Index;
use std::rc::Rc;
use std::slice;
use std::iter;

// checkout https://github.com/contain-rs/linked-hash-map
// Could potentially replace Rc<T> with
// struct Elem<T> { e: *const T } at the cost of extra complexity

#[derive(Clone)]
pub struct UniqVec<T> {
    order: Vec<Rc<T>>,
    dedup: HashSet<Rc<T>>,
}

impl<T: Hash + Eq> UniqVec<T> {
    pub fn new() -> UniqVec<T> {
        UniqVec{order: Vec::new(), dedup: HashSet::new()}
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

    pub fn iter<'a>(&'a self) ->
    iter::Map<slice::Iter<'a, Rc<T>>, fn(&'a Rc<T>) -> &'a T> {
        fn f<X>(e: &Rc<X>) -> &X {&**e};
        self.order.iter().map(f)
    }
}

impl<T: Clone> UniqVec<T> {
    pub fn to_vec(&self) -> Vec<T> {
        self.order.iter().map(|e| (**e).clone()).collect()
    }
}


impl<T: Hash + Eq> Index<usize> for UniqVec<T> {
    type Output = T;
    fn index<'b>(&'b self, idx: usize) -> &'b T {
        self.order.index(idx)
    }
}

impl<T: Hash + Eq> Extend<T> for UniqVec<T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iterable: I) {
        for item in iterable { self.push(item); }
    }
}

impl<T: Hash + Eq> FromIterator<T> for UniqVec<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iterable: I) -> Self {
        let mut uniquedvec = UniqVec::new();
        uniquedvec.extend(iterable.into_iter());
        uniquedvec
    }
}
