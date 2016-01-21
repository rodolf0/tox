use earley::types::{Symbol, Rule};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct Grammar {
    start: Rc<Symbol>,
    rules: Vec<Rc<Rule>>,
    nullable: HashSet<String>,
}

impl Grammar {
    // get rules filtered by name
    pub fn rules<'a>(&'a self, name: &'a str) ->
           Box<Iterator<Item=&'a Rc<Rule>> + 'a> {
        Box::new(self.rules.iter().filter(move |r| r.name() == name))
    }

    // grammar's start symbol
    pub fn start<'a>(&'a self) -> &'a str { self.start.name() }

    // check if a symbol in the grammar can derive in a nullable
    pub fn is_nullable(&self, nonterm: &str) -> bool {
        self.nullable.contains(nonterm)
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

    fn build_nullable(&self) -> HashSet<String> {
        let mut nullable: HashSet<String> = HashSet::new();
        loop {
            let old_size = nullable.len();
            // for-each rule in the grammar, check if it's nullable
            for rule in self.rules.iter() {
                // for a rule to be nullable all symbols in the spec need to be
                // in the nullable set, else they're not nullable. All empty
                // specs will therefore be nullable (all of it's 0 symbols are)
                let isnullable = rule.nullable(&nullable);
                if isnullable { nullable.insert(rule.name().to_string()); }
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
        }
    }
}
