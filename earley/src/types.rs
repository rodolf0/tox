use std::collections::{HashSet, HashMap};
use std::ops::Index;
use std::{fmt, hash, mem, iter, slice};
use std::rc::Rc;
use std::cell::RefCell;

pub enum Symbol {
    NonTerm(String),
    Terminal(String, Box<Fn(&str)->bool>),
}

impl Symbol {
// TODO: remove in favor of From
    pub fn nonterm<S: Into<String>>(s: S) -> Self { Symbol::NonTerm(s.into()) }

    pub fn terminal<S, F>(name: S, f: F) -> Self
            where S: Into<String>, F: 'static + Fn(&str)->bool {
        Symbol::Terminal(name.into(), Box::new(f))
    }

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

//impl<S, F> From<(S, F)> for Symbol
        //where S: Into<String>, F: 'static + FnMut(&str)->bool {
    //fn from(from: (S, F)) -> Self {
        //Symbol::Terminal(from.0.into(), Box::new(from.1))
    //}
//}

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

    pub fn name(&self) -> String { self.name.name() }

    pub fn spec(&self) -> String {
        self.spec.iter().map(|s| s.name()).collect::<Vec<_>>().join(" ")
    }

    pub fn spec_parts(&self) -> Vec<String> {
        self.spec.iter().map(|s| s.name().to_string()).collect()
    }
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
    end: usize,    // Earley state where item ends
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
    pub fn new(rule: Rc<Rule>, dot: usize, start: usize, end: usize) -> Item {
        Item{rule: rule, dot: dot, start: start, end: end,
             bp: RefCell::new(HashSet::new())}
    }

    pub fn start(&self) -> usize { self.start }
    pub fn complete(&self) -> bool { self.dot >= self.rule.spec.len() }

    pub fn str_rule(&self) -> String {
        format!("{} -> {}", self.rule.name(), self.rule.spec())
    }

    pub fn next_symbol<'a>(&'a self) -> Option<&'a Symbol> {
        self.rule.spec.get(self.dot).map(|s| &**s)
    }

    // check if other item's next-symbol matches our rule's name
    pub fn can_complete(&self, other: &Rc<Item>) -> bool {
        self.complete() && match other.next_symbol() {
            Some(&Symbol::NonTerm(ref name)) => *name == self.rule.name(),
            _ => false
        }
    }

    // check item's next symbol is a temrinal that scans lexeme
    pub fn can_scan(&self, lexeme: &str) -> bool {
        match self.next_symbol() {
            Some(&Symbol::Terminal(_, ref f)) => f(lexeme),
            _ => false
        }
    }

    // build a new Item for a prediction
    pub fn predict_new(rule: &Rc<Rule>, start: usize) -> Item {
        Item{rule: rule.clone(), dot: 0, start: start, end: start,
             bp: RefCell::new(HashSet::new())}
    }

    // produce an Item after scanning using another item as the base
    pub fn scan_new(source: &Rc<Item>, end: usize, input: &str) -> Item {
        let mut _bp = HashSet::new();
        _bp.insert((source.clone(), Trigger::Scan(input.to_string())));
        Item{rule: source.rule.clone(), dot: source.dot+1,
             start: source.start, end: end, bp: RefCell::new(_bp)}
    }

    pub fn complete_new(source: &Rc<Item>, trigger: &Rc<Item>, end: usize) -> Item {
        let mut _bp = HashSet::new();
        _bp.insert((source.clone(), Trigger::Completion(trigger.clone())));
        Item{rule: source.rule.clone(), dot: source.dot+1,
             start: source.start, end: end, bp: RefCell::new(_bp)}
    }

    pub fn back_pointers(&self) -> HashSet<(Rc<Item>, Trigger)> {
        self.bp.borrow().clone()
    }
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pre = self.rule.spec.iter()
            .take(self.dot).map(|s| s.name()).collect::<Vec<_>>().join(" ");
        let post = self.rule.spec.iter()
            .skip(self.dot).map(|s| s.name()).collect::<Vec<_>>().join(" ");
        write!(f, "({} - {}) {} -> {} \u{00b7} {} #bp: {}",
               self.start, self.end, self.rule.name(), pre, post,
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
    pub fn push(&mut self, item: Item) {
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
    pub fn filter_by_rule<'a>(&'a self, name: String) ->
           Box<Iterator<Item=&'a Rc<Item>> + 'a> {
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
    type Output = Rc<Item>;
    fn index<'b>(&'b self, idx: usize) -> &'b Rc<Item> { self.order.index(idx) }
}

impl fmt::Debug for StateSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.order.fmt(f) }
}

///////////////////////////////////////////////////////////////////////////////

pub struct Grammar {
    start: Rc<Symbol>,
    rules: Vec<Rc<Rule>>,
}

impl Grammar {
    // get rules filtered by name
    pub fn rules<'a, S: Into<String>>(&'a self, name: S) ->
           Box<Iterator<Item=&'a Rc<Rule>> + 'a> {
        let name = name.into();
        Box::new(self.rules.iter().filter(move |r| r.name() == name))
    }

    pub fn all_rules<'a>(&'a self) -> slice::Iter<'a, Rc<Rule>> {
        self.rules.iter()
    }

    // grammar's start symbol
    pub fn start(&self) -> String { self.start.name() }
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

    pub fn symbols<S, I>(&mut self, symbols: I) -> &mut Self
            where S: Into<Symbol>, I: IntoIterator<Item=S> {
        self.symbols.extend(
            symbols.into_iter().map(|s| {
                let x = s.into();
                (x.name().to_string(), Rc::new(x))
            }));
                                    //(s.name().to_string(), Rc::new(s))));
        self
    }

    pub fn symbol<S: Into<Symbol>>(&mut self, symbol: S) -> &mut Self {
        let x = symbol.into();
        self.symbols.insert(x.name().to_string(), Rc::new(x));
        self
    }

    //pub fn rule<I, N, S>(&mut self, name: N, spec: I) -> &mut Self
            //where N: AsRef<str>, S: AsRef<str>,
                  //I: IntoIterator<Item=S> {
// &[S]
    pub fn rule<S>(&mut self, name: S, spec: Vec<S>) -> &mut Self
            where S: AsRef<str>, S: AsRef<str> {
        let rule = Rule::new(
            self.symbols[name.as_ref()].clone(),
            spec.into_iter().map(|s| self.symbols[s.as_ref()].clone()).collect()
        );
        self.rules.push(Rc::new(rule));
        self
    }

    pub fn into_grammar<S: AsRef<str>>(self, start: S) -> Grammar {
        Grammar{
            start: self.symbols[start.as_ref()].clone(),
            rules: self.rules,
        }
    }
}
