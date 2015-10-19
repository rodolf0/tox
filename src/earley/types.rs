use earley::uniqvec::UniqVec;
use std::collections::{HashMap, HashSet};
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
// TODO: grammar on function input, also add function name?
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

impl Symbol {
    pub fn nt_str<'a>(&'a self) -> Option<&'a str> {
        match self {
            &Symbol::NT(ref nonterm) => Some(&nonterm.0),
            _ => None
        }
    }
}

///////////////////////////////////////////////////////////
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Rule {
    // TODO: should these be Strings indexing the grammar's symbols?
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
    pub nullable: HashSet<String>,
}

impl Grammar {
    pub fn new<S: Into<String>>(start: S) -> Grammar {
        Grammar{
            start: start.into(),
            symbols: HashMap::new(),
            rules: HashMap::new(),
            nullable: HashSet::new(),
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

    // return a set of nullable rules according to the current grammar
    pub fn build_nullable(&mut self) {
        let mut nullable: HashSet<String> = HashSet::new();
        loop {
            let old_size = nullable.len();
            // for-each rule in the grammar, check if it's nullable
            let rules = self.rules.values().flat_map(|ruleset| ruleset.iter());
            for rule in rules {
                // for a rule to be nullable all symbols in the spec need
                // to be in the nullable set, else they're not nullable.
                // All empty specs will be nullable
                let isnull = rule.spec.iter().all(|symbol| match &**symbol {
                    &Symbol::NT(ref nt) => nullable.contains(&nt.0),
                    _ => false,
                });
                if isnull {
                    let name = match &*rule.name {
                        &Symbol::NT(ref name) => name.0.clone(),
                        _ => unreachable!()
                    };
                    nullable.insert(name);
                }
            }
            // we're done building the set when it no longer grows
            if old_size == nullable.len() { break; }
        }
        self.nullable = nullable;
    }
}

///////////////////////////////////////////////////////////
pub type StateSet = UniqVec<Item>;
