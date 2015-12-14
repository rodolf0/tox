use earley::symbol::Symbol;
use std::collections::HashSet;
use std::ops::Index;
use std::rc::Rc;
use std::{slice, iter, fmt};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Rule {
    pub name: Rc<Symbol>,
    pub spec: Vec<Rc<Symbol>>,
}

impl Rule {
    pub fn new(name: Rc<Symbol>, spec: Vec<Rc<Symbol>>) -> Rule {
        Rule{name: name, spec: spec}
    }

    pub fn name<'a>(&'a self) -> &'a str { self.name.name() }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Item {
    pub rule: Rc<Rule>,
    pub dot: usize,    // index into the production
    pub start: usize,  // Earley state where this item starts
}

impl Item {
    pub fn new(rule: Rc<Rule>, dot: usize, start: usize) -> Item {
        Item{rule: rule, dot: dot, start: start}
    }

    pub fn next_symbol<'a>(&'a self) -> Option<&'a Symbol> {
        self.rule.spec.get(self.dot).map(|s| &**s)
    }

    pub fn complete(&self) -> bool {
        self.dot >= self.rule.spec.len()
    }
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pre = self.rule.spec.iter()
            .take(self.dot).map(|s| s.name()).collect::<Vec<&str>>().join(" ");
        let post = self.rule.spec.iter()
            .skip(self.dot).map(|s| s.name()).collect::<Vec<&str>>().join(" ");
        write!(f, "({}) {:10} -> {} \u{00b7} {}", self.start, self.rule.name(), pre, post)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct StateSet {
    order: Vec<Item>,
    dedup: HashSet<Item>,
}

impl StateSet {
    pub fn new() -> StateSet {
        StateSet{order: Vec::new(), dedup: HashSet::new()}
    }

    pub fn push(&mut self, item: Item) {
        if !self.dedup.contains(&item) {
            self.order.push(item.clone());
            self.dedup.insert(item);
        }
    }

    pub fn len(&self) -> usize { self.dedup.len() }

    pub fn iter<'a>(&'a self) -> slice::Iter<'a, Item> {
        self.order.iter()
    }
}

impl Extend<Item> for StateSet {
    fn extend<I: IntoIterator<Item=Item>>(&mut self, iterable: I) {
        for item in iterable { self.push(item); }
    }
}

impl iter::FromIterator<Item> for StateSet {
    fn from_iter<I: IntoIterator<Item=Item>>(iterable: I) -> Self {
        let mut ss = StateSet::new();
        ss.extend(iterable.into_iter());
        ss
    }
}

impl Index<usize> for StateSet {
    type Output = Item;
    fn index<'b>(&'b self, idx: usize) -> &'b Item {
        self.order.index(idx)
    }
}

impl fmt::Debug for StateSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.order.fmt(f)
    }
}
