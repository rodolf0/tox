use types::{Symbol, Rule};
use std::collections::HashMap;
use std::rc::Rc;

pub struct Grammar {
    start: Rc<Symbol>,
    rules: Vec<Rc<Rule>>,
}

impl Grammar {
    // get rules filtered by name
    pub fn rules<'a>(&'a self, name: &'a str) ->
           Box<Iterator<Item=&'a Rc<Rule>> + 'a> {
        Box::new(self.rules.iter().filter(move |r| r.name() == name))
    }

    // grammar's start symbol
    pub fn start<'a>(&'a self) -> &'a str { self.start.name() }
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

    pub fn symbol(&mut self, symbol: Symbol) -> &mut Self {
        self.symbols.insert(symbol.name().to_string(), Rc::new(symbol));
        self
    }

    pub fn rule<S>(&mut self, name: S, spec: Vec<S>) -> &mut Self
            where S: Into<String> + AsRef<str> {
        let rule = Rule::new(
            self.symbols[name.as_ref()].clone(),
            spec.iter().map(|s| self.symbols[s.as_ref()].clone()).collect()
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
