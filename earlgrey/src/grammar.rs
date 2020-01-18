#![deny(warnings)]

use std::collections::HashMap;
use std::{fmt, hash};
use std::rc::Rc;
use std::string;


/// Symbol has a unique name. It is a non-terminal unless it
/// provides a predicate to match lexemes and becomes a terminal
pub struct Symbol(String, Option<Box<dyn Fn(&str)->bool>>);

impl Symbol {
    /// Return the name of the symbol only if its a NonTerminal
    pub fn nonterm(&self) -> Option<&str> {
        self.1.as_ref().map_or(Some(self.0.as_ref()), |_| None)
    }

    /// Return the name and the predicate only if this symbols is a Terminal
    pub fn terminal(&self) -> Option<(&str, &dyn Fn(&str) -> bool)> {
        self.1.as_ref().map(|f| (self.0.as_ref(), f.as_ref()))
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    #[cfg(test)]
    pub fn new(name: &str) -> Rc<Symbol> {
        Rc::new(Symbol(name.to_string(), None))
    }

    #[cfg(test)]
    pub fn new2(name: &str, pred: impl Fn(&str)->bool + 'static) -> Rc<Symbol> {
        Rc::new(Symbol(name.to_string(), Some(Box::new(pred))))
    }
}

// Hashable Symbols allow storing them in containers (eg: HashMap)
// The name is the only way to dedup Terminals (ie: predicate is ignored)
impl hash::Hash for Symbol {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        self.0 == other.0
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some((name, _)) = self.terminal() {
            return write!(f, "Terminal({})", name);
        } else if let Some(name) = self.nonterm() {
            return write!(f, "NonTerm({})", name);
        };
        write!(f, "BUG: unknown symbol type")
    }
}

/// A grammar Rule "S -> S b" has a head that must be a non-Terminal.
/// The spec is a list of Terminal and Non-Terminal Symbols.
/// Normally `Rule`s are built by GrammarBuilder not directly by user.
#[derive(PartialEq, Hash)]
pub struct Rule {
    pub head: String,
    pub spec: Vec<Rc<Symbol>>,
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

impl Rule {
    #[cfg(test)]
    pub fn new(head: &str, spec: &[Rc<Symbol>]) -> Self {
        Rule {
            head: head.to_string(),
            spec: spec.iter().cloned().collect()
        }
    }
}


#[derive(Clone,Debug)]
pub struct Grammar {
    pub start: String,
    pub rules: Vec<Rc<Rule>>,
}


/// Builds a Gramar while validating existence of Symbols and checking rules.
#[derive(Default)]
pub struct GrammarBuilder {
    symbols: HashMap<String, Rc<Symbol>>,
    rules: Vec<Rc<Rule>>,
    error: Option<String>,
}

impl GrammarBuilder {
    fn _add_symbol(&mut self, symbol: Symbol, quiet: bool) {
        // Check for duplicate symbols to avoid overwriting by mistake
        if !self.symbols.contains_key(symbol.name()) {
            self.symbols.insert(symbol.name().to_string(), Rc::new(symbol));
        } else if !quiet {
            // Convenience for adding symbols programatically
            self.error = Some(format!("Duplicate Symbol: {}", symbol.name()));
        }
    }

    pub fn nonterm<S>(mut self, name: S) -> Self where S: Into<String> {
        self._add_symbol(Symbol(name.into(), None), false);
        self
    }

    pub fn terminal<S, P>(mut self, name: S, pred: P) -> Self
        where S: Into<String>,
              P: 'static + Fn(&str) -> bool,
    {
        self._add_symbol(Symbol(name.into(), Some(Box::new(pred))), false);
        self
    }

    // Quiet silently ignores adding pre-existent symbols to the grammar.
    // Also quiet versions don't use chaining to be invoked in loops.

    pub fn quiet_nonterm<S>(&mut self, name: S) where S: Into<String> {
        self._add_symbol(Symbol(name.into(), None), true)
    }

    pub fn quiet_terminal<S, P>(&mut self, name: S, pred: P)
        where S: Into<String>,
              P: 'static + Fn(&str) -> bool,
    {
        self._add_symbol(Symbol(name.into(), Some(Box::new(pred))), true)
    }

    /// Register new rules for the grammar
    fn _add_rule<S>(&mut self, head: S, spec: &[S], quiet: bool)
        where S: AsRef<str>
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

    pub fn rule<S>(mut self, head: S, spec: &[S]) -> Self where S: AsRef<str> {
        self._add_rule(head, spec, false);
        self
    }

    pub fn quiet_rule<S>(&mut self, head: S, spec: &[S]) where S: AsRef<str> {
        self._add_rule(head, spec, true)
    }

    /// Consume builder into Grammar
    pub fn into_grammar<S>(mut self, start: S) -> Result<Grammar, String>
        where S: Into<String>
    {
        let start = start.into();
        if !self.symbols.contains_key(&start) {
            self.error = Some(format!("Missing Symbol: {}", start));
        }
        self.error.map_or(Ok(Grammar{start, rules: self.rules}), |e| Err(e))
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
    use std::collections::HashMap;

    #[test]
    fn symbol_eq() {
        assert_eq!(Symbol::new("X"), Symbol::new2("X", |_| true));
        let mut m = HashMap::new();
        m.insert("X", Symbol::new("X"));
        m.insert("X", Symbol::new2("X", |_| true));
        assert!(m.len() == 1);
    }

    #[test]
    fn symbol_extract() {
        assert!(Symbol::new("X").terminal().is_none());
        assert!(Symbol::new("X").nonterm().is_some());
        assert!(Symbol::new2("X", |_| true).terminal().is_some());
        assert!(Symbol::new2("X", |_| true).nonterm().is_none());
    }

    #[test]
    fn symbol_terminal() {
        let symd = Symbol::new2("d", |n| n.chars().all(|c| "123".contains(c)));
        let (name, pred) = symd.terminal().unwrap();
        assert_eq!(name, "d");
        assert!(pred("32"));
        assert!(!pred("55"));
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
