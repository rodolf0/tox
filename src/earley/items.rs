use earley::symbol::Symbol;
use std::collections::HashSet;
use std::ops::Index;
use std::rc::Rc;
use std::{fmt, hash, iter, slice};

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

    pub fn spec(&self) -> String {
        self.spec.iter().map(|s| s.name()).collect::<Vec<_>>().join(" ")
    }
}

///////////////////////////////////////////////////////////////////////////////
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Trigger {
    Completion(Item),
    Scan(String),
}

#[derive(Clone)]
pub struct Item {
    pub rule: Rc<Rule>,
    pub dot: usize,    // index into the production
    pub start: usize,  // Earley state where item starts
    pub end: usize,    // Earley state where item ends
    // backpointers to source of this item: (source-item, trigger)
    pub bp: HashSet<(Item, Trigger)>, // TODO: indexes into some table
}

// override Hash/Eq to avoid 'bp' from deduplicate Items in StateSets
impl hash::Hash for Item {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.dot.hash(state);
        self.start.hash(state);
        self.end.hash(state);
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.rule == other.rule && self.dot == other.dot &&
        self.start == other.start && self.end == other.end
    }
}

impl Eq for Item {}

impl Item {
    pub fn new(rule: Rc<Rule>, dot: usize, start: usize, end: usize) -> Item {
        Item{rule: rule, dot: dot, start: start, end: end, bp: HashSet::new()}
    }

    pub fn new2(rule: Rc<Rule>, dot: usize, start: usize, end: usize,
                bp: (Item, Trigger)) -> Item {
        let mut _bp = HashSet::new();
        _bp.insert(bp);
        Item{rule: rule, dot: dot, start: start, end: end, bp: _bp}
    }

    pub fn next_symbol<'a>(&'a self) -> Option<&'a Symbol> {
        self.rule.spec.get(self.dot).map(|s| &**s)
    }

    pub fn complete(&self) -> bool {
        self.dot >= self.rule.spec.len()
    }

    // check if item is complete and rule name matches <name>
    pub fn completes(&self, name: &str) -> bool {
        self.complete() && self.rule.name() == name
    }
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pre = self.rule.spec.iter()
            .take(self.dot).map(|s| s.name()).collect::<Vec<_>>().join(" ");
        let post = self.rule.spec.iter()
            .skip(self.dot).map(|s| s.name()).collect::<Vec<_>>().join(" ");
        write!(f, "({} - {}) {} -> {} \u{00b7} {} # {:?}",
               self.start, self.end, self.rule.name(), pre, post, self.bp)
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

    // push items into the set, merging back-pointer sets
    pub fn push(&mut self, item: Item) {
        if self.dedup.contains(&item) {
            self.dedup.remove(&item);
            let i = self.order.iter().position(|it| *it == item);
            let mut updated = self.order.get_mut(i.unwrap()).unwrap();
            updated.bp.extend(item.bp.into_iter());
            self.dedup.insert(updated.clone());
        } else {
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
