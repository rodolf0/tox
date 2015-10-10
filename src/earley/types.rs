use earley::uniqvec::UniqVec;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

///////////////////////////////////////////////////////////
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct NonTerminal(pub String);

impl NonTerminal {
    pub fn new<S: Into<String>>(s: S) -> Self { NonTerminal(s.into()) }
}

///////////////////////////////////////////////////////////
pub struct Terminal(Box<Fn(&str)->bool>);

impl Terminal {
    pub fn new<F: 'static + Fn(&str)->bool>(f: F) -> Self {
        Terminal(Box::new(f))
    }

    fn id(&self) -> u64 { self as *const Terminal as u64 }

    pub fn check(&self, input: &str) -> bool {
        let &Terminal(ref func) = self;
        func(input)
    }
}

impl fmt::Debug for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Terminal[{}]", self.id())
    }
}

impl Hash for Terminal {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id().hash(state); }
}

impl PartialEq for Terminal {
    fn eq(&self, other: &Terminal) -> bool { self.id() == other.id() }
}

impl Eq for Terminal {}

///////////////////////////////////////////////////////////
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Symbol {
    NT(NonTerminal),
    T(Terminal),
}

impl From<Terminal> for Symbol {
    fn from(t: Terminal) -> Symbol { Symbol::T(t) }
}

impl From<NonTerminal> for Symbol {
    fn from(nt: NonTerminal) -> Symbol { Symbol::NT(nt) }
}

///////////////////////////////////////////////////////////
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Rule {
    pub name: Rc<Symbol>,
    pub spec: Vec<Rc<Symbol>>,
}

///////////////////////////////////////////////////////////
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Item {
    pub rule: Rc<Rule>,
    pub start: usize,  // start of match (relative to input)
    pub dot: usize,    // how far are we in the rule
}

impl Item {
    pub fn next_symbol<'a>(&'a self) -> Option<&'a Symbol> {
        if let Some(symbol) = self.rule.spec.get(self.dot) {
            Some(&*symbol)
        } else {
            None
        }
    }
}

///////////////////////////////////////////////////////////
pub struct Grammar {
    pub start: String,
    pub symbols: HashMap<String, Rc<Symbol>>,
    pub rules: HashMap<String, Vec<Rc<Rule>>>,
}

impl Grammar {
    pub fn new<S: Into<String>>(start: S) -> Grammar {
        Grammar{
            start: start.into(),
            symbols: HashMap::new(),
            rules: HashMap::new(),
        }
    }

    // register symbols used to build grammar rules
    pub fn set_sym<N, S>(&mut self, name: N, symbol: S)
        where N: Into<String>, S: Into<Symbol> {
        self.symbols.insert(name.into(), Rc::new(symbol.into()));
    }

    // add new named grammar rule, rules are kept in order of addition
    pub fn add_rule<S>(&mut self, name: S, spec: Vec<S>)
    where S: Into<String> + AsRef<str> {
        let rule = Rc::new(Rule{
            name: self.symbols[name.as_ref()].clone(),
            spec: spec.iter()
                    .map(|s| self.symbols[s.as_ref()].clone())
                    .collect(),
        });
        self.rules.entry(name.into()).or_insert(Vec::new()).push(rule);
    }
}

///////////////////////////////////////////////////////////
pub type StateSet = UniqVec<Item>;
//pub struct StateSet(UniqVec<Item>);
