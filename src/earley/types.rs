use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::{mem, fmt};

///////////////////////////////////////////////////////////
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct NonTerminal(String);

impl NonTerminal {
    pub fn new<S: Into<String>>(s: S) -> Self { NonTerminal(s.into()) }
    pub fn name<'a>(&'a self) -> &'a str { &self.0 }
}

///////////////////////////////////////////////////////////
pub struct Terminal(String, Box<Fn(&str)->bool>);

impl Terminal {
    pub fn new<S, F>(name: S, f: F) -> Self
        where S: Into<String>, F: 'static + Fn(&str)->bool {
        Terminal(name.into(), Box::new(f))
    }

    fn fnid(&self) -> (usize, usize) {
        unsafe { mem::transmute::<_, (usize, usize)>(&*self.1) }
    }

    pub fn check(&self, input: &str) -> bool {
        let &Terminal(_, ref func) = self;
        func(input)
    }

    pub fn name<'a>(&'a self) -> &'a str { &self.0 }
}

impl fmt::Debug for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Hash for Terminal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        let (x, y) = self.fnid();
        x.hash(state); y.hash(state);
    }
}

impl PartialEq for Terminal {
    fn eq(&self, other: &Terminal) -> bool {
        self.0 == other.0 && self.fnid() == other.fnid()
    }
}

impl Eq for Terminal {}

///////////////////////////////////////////////////////////
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Symbol {
    NT(NonTerminal),
    T(Terminal),
}

impl Symbol {
    pub fn name<'a>(&'a self) -> &'a str {
        match self {
            &Symbol::NT(ref sym) => sym.name(),
            &Symbol::T(ref sym) => sym.name(),
        }
    }
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
        self.rule.spec.get(self.dot).map(|s| &**s)
    }
}

///////////////////////////////////////////////////////////
pub struct GrammarBuilder {
    symbols: HashMap<String, Rc<Symbol>>,
    rules: Vec<Rc<Rule>>,
}

impl GrammarBuilder {
    pub fn new() -> GrammarBuilder {
        GrammarBuilder{
            symbols: HashMap::new(),
            rules: Vec::new(),
        }
    }

    pub fn symbol<S: Into<Symbol>>(&mut self, symbol: S) -> &mut Self {
        let symbol = symbol.into();
        self.symbols.insert(symbol.name().to_string(), Rc::new(symbol));
        self
    }

    pub fn rule<S>(&mut self, name: S, spec: Vec<S>) -> &mut Self
    where S: Into<String> + AsRef<str> {
        let r = Rc::new(Rule{
            name: self.symbols[name.as_ref()].clone(),
            spec: spec.iter()
                    .map(|s| self.symbols[s.as_ref()].clone())
                    .collect(),
        });
        self.rules.push(r);
        self
    }

    // return a set of nullable rules names according to the current grammar
    fn build_nullable(&self) -> HashSet<String> {
        let mut nullable: HashSet<String> = HashSet::new();
        loop {
            let old_size = nullable.len();
            // for-each rule in the grammar, check if it's nullable
            for rule in self.rules.iter() {
                // for a rule to be nullable all symbols in the spec need
                // to be in the nullable set, else they're not nullable.
                // All empty specs will therefore be nullable (all symbols are)
                let isnullable = rule.spec.iter().all(|symbol| match &**symbol {
                    nt @ &Symbol::NT(_) => nullable.contains(nt.name()),
                    _ => false,
                });
                if isnullable {
                    nullable.insert(rule.name.name().to_string());
                }
            }
            // we're done building the set when it no longer grows
            if old_size == nullable.len() { break; }
        }
        nullable
    }

    pub fn into_grammar<S: AsRef<str>>(self, start: S) -> Grammar {
        Grammar{
            start: self.symbols[start.as_ref()].clone(),
            nullable: self.build_nullable(),
            rules: self.rules,
            symbols: self.symbols,
        }
    }
}

///////////////////////////////////////////////////////////
pub struct Grammar {
    pub start: Rc<Symbol>,
    pub rules: Vec<Rc<Rule>>,
    pub symbols: HashMap<String, Rc<Symbol>>,
    pub nullable: HashSet<String>,
}

impl Grammar {
    pub fn rules<'a, S: Into<String>>(&'a self, name: S) ->
	Box<Iterator<Item=&'a Rc<Rule>> + 'a> {
        let name = name.into();
        Box::new(self.rules.iter().filter(move |r| r.name.name() == name))
    }
}
