use earley::uniquedvec::UniqedVec;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

// ex: let semicolon = Rc::new(|check| check == ";");
// ex: let arithop = Rc::new(|op| op in ["+", "-", "*"]);
// ex: let number = Rc::new(|op| op in [0-9]*);
pub struct Terminal {
    pub f: Rc<Fn(&str) -> bool>,
}

impl PartialEq for Terminal {
    fn eq(&self, other: &Terminal) -> bool {
        self == other
    }
}

impl Eq for Terminal {}

impl Hash for Terminal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let addr = self as *const Terminal as u64;
        addr.hash(state);
    }
}

// ex: 'Sum'
pub type NonTerminal = String;

#[derive(PartialEq, Eq, Hash)]
pub enum Symbol {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
}

#[derive(PartialEq, Eq, Hash)]
pub struct Rule {
    pub left: Symbol,
    pub right: Vec<Symbol>,
}

pub struct Grammar {
    pub start: NonTerminal,
    pub rules: HashMap<NonTerminal, Vec<Rc<Rule>>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Item {
    pub rule: Rc<Rule>,
    pub start: usize,  // where does the match start (relative to input)
    // TODO: changed dot -> next? better reading algo?
    pub dot: usize,    // how far are we in the rule
}

/*
impl Item {
    fn next_symbol(&self) {
        self.rule[self.dot+1]
    }
}
*/

pub type StateSet = UniqedVec<Item>;
