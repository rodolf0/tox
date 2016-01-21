use std::collections::HashSet;
use std::ops::Index;
use std::rc::Rc;
use std::{fmt, hash, mem, iter, slice};

pub enum Symbol {
    NonTerm(String),
    Terminal(String, Box<Fn(&str)->bool>),
}

impl Symbol {
    pub fn nonterm<S: Into<String>>(s: S) -> Self { Symbol::NonTerm(s.into()) }

    pub fn terminal<S, F>(name: S, f: F) -> Self
    where S: Into<String>, F: 'static + Fn(&str)->bool {
        Symbol::Terminal(name.into(), Box::new(f))
    }

    pub fn name<'a>(&'a self) -> &'a str {
        match self {
            &Symbol::NonTerm(ref name) => name,
            &Symbol::Terminal(ref name, _) => name,
        }
    }

    pub fn term_match(&self, input: &str) -> bool {
        match self {
            &Symbol::Terminal(_, ref f) => f(input),
            &Symbol::NonTerm(_) => false,
        }
    }

    pub fn is_nonterm(&self) -> bool {
        match self { &Symbol::NonTerm(_) => true, _ => false }
    }

    pub fn is_term(&self) -> bool {
        match self { &Symbol::Terminal(_, _) => true, _ => false }
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Symbol::NonTerm(ref name) => write!(f, "{}", name),
            &Symbol::Terminal(ref name, _) => write!(f, "'{}'", name),
        }
    }
}

impl hash::Hash for Symbol {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            &Symbol::NonTerm(ref name) => name.hash(state),
            &Symbol::Terminal(ref name, ref f) => {
                name.hash(state);
                let (x, y) = unsafe { mem::transmute::<_, (usize, usize)>(&**f) };
                x.hash(state); y.hash(state);
            }
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        match (self, other) {
            (&Symbol::NonTerm(ref a), &Symbol::NonTerm(ref b)) => a == b,
            (&Symbol::Terminal(ref name_a, ref func_a),
             &Symbol::Terminal(ref name_b, ref func_b)) => {
                name_a == name_b && unsafe {
                    let a = mem::transmute::<_, (usize, usize)>(&**func_a);
                    let b = mem::transmute::<_, (usize, usize)>(&**func_b);
                    a == b
                }
            },
            _ => false,
        }
    }
}

impl Eq for Symbol {}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Rule {
    name: Rc<Symbol>,
    spec: Vec<Rc<Symbol>>,
}

impl Rule {
    pub fn new(name: Rc<Symbol>, spec: Vec<Rc<Symbol>>) -> Rule {
        Rule{name: name, spec: spec}
    }

    pub fn name<'a>(&'a self) -> &'a str { self.name.name() }

    pub fn spec(&self) -> String {
        self.spec.iter().map(|s| s.name()).collect::<Vec<_>>().join(" ")
    }

    // TODO: deprecate after nullable symbols re-write
    pub fn nullable(&self, nullset: &HashSet<String>) -> bool {
        self.spec.iter().all(|s| s.is_nonterm() && nullset.contains(s.name()))
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
    rule: Rc<Rule>,
    dot: usize,    // index into the production
    pub start: usize,  // Earley state where item starts
    end: usize,    // Earley state where item ends
    // backpointers to source of this item: (source-item, trigger)
    pub bp: HashSet<(Item, Trigger)>, // TODO: Rc<Item> for less mem
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

// override Hash/Eq to avoid 'bp' from deduplicate Items in StateSets
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

    pub fn next_symbol<'a>(&'a self) -> Option<&'a Symbol> {
        self.rule.spec.get(self.dot).map(|s| &**s)
    }

    pub fn complete(&self) -> bool { self.dot >= self.rule.spec.len() }

    pub fn rule_spec(&self) -> String { self.rule.spec() }

    // check if other item's next-symbol matches our rule's name
    pub fn can_complete(&self, other: &Item) -> bool {
        self.complete() && match other.next_symbol() {
            Some(s) if s.is_nonterm() &&
                       s.name() == self.rule.name() => true,
            _ => false
        }
    }

    // build a new Item for a prediction
    pub fn predict_new(rule: &Rc<Rule>, start: usize) -> Item {
        Item{rule: rule.clone(), dot: 0,
             start: start, end: start, bp: HashSet::new()}
    }

    // use an existing Item as a template for a new one but advance it
    pub fn advance(tpl: &Item, end: usize) -> Item {
        Item{rule: tpl.rule.clone(), dot: tpl.dot+1,
             start: tpl.start, end: end, bp: HashSet::new()}
    }

    // produce an Item after scanning using another item as the base
    pub fn scan_new(source: &Item, end: usize, input: &str) -> Item {
        let mut _bp = HashSet::new();
        _bp.insert((source.clone(), Trigger::Scan(input.to_string())));
        Item{rule: source.rule.clone(), dot: source.dot+1,
             start: source.start, end: end, bp: _bp}
    }

    pub fn complete_new(source: &Item, trigger: &Item, end: usize) -> Item {
        let mut _bp = HashSet::new();
        _bp.insert((source.clone(), Trigger::Completion(trigger.clone())));
        Item{rule: source.rule.clone(), dot: source.dot+1,
             start: source.start, end: end, bp: _bp}
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
    order: Vec<Item>, // TODO: use Rc<Item> for less mem
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

    pub fn iter<'a>(&'a self) -> slice::Iter<'a, Item> { self.order.iter() }

    pub fn filter_by_rule<'a>(&'a self, name: &'a str) ->
           Box<Iterator<Item=&'a Item> + 'a> {
        Box::new(self.order.iter().filter(move |it| it.rule.name() == name))
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
    fn index<'b>(&'b self, idx: usize) -> &'b Item { self.order.index(idx) }
}

impl fmt::Debug for StateSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.order.fmt(f) }
}
