#![deny(warnings)]

use std::collections::HashMap;
use std::{fmt, hash};
use std::rc::Rc;
use crate::parser::Error;


pub enum Symbol {
    NonTerm(String),
    Terminal(String, Box<Fn(&str)->bool>),  // predicate that matches Terminal
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

#[derive(Default)]
pub struct GrammarBuilder {
    symbols: HashMap<String, Rc<Symbol>>,
    rules: Vec<Rc<Rule>>,
    error: Option<Error>,
}

///////////////////////////////////////////////////////////////////////////////

impl Symbol {
    pub fn name(&self) -> &str {
        match *self {
            Symbol::NonTerm(ref name) => name,
            Symbol::Terminal(ref name, _) => name,
        }
    }
}

// Symbol implements Hash + PartialEq so they can be uniq'd in HashSets
// Symbols are deduped by name ONLY
impl hash::Hash for Symbol {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match *self {
            Symbol::NonTerm(ref name) => name.hash(state),
            Symbol::Terminal(ref name, _) => name.hash(state)
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
    fn add_symbol(&mut self, symbol: Symbol, allowdups: bool) {
        // NOTE: we check existence to avoid new syms stomping on plugged syms
        if !self.symbols.contains_key(symbol.name()) {
            self.symbols.insert(symbol.name().to_string(), Rc::new(symbol));
        } else if !allowdups {
            self.error =
                Some(Error::DuplicateSym(symbol.name().to_string()));
        }
    }

    pub fn add_nonterm<S: Into<String>>(&mut self, nt: S, allowdups: bool) {
        self.add_symbol(Symbol::NonTerm(nt.into()), allowdups);
    }

    pub fn nonterm<S: Into<String>>(mut self, nt: S) -> Self {
        self.add_symbol(Symbol::NonTerm(nt.into()), false);
        self
    }

    pub fn add_terminal<S, TM>(&mut self, nt: S, tm: TM, allowdups: bool)
            where S: Into<String>, TM: 'static + Fn(&str)->bool {
        self.add_symbol(Symbol::Terminal(nt.into(), Box::new(tm)), allowdups);
    }

    pub fn terminal<S, TM>(mut self, nt: S, tm: TM) -> Self
            where S: Into<String>, TM: 'static + Fn(&str)->bool {
        self.add_symbol(Symbol::Terminal(nt.into(), Box::new(tm)), false);
        self
    }

    pub fn add_rule<H, S>(&mut self, head: H, spec: &[S], allowdups: bool)
            where H: Into<String>, S: AsRef<str> {
        // check for missing symbols first
        if let Some(s) = spec.iter()
                .find(|s| !self.symbols.contains_key(s.as_ref())) {
            self.error = Some(Error::MissingSym(s.as_ref().to_string()));
            return;
        }
        let head = head.into();
        if !self.symbols.contains_key(&head) {
            self.error = Some(Error::MissingSym(head));
            return;
        }
        let rule = Rule{
            head,
            spec: spec.iter()
                    .map(|s| self.symbols[s.as_ref()].clone()).collect()
        };
        // check for duplicate rules
        let rulestr = rule.to_string();
        if self.rules.iter().all(|r| r.to_string() != rulestr) {
            self.rules.push(Rc::new(rule));
        } else if !allowdups {
            self.error = Some(Error::DuplicateRule(rulestr));
        }
    }

    pub fn rule<H, S>(mut self, head: H, spec: &[S]) -> Self
            where H: Into<String>, S: AsRef<str> {
        self.add_rule(head, spec, false);
        self
    }

    pub fn into_grammar<S>(self, start: S) -> Result<Grammar, Error>
            where S: Into<String> {
        if let Some(e) = self.error {
            return Err(e);
        }
        let start = start.into();
        if !self.symbols.contains_key(&start) {
            return Err(Error::MissingSym(start));
        }
        Ok(Grammar{start, rules: self.rules})
    }

    // used to generate symbols programatically
    pub fn unique_symbol_name(&self) -> String {
        format!("<Uniq-{}>", self.symbols.len())
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{GrammarBuilder, Error};

    #[test]
    fn build_grammar() {
        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .terminal("Num", |n| n.chars().all(|c| "123".contains(c)))
            .terminal("+", |n| n == "+")
            .rule("Sum", &["Sum", "+", "Num"])
            .rule("Sum", &["Num"])
            .into_grammar("Sum");
        assert!(g.is_ok());
    }

    #[test]
    fn dup_symbol() {
        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .nonterm("Sum")
            .into_grammar("Sum");
        assert_eq!(g.unwrap_err(),
                   Error::DuplicateSym("Sum".to_string()));
    }

    #[test]
    fn dup_rule() {
        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .terminal("Num", |n| n.chars().all(|c| "123".contains(c)))
            .terminal("+", |n| n == "+")
            .rule("Sum", &["Sum", "+", "Num"])
            .rule("Sum", &["Sum", "+", "Num"])
            .rule("Sum", &["Num"])
            .into_grammar("Sum");
        assert_eq!(g.unwrap_err(),
                   Error::DuplicateRule("Sum -> Sum + Num".to_string()));
    }

    #[test]
    fn missing_start() {
        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .terminal("Num", |n| n.chars().all(|c| "123".contains(c)))
            .rule("Sum", &["Num"])
            .into_grammar("Xum");
        assert_eq!(g.unwrap_err(), Error::MissingSym("Xum".to_string()));

        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .rule("Sum", &["Num"])
            .into_grammar("Sum");
        assert_eq!(g.unwrap_err(), Error::MissingSym("Num".to_string()));
    }
}
