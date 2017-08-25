#![deny(warnings)]

use std::collections::HashMap;
use std::{fmt, hash};
use std::rc::Rc;


#[derive(Debug,PartialEq)]
pub enum GrammarError {
    MissingSym(String),
    DuplicateSym(String),
    DuplicateRule(String),
}

//pub type TokenMatcher<T> = Box<Fn(&T)->bool>;

pub enum Symbol {
    NonTerm(String),
    Terminal(String, Box<Fn(&str)->bool>),  // predicate that matches Terminal
    //Terminal(String, TokenMatcher<T>),  // predicate that matches Terminal
}

#[derive(PartialEq,Hash)]
pub struct Rule {
    pub head: String,
    pub spec: Vec<Rc<Symbol>>,
}

#[derive(Clone,Debug)]
pub struct Grammar {
    pub start: String,
    pub rules: Vec<Rc<Rule>>,
}

pub struct GrammarBuilder {
    symbols: HashMap<String, Rc<Symbol>>,
    rules: Vec<Rc<Rule>>,
    error: Option<GrammarError>,
}

///////////////////////////////////////////////////////////////////////////////

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            &Symbol::NonTerm(ref name) => name,
            &Symbol::Terminal(ref name, _) => name,
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

// Symbol implements Hash + PartialEq so they can be uniq'd in HashSets
// Symbols are deduped by name ONLY
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

///////////////////////////////////////////////////////////////////////////////

impl Rule {
    pub fn symbol_at(&self, idx: usize) -> Option<&Rc<Symbol>> {
        self.spec.get(idx)
    }

    pub fn to_string(&self) -> String {
        format!("{} -> {}", self.head, self.spec.iter().map(
                |s| s.name()).collect::<Vec<_>>().join(" "))
    }
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

///////////////////////////////////////////////////////////////////////////////

impl Grammar {
    pub fn rules_for(&self, head: &str) -> Vec<Rc<Rule>> {
        self.rules.iter()
            .filter(|r| r.head == head)
            .cloned()
            .collect()
    }

    pub fn str_rules(&self) -> Vec<String> {
        self.rules.iter().map(|r| r.to_string()).collect()
    }
}

///////////////////////////////////////////////////////////////////////////////

impl GrammarBuilder {
    pub fn new() -> GrammarBuilder {
        GrammarBuilder{symbols: HashMap::new(), rules: Vec::new(), error: None}
    }

    pub fn add_symbol<S: Into<Symbol>>(&mut self, symbol: S, ignoredup: bool) {
        // NOTE: we check existence to avoid new symbols stomping on pluged ones
        let symbol = symbol.into();
        if !self.symbols.contains_key(symbol.name()) {
            self.symbols.insert(symbol.name().to_string(), Rc::new(symbol));
        } else if !ignoredup {
            self.error =
                Some(GrammarError::DuplicateSym(symbol.name().to_string()));
        }
    }

    pub fn symbol<S: Into<Symbol>>(mut self, symbol: S) -> Self {
        self.add_symbol(symbol, false);
        self
    }

    pub fn add_rule<H, S>(&mut self, head: H, spec: &[S])
            where H: Into<String>, S: AsRef<str> {
        // check for missing symbols first
        if let Some(s) = spec.iter()
                .filter(|s| !self.symbols.contains_key(s.as_ref())).next() {
            self.error = Some(GrammarError::MissingSym(s.as_ref().to_string()));
            return;
        }
        let head = head.into();
        if !self.symbols.contains_key(&head) {
            self.error = Some(GrammarError::MissingSym(head));
            return;
        }
        let rule = Rule{
            head: head,
            spec: spec.into_iter()
                    .map(|s| self.symbols[s.as_ref()].clone()).collect()
        };
        // check for duplicate rules
        let rulestr = rule.to_string();
        if self.rules.iter().any(|r| r.to_string() == rulestr) {
            self.error = Some(GrammarError::DuplicateRule(rulestr));
            return;
        }
        self.rules.push(Rc::new(rule));
    }

    pub fn rule<H, S>(mut self, head: H, spec: &[S]) -> Self
            where H: Into<String>, S: AsRef<str> {
        self.add_rule(head, spec);
        self
    }

    pub fn into_grammar<S>(self, start: S) -> Result<Grammar, GrammarError>
            where S: Into<String> {
        if let Some(e) = self.error {
            return Err(e);
        }
        let start = start.into();
        if !self.symbols.contains_key(&start) {
            return Err(GrammarError::MissingSym(start));
        }
        Ok(Grammar{start: start, rules: self.rules})
    }

    // used to generate symbols programatically
    pub fn unique_symbol_name(&self) -> String {
        format!("<Uniq-{}>", self.symbols.len())
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{GrammarBuilder, GrammarError};

    #[test]
    fn build_grammar() {
        let g = GrammarBuilder::new()
            .symbol("Sum")
            .symbol(("Num", |n: &str| n.chars().all(|c| "123".contains(c))))
            .symbol(("+", |n: &str| n == "+"))
            .rule("Sum", &["Sum", "+", "Num"])
            .rule("Sum", &["Num"])
            .into_grammar("Sum");
        assert!(g.is_ok());
    }

    #[test]
    fn dup_symbol() {
        let g = GrammarBuilder::new()
            .symbol("Sum")
            .symbol("Sum")
            .into_grammar("Sum");
        assert_eq!(g.unwrap_err(),
                   GrammarError::DuplicateSym("Sum".to_string()));
    }

    #[test]
    fn dup_rule() {
        let g = GrammarBuilder::new()
            .symbol("Sum")
            .symbol(("Num", |n: &str| n.chars().all(|c| "123".contains(c))))
            .symbol(("+", |n: &str| n == "+"))
            .rule("Sum", &["Sum", "+", "Num"])
            .rule("Sum", &["Sum", "+", "Num"])
            .rule("Sum", &["Num"])
            .into_grammar("Sum");
        assert_eq!(g.unwrap_err(),
                   GrammarError::DuplicateRule("Sum -> Sum + Num".to_string()));
    }

    #[test]
    fn missing_start() {
        let g = GrammarBuilder::new()
            .symbol("Sum")
            .symbol(("Num", |n: &str| n.chars().all(|c| "123".contains(c))))
            .rule("Sum", &["Num"])
            .into_grammar("Xum");
        assert_eq!(g.unwrap_err(), GrammarError::MissingSym("Xum".to_string()));

        let g = GrammarBuilder::new()
            .symbol("Sum")
            .rule("Sum", &["Num"])
            .into_grammar("Sum");
        assert_eq!(g.unwrap_err(), GrammarError::MissingSym("Num".to_string()));
    }
}
