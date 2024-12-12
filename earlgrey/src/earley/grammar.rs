#![deny(warnings)]

use std::collections::HashMap;
use std::rc::Rc;
use std::{fmt, hash};

pub enum Symbol {
    NonTerm(String),
    // A terminal has a predicate to validate that input is accepted
    Term(String, Box<dyn Fn(&str) -> bool>),
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
            _ => false,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Symbol::Term(_, _))
    }
}

// Hashable Symbols allow storing them in containers (eg: HashMap)
impl hash::Hash for Symbol {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Symbol::Term(name, matcher) => {
                name.hash(state);
                (matcher as *const dyn Fn(&str) -> bool).hash(state);
            }
            Symbol::NonTerm(name) => name.hash(state),
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        match (self, other) {
            (Symbol::Term(s, m1), Symbol::Term(o, m2)) => {
                s == o
                    && std::ptr::addr_eq(
                        &m1 as *const dyn Fn(&str) -> bool,
                        &m2 as *const dyn Fn(&str) -> bool,
                    )
            }
            (Symbol::NonTerm(s), Symbol::NonTerm(o)) => s == o,
            _ => false,
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

#[derive(PartialEq, Hash)]
pub struct Rule {
    pub head: String,
    pub spec: Vec<Rc<Symbol>>,
}

impl Rule {
    #[cfg(test)]
    pub fn new(head: &str, spec: &[Rc<Symbol>]) -> Self {
        Rule {
            head: head.to_string(),
            spec: spec.iter().cloned().collect(),
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} -> {}",
            self.head,
            self.spec
                .iter()
                .map(|s| s.name())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
pub struct Grammar {
    pub start: String,
    pub rules: Vec<Rc<Rule>>,
}

impl fmt::Debug for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::collections::hash_map::Entry;
        let mut group_order = Vec::new();
        let mut rule_groups = HashMap::new();
        for r in &self.rules {
            match rule_groups.entry(&r.head) {
                Entry::Vacant(e) => {
                    group_order.push(&r.head);
                    e.insert(Vec::new()).push(r);
                }
                Entry::Occupied(mut e) => e.get_mut().push(r),
            }
        }
        writeln!(f, "Start: {}", self.start)?;
        for head in group_order {
            writeln!(f)?;
            for rule in rule_groups.get(head).unwrap() {
                writeln!(f, "{}", rule)?;
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct GrammarBuilder {
    symbols: HashMap<String, Rc<Symbol>>,
    rules: Vec<Rc<Rule>>,
    error: Option<String>,
}

/// Builds a Gramar while validating existence of Symbols and checking rules.
impl GrammarBuilder {
    fn add_symbol(&mut self, symbol: Symbol, ignore_dups: bool) {
        // Check for duplicate symbols to avoid overwriting by mistake
        if !self.symbols.contains_key(symbol.name()) {
            self.symbols
                .insert(symbol.name().to_string(), Rc::new(symbol));
        } else if !ignore_dups {
            // Convenience for adding symbols programatically
            self.error = Some(format!("Duplicate Symbol: {}", symbol.name()));
        }
    }

    pub fn nonterm(mut self, name: &str) -> Self {
        self.add_symbol(Symbol::NonTerm(name.into()), false);
        self
    }

    pub fn terminal(mut self, name: &str, pred: impl Fn(&str) -> bool + 'static) -> Self {
        self.add_symbol(Symbol::Term(name.into(), Box::new(pred)), false);
        self
    }

    pub fn nonterm_try(&mut self, name: &str) {
        self.add_symbol(Symbol::NonTerm(name.into()), true);
    }

    pub fn terminal_try(&mut self, name: &str, pred: impl Fn(&str) -> bool + 'static) {
        self.add_symbol(Symbol::Term(name.into(), Box::new(pred)), true);
    }

    // Register new rules for the grammar
    fn add_rule(&mut self, head: &str, spec: &[&str], ignore_dups: bool) {
        // First check that all symbols have been registered (need references)
        if let Some(s) = spec.iter().find(|&n| !self.symbols.contains_key(*n)) {
            self.error = Some(format!("Missing Symbol: {}", s));
            return;
        }
        // Check the head
        if let Some(s) = self.symbols.get(head) {
            if s.is_terminal() {
                self.error = Some(format!("Rule head must be Term: {}", head));
                return;
            }
        } else {
            self.error = Some(format!("Missing Symbol: {}", head));
            return;
        }
        // Build the rule
        let rule = Rc::new(Rule {
            head: head.to_string(),
            spec: spec.iter().map(|&s| self.symbols[s].clone()).collect(),
        });
        // Check this rule is only added once. NOTE: `Rc`s equal on inner value
        if !self.rules.contains(&rule) {
            self.rules.push(rule);
        } else if !ignore_dups {
            self.error = Some(format!("Duplicate Rule: {}", rule));
        }
    }

    pub fn rule(mut self, head: &str, spec: &[&str]) -> Self {
        self.add_rule(head, spec, false);
        self
    }

    pub fn rule_try(&mut self, head: &str, spec: &[&str]) {
        self.add_rule(head, spec, true)
    }

    pub fn into_grammar(mut self, start: &str) -> Result<Grammar, String> {
        let start = start.into();
        if let Some(s) = self.symbols.get(&start) {
            if s.is_terminal() {
                self.error = Some(format!("Grammar start must be NonTerm: {}", start));
            }
        } else {
            self.error = Some(format!("Missing start Symbol: {}", start));
        }
        self.error.map_or(
            Ok(Grammar {
                start,
                rules: self.rules,
            }),
            Err,
        )
    }

    // Generate unique name for a Symbol (used to build grammar mechanically)
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
    fn symbol_check_eq_hash() {
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
    fn symbol_terminal_matches() {
        let term = Symbol::Term(
            "uint".to_string(),
            Box::new(|n| n.chars().all(|c| "1234567890".contains(c))),
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
    fn grammar_has_dup_symbol() {
        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .nonterm("Sum")
            .into_grammar("Sum");
        assert_eq!(g.unwrap_err(), "Duplicate Symbol: Sum");
    }

    #[test]
    fn grammar_has_dup_rule() {
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
    fn grammar_rule_head_nonterm() {
        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .terminal("Num", |n| n.chars().all(|c| "123".contains(c)))
            .terminal("+", |n| n == "+")
            .rule("Sum", &["Sum", "+", "Num"])
            .rule("Sum", &["Num"])
            .into_grammar("Num");
        assert_eq!(g.unwrap_err(), "Grammar start must be NonTerm: Num");
    }

    #[test]
    fn grammar_missing_symbol() {
        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .terminal("Num", |n| n.chars().all(|c| "123".contains(c)))
            .rule("Sum", &["Num"])
            .into_grammar("Xum");
        assert_eq!(g.unwrap_err(), "Missing start Symbol: Xum");

        // Check missing symbol in rule body
        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .terminal("Num", |n| n.chars().all(|c| "123".contains(c)))
            .rule("Sum", &["Num", "+", "Num"])
            .into_grammar("Sum");
        assert_eq!(g.unwrap_err(), "Missing Symbol: +");

        // Check missing rule head symbol
        let g = GrammarBuilder::default()
            .nonterm("Sum")
            .terminal("Num", |n| n.chars().all(|c| "123".contains(c)))
            .rule("Rum", &["Num"])
            .into_grammar("Sum");
        assert_eq!(g.unwrap_err(), "Missing Symbol: Rum");
    }
}
