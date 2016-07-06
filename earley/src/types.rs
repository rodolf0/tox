use std::collections::{HashSet, HashMap};
use std::ops::Index;
use std::{fmt, hash, iter, slice};
use std::rc::Rc;
use std::cell::RefCell;

pub enum Symbol {
    NonTerm(String),
    Terminal(String, Box<Fn(&str)->bool>),
}

impl Symbol {
    pub fn name(&self) -> String {
        match self {
            &Symbol::NonTerm(ref name) => name.clone(),
            &Symbol::Terminal(ref name, _) => name.clone(),
        }
    }
}

impl<'a> From<&'a str> for Symbol {
    fn from(from: &str) -> Self { Symbol::NonTerm(from.to_string()) }
}

impl<'a, F> From<(&'a str, F)> for Symbol
        where F: 'static + Fn(&str)->bool {
    fn from(from: (&str, F)) -> Self {
        Symbol::Terminal(from.0.to_string(), Box::new(from.1))
    }
}

impl hash::Hash for Symbol {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            &Symbol::NonTerm(ref name) => name.hash(state),
            &Symbol::Terminal(ref name, _) => name.hash(state)
        }
    }
}
impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        match (self, other) {
            (&Symbol::NonTerm(ref a), &Symbol::NonTerm(ref b)) => a == b,
            (&Symbol::Terminal(ref a, _), &Symbol::Terminal(ref b, _)) => a == b,
            _ => false,
        }
    }
}
impl Eq for Symbol {}

///////////////////////////////////////////////////////////////////////////////

#[derive(Hash, PartialEq, Eq)]
struct Rule {
    name: String,
    spec: Vec<Rc<Symbol>>,
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Trigger {
    Completion(Rc<Item>),
    Scan(String),
}

#[derive(Clone)]
pub struct Item {
    rule: Rc<Rule>,
    dot: usize,    // index into the production
    start: usize,  // Earley state where item starts
    end: usize,    // Earley state where item ends (needed for dedup)
    // backpointers to source of this item: (source-item, trigger)
    bp: RefCell<HashSet<(Rc<Item>, Trigger)>>,
}

// Items are deduped only by rule, dot, start, end (ie: not bp)
impl hash::Hash for Item {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.dot.hash(state);
        self.start.hash(state);
        self.end.hash(state);
    }
}

// Items are deduped only by rule, dot, start, end (ie: not bp)
impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.rule == other.rule && self.dot == other.dot &&
        self.start == other.start && self.end == other.end
    }
}
impl Eq for Item {}

impl Item {
    pub fn start(&self) -> usize { self.start }
    pub fn complete(&self) -> bool { self.dot >= self.rule.spec.len() }
    pub fn str_rule(&self) -> String {
        format!("{} -> {}", self.rule.name, self.rule.spec.iter().map(
                |s| s.name()).collect::<Vec<_>>().join(" "))
    }
    pub fn next_symbol<'a>(&'a self) -> Option<&'a Symbol> {
        self.rule.spec.get(self.dot).map(|s| &**s)
    }
    pub fn back_pointers(&self) -> HashSet<(Rc<Item>, Trigger)> {
        self.bp.borrow().clone()
    }
}

impl Item {
    // check if other item's next-symbol matches our rule's name
    fn can_complete(&self, other: &Rc<Item>) -> bool {
        self.complete() && match other.next_symbol() {
            Some(&Symbol::NonTerm(ref name)) => *name == self.rule.name,
            _ => false
        }
    }
    // check item's next symbol is a temrinal that scans lexeme
    fn can_scan(&self, lexeme: &str) -> bool {
        match self.next_symbol() {
            Some(&Symbol::Terminal(_, ref f)) => f(lexeme),
            _ => false
        }
    }
    // build a new Item for a prediction
    fn predict_new(rule: &Rc<Rule>, start: usize) -> Item {
        Item{rule: rule.clone(), dot: 0, start: start, end: start,
             bp: RefCell::new(HashSet::new())}
    }
    // produce an Item after scanning using another item as the base
    fn scan_new(source: &Rc<Item>, end: usize, input: &str) -> Item {
        let mut _bp = HashSet::new();
        _bp.insert((source.clone(), Trigger::Scan(input.to_string())));
        Item{rule: source.rule.clone(), dot: source.dot+1,
             start: source.start, end: end, bp: RefCell::new(_bp)}
    }
    // build a new item completing another one
    fn complete_new(source: &Rc<Item>, trigger: &Rc<Item>, end: usize) -> Item {
        let mut _bp = HashSet::new();
        _bp.insert((source.clone(), Trigger::Completion(trigger.clone())));
        Item{rule: source.rule.clone(), dot: source.dot+1,
             start: source.start, end: end, bp: RefCell::new(_bp)}
    }
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pre = self.rule.spec.iter()
            .take(self.dot).map(|s| s.name()).collect::<Vec<_>>().join(" ");
        let post = self.rule.spec.iter()
            .skip(self.dot).map(|s| s.name()).collect::<Vec<_>>().join(" ");
        write!(f, "({} - {}) {} -> {} \u{00b7} {} #bp: {}",
               self.start, self.end, self.rule.name, pre, post,
               self.bp.borrow().len())
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct StateSet {
    order: Vec<Rc<Item>>,
    dedup: HashSet<Rc<Item>>,
}

// Statesets are filled with Item's via push/extend. These are boxed to share BP
impl StateSet {
    pub fn new() -> StateSet {
        StateSet{order: Vec::new(), dedup: HashSet::new()}
    }

    // push items into the set, merging back-pointer sets
    fn push(&mut self, item: Item) {
        if let Some(existent) = self.dedup.get(&item) {
            existent.bp.borrow_mut().extend(item.bp.into_inner());
            return;
        }
        let item = Rc::new(item);
        self.order.push(item.clone());
        self.dedup.insert(item);
    }

    pub fn len(&self) -> usize { self.dedup.len() }

    pub fn iter<'a>(&'a self) -> slice::Iter<'a, Rc<Item>> { self.order.iter() }

    // get all items whose rule name is 'name'
    pub fn filter_by_rule<'a, S: Into<String>>(&'a self, name: S) ->
           Box<Iterator<Item=&'a Rc<Item>> + 'a> {
        let name = name.into();
        Box::new(self.order.iter().filter(move |it| it.rule.name == name))
    }

    pub fn completed_by(&self, item: &Rc<Item>, at: usize) -> Vec<Item> {
        self.order.iter()
            .filter(|source| item.can_complete(source))
            .map(|source| Item::complete_new(source, item, at))
            .collect::<Vec<_>>()
    }

    pub fn advanced_by_scan(&self, lexeme: &str, end: usize) -> Vec<Item> {
        self.order.iter()
            .filter(|item| item.can_scan(lexeme))
            .map(|item| Item::scan_new(item, end, lexeme))
            .collect::<Vec<_>>()
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
    type Output = Rc<Item>;
    fn index<'b>(&'b self, idx: usize) -> &'b Rc<Item> { self.order.index(idx) }
}

impl fmt::Debug for StateSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.order.fmt(f) }
}

///////////////////////////////////////////////////////////////////////////////

pub struct Grammar {
    start: String,
    rules: Vec<Rc<Rule>>,
}

impl Grammar {
    // grammar's start symbol
    pub fn start(&self) -> String { self.start.clone() }

    pub fn predict_new(&self, name: &str, state_idx: usize) -> Vec<Item> {
        self.rules.iter()
            .filter(|r| r.name == name)
            .map(|r| Item::predict_new(r, state_idx))
            .collect()
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct GrammarBuilder {
    symbols: HashMap<String, Rc<Symbol>>,
    rules: Vec<Rc<Rule>>,
}

impl GrammarBuilder {
    pub fn new() -> GrammarBuilder {
        GrammarBuilder{ symbols: HashMap::new(), rules: Vec::new()}
    }

    pub fn symbol<S: Into<Symbol>>(&mut self, symbol: S) -> &mut Self {
        let symbol = symbol.into();
        self.symbols.insert(symbol.name(), Rc::new(symbol));
        self
    }

    pub fn rule<N>(&mut self, name: N, spec: &[N]) -> &mut Self
            where N: Into<String> + AsRef<str> {
        let rule = Rule{
            name: name.into(),
            spec: spec.into_iter().map(|s| match self.symbols.get(s.as_ref()) {
                Some(s) => s.clone(),
                None => panic!("Missing symbol: {}", s.as_ref())
            }).collect()
        };
        self.rules.push(Rc::new(rule));
        self
    }

    pub fn into_grammar<S: Into<String>>(self, start: S) -> Grammar {
        let start = start.into();
        match self.symbols.contains_key(&start) {
            true => Grammar{start: start, rules: self.rules},
            false => panic!("Missing symbol: {}", start),
        }
    }
}


///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::collections::HashSet;
    use std::cell::RefCell;
    use super::{Rule, Item, Symbol, StateSet};

    fn item(rule: Rc<Rule>, dot: usize, start: usize, end: usize) -> Item {
        Item{rule: rule, dot: dot, start: start, end: end,
             bp: RefCell::new(HashSet::new())}
    }

    #[test]
    fn item_dedupness() {
        fn testfn(o: &str) -> bool { o.len() == 1 && "+-".contains(o) }
        let rule = Rc::new(Rule{name: "S".to_string(), spec: vec![
                Rc::new(Symbol::from("S")),
                Rc::new(Symbol::from(("+-", testfn))),
                Rc::new(Symbol::from(("[0-9]", |n: &str|
                                 n.chars().all(|c| "1234567890".contains(c))))),
        ]});
        // test item comparison
        assert_eq!(item(rule.clone(), 0, 0, 0), item(rule.clone(), 0, 0, 0));
        assert!(item(rule.clone(), 0, 0, 0) != item(rule.clone(), 0, 1, 0));
        //check that items are deduped in statesets
        let mut ss = StateSet::new();
        ss.push(item(rule.clone(), 0, 0, 0));
        ss.push(item(rule.clone(), 0, 0, 0));
        assert_eq!(ss.len(), 1);
        ss.push(item(rule.clone(), 1, 0, 1));
        assert_eq!(ss.len(), 2);
        ss.push(item(rule.clone(), 1, 0, 1));
        assert_eq!(ss.len(), 2);
        ss.push(item(rule.clone(), 2, 0, 1));
        assert_eq!(ss.len(), 3);
    }
}
