#![deny(warnings)]

use std::collections::HashMap;
use std::{fmt, hash};
use std::rc::Rc;
use std::string;

pub enum Symbol {
    NonTerm(String),
    // A terminal has a predicate to validate that input is accepted
    Term(String, Box<dyn Fn(&str)->bool>),
}

#[derive(PartialEq, Hash)]
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
    error: Option<String>,
}


impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Symbol::NonTerm(name) => name,
            Symbol::Term(name, _) => name,
        }
    }

    pub fn matches(&self, input: &str) -> bool {
        match self {
            Symbol::Term(_, matcher) => matcher(input),
            _ => false
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Symbol::Term(_, _))
    }
}

// Hashable Symbols allow storing them in containers (eg: HashMap)
// The name is the only way to dedup Terminals (ie: predicate is ignored)
impl hash::Hash for Symbol {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            // non-eq objects don't "have to" hash differently
            Symbol::Term(name, _) => name.hash(state),
            Symbol::NonTerm(name) => name.hash(state),
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        match (self, other) {
            (Symbol::Term(s, _), Symbol::Term(o, _))  => s == o,
            (Symbol::NonTerm(s), Symbol::NonTerm(o))  => s == o,
            _ => false
        }
    }
}

impl Eq for Symbol {}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Symbol::Term(name, _) => write!(f, "Term({})", name),
            Symbol::NonTerm(name) => write!(f, "NonTerm({})", name),
        }
    }
}

impl Rule {
    #[cfg(test)]
    pub fn new(head: &str, spec: &[Rc<Symbol>]) -> Self {
        Rule {
            head: head.to_string(),
            spec: spec.iter().cloned().collect()
        }
    }
}

impl string::ToString for Rule {
    fn to_string(&self) -> String {
        format!("{} -> {}", self.head, self.spec.iter().map(
                |s| s.name()).collect::<Vec<_>>().join(" "))
    }
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Builds a Gramar while validating existence of Symbols and checking rules.
impl GrammarBuilder {
    fn add_symbol(&mut self, symbol: Symbol, quiet: bool) {
        // Check for duplicate symbols to avoid overwriting by mistake
        if !self.symbols.contains_key(symbol.name()) {
            self.symbols.insert(symbol.name().to_string(), Rc::new(symbol));
        } else if !quiet {
            // Convenience for adding symbols programatically
            self.error = Some(format!("Duplicate Symbol: {}", symbol.name()));
        }
    }

    pub fn nonterm(mut self, name: impl Into<String>) -> Self {
        self.add_symbol(Symbol::NonTerm(name.into()), false);
        self
    }

    pub fn terminal(
        mut self, 
        name: impl Into<String>,
        pred: impl Fn(&str) -> bool + 'static) -> Self
    {
        self.add_symbol(Symbol::Term(name.into(), Box::new(pred)), false);
        self
    }

    // Quiet silently ignores adding pre-existent symbols to the grammar.
    // Also quiet versions don't use chaining to be invoked in loops.

    pub fn quiet_nonterm(&mut self, name: impl Into<String>) {
        self.add_symbol(Symbol::NonTerm(name.into()), true);
    }

    pub fn quiet_terminal(
        &mut self,
        name: impl Into<String>,
        pred: impl Fn(&str) -> bool + 'static)
    {
        self.add_symbol(Symbol::Term(name.into(), Box::new(pred)), true);
    }

    /// Register new rules for the grammar
    fn _add_rule<S, S2>(&mut self, head: S, spec: &[S2], quiet: bool)
        where S: AsRef<str>, S2: AsRef<str>
    {
        // First check that all symbols have been registered (need references)
        if let Some(s) = spec.iter().find(|n| !self.symbols.contains_key(n.as_ref())) {
            self.error = Some(format!("Missing Symbol: {}", s.as_ref()));
            return;
        }
        // nit TODO: check if head is a nonterminal
        if !self.symbols.contains_key(head.as_ref()) {
            self.error = Some(format!("Missing Symbol: {}", head.as_ref()));
            return;
        }
        // Build the rule
        let rule = Rc::new(Rule {
            head: head.as_ref().to_string(),
            spec: spec.iter().map(|s| self.symbols[s.as_ref()].clone()).collect()
        });
        // Check this rule is only added once. NOTE: `Rc`s equal on inner value
        if !self.rules.contains(&rule) {
            self.rules.push(rule);
        } else if !quiet {
            self.error = Some(format!("Duplicate Rule: {}", rule.to_string()));
        }
    }

    pub fn rule<S, S2>(mut self, head: S, spec: &[S2]) -> Self
        where S: AsRef<str>, S2: AsRef<str>
    {
        self._add_rule(head, spec, false);
        self
    }

    pub fn quiet_rule<S, S2>(&mut self, head: S, spec: &[S2])
        where S: AsRef<str>, S2: AsRef<str>
    {
        self._add_rule(head, spec, true)
    }

    pub fn into_grammar(mut self, start: impl Into<String>) -> Result<Grammar, String>
    {
        let start = start.into();
        if !self.symbols.contains_key(&start) {
            self.error = Some(format!("Missing Symbol: {}", start));
        }
        self.error.map_or(Ok(Grammar{start, rules: self.rules}), Err)
    }

    /// Generate unique name for a Symbol (used to build grammar mechanically)
    pub fn unique_symbol_name(&self) -> String {
        format!("<Uniq-{}>", self.symbols.len())
    }
}


///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{GrammarBuilder, Symbol};
    use std::collections::HashSet;

    #[test]
    fn symbol_eq_hash() {
        assert_ne!(
            Symbol::NonTerm("X".to_string()),
            Symbol::Term("X".to_string(), Box::new(|_| true))
        );
        // Check that term and non-term of equal name are not the same
        let mut m = HashSet::new();
        m.insert(Symbol::NonTerm("X".to_string()));
        m.insert(Symbol::Term("X".to_string(), Box::new(|_| true)));
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn symbol_terminal() {
        let term = Symbol::Term(
            "uint".to_string(),
            Box::new(|n| n.chars().all(|c| "1234567890".contains(c)))
        );
        assert_eq!(term.name(), "uint");
        assert!(term.matches("123"));
    }

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
        assert_eq!(g.unwrap_err(), "Duplicate Symbol: Sum");
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
        assert_eq!(g.unwrap_err(), "Duplicate Rule: Sum -> Sum + Num");
    }

    #[test]
    fn missing_symbol() {
        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .terminal("Num", |n| n.chars().all(|c| "123".contains(c)))
            .rule("Sum", &["Num"])
            .into_grammar("Xum");
        assert_eq!(g.unwrap_err(), "Missing Symbol: Xum");

        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .rule("Sum", &["Num"])
            .into_grammar("Sum");
        assert_eq!(g.unwrap_err(), "Missing Symbol: Num");
    }
}
