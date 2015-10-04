use earley::UniqVec;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::fmt;

///////////////////////////////////////////////////////////
pub struct Terminal {
    func: Box<Fn(&str) -> bool>,
}

impl PartialEq for Terminal {
    fn eq(&self, other: &Terminal) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Terminal {}

impl Hash for Terminal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl fmt::Debug for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Terminal[{}]", self.id())
    }
}

impl Terminal {
    pub fn check(&self, input: &str) -> bool {
        (*self.func)(input)
    }

    fn id(&self) -> u64 {
        &self.func as *const Box<_> as u64
    }
}

pub type NonTerminal = String;

///////////////////////////////////////////////////////////
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Symbol {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
}

impl Symbol {
    pub fn nonterm<S: Into<String>>(nt: S) -> Symbol {
        Symbol::NonTerminal(nt.into())
    }
    //pub fn terminal<F: 'static + for<'a> Fn(&'a str) -> bool>(f: F) -> Symbol {
    pub fn terminal<F: 'static + Fn(&str) -> bool>(f: F) -> Symbol {
        Symbol::Terminal(Terminal{func: Box::new(f)})
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Rule {
    pub name: NonTerminal,
    pub spec: Vec<Symbol>,
}

impl Rule {
    pub fn new<S: Into<String>>(name: S, spec: Vec<Symbol>) -> Rule {
        Rule{
            name: name.into(),
            spec: spec,
        }
    }
}

///////////////////////////////////////////////////////////
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub rule: Rc<Rule>,
    pub start: usize,  // start of match (relative to input)
    pub dot: usize,    // how far are we in the rule
}

impl Item {
    pub fn next_symbol(&self) -> Option<&Symbol> {
        self.rule.spec.get(self.dot)
    }
}

///////////////////////////////////////////////////////////
pub struct Grammar {
    pub start: NonTerminal,
    pub rules: HashMap<NonTerminal, Vec<Rc<Rule>>>,
}

pub type StateSet = UniqVec<Item>;
