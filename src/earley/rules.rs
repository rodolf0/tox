use earley::symbol::Symbol;
use std::rc::Rc;

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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
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
