use crate::earley::{EarleyForest, EarleyParser};
use crate::ebnf::EbnfGrammarParser;
use crate::sexpr::Sexpr;
use std::collections::HashMap;
use std::rc::Rc;

#[allow(clippy::type_complexity)]
pub struct ParserBuilder<'a, T: Clone + 'a> {
    grammar_str: String,
    start_rule: String,
    terminals: HashMap<String, Rc<dyn Fn(&str) -> Option<T> + 'a>>,
    terminal_preds: HashMap<String, Rc<dyn Fn(&str) -> bool + 'static>>,
    literals: HashMap<String, T>,
    actions: HashMap<String, Rc<dyn Fn(Vec<T>) -> T + 'a>>,
    default_action: Option<Rc<dyn Fn(&str, Vec<T>) -> T + 'a>>,
    optional_empty: Option<Rc<dyn Fn() -> T + 'a>>,
    list_empty: Option<Rc<dyn Fn() -> T + 'a>>,
    list_action: Option<Rc<dyn Fn(T, Vec<T>) -> T + 'a>>,
    unmapped_literal: Option<Rc<dyn Fn(&str) -> T + 'a>>,
}

impl<'a, T: Clone + 'a> ParserBuilder<'a, T> {
    pub fn new(grammar: &str, start: &str) -> Self {
        Self {
            grammar_str: grammar.to_string(),
            start_rule: start.to_string(),
            terminals: HashMap::new(),
            terminal_preds: HashMap::new(),
            literals: HashMap::new(),
            actions: HashMap::new(),
            default_action: None,
            optional_empty: None,
            list_empty: None,
            list_action: None,
            unmapped_literal: None,
        }
    }

    pub fn terminal(mut self, name: &str, parse: impl Fn(&str) -> Option<T> + 'static) -> Self {
        let p = Rc::new(parse);
        self.terminals
            .insert(name.to_string(), p.clone())
            .map(|_| panic!("Duplicate terminal registered: {}", name));
        self.terminal_preds
            .insert(name.to_string(), Rc::new(move |s| p(s).is_some()));
        self
    }

    pub fn literal(mut self, name: &str, value: T) -> Self {
        self.literals
            .insert(name.to_string(), value)
            .map(|_| panic!("Duplicate literal registered: {}", name));
        self
    }

    pub fn literals(mut self, names: &[&str], value: T) -> Self {
        for name in names {
            self.literals
                .insert(name.to_string(), value.clone())
                .map(|_| panic!("Duplicate literal registered: {}", name));
        }
        self
    }

    pub fn unmapped_literal(mut self, parse: impl Fn(&str) -> T + 'static) -> Self {
        self.unmapped_literal = Some(Rc::new(parse));
        self
    }

    pub fn action(mut self, rule: &str, action: impl Fn(Vec<T>) -> T + 'static) -> Self {
        self.actions
            .insert(rule.to_string(), Rc::new(action))
            .map(|_| panic!("Duplicate action registered for rule: {}", rule));
        self
    }

    pub fn action1(mut self, rule: &str, action: impl Fn(T) -> T + 'static) -> Self {
        self.actions
            .insert(rule.to_string(), Rc::new(move |mut v| action(v.remove(0))))
            .map(|_| panic!("Duplicate action registered for rule: {}", rule));
        self
    }

    pub fn action2(mut self, rule: &str, action: impl Fn(T, T) -> T + 'static) -> Self {
        self.actions
            .insert(
                rule.to_string(),
                Rc::new(move |mut v| {
                    let a2 = v.remove(1);
                    let a1 = v.remove(0);
                    action(a1, a2)
                }),
            )
            .map(|_| panic!("Duplicate action registered for rule: {}", rule));
        self
    }

    pub fn action3(mut self, rule: &str, action: impl Fn(T, T, T) -> T + 'static) -> Self {
        self.actions
            .insert(
                rule.to_string(),
                Rc::new(move |mut v| {
                    let a3 = v.remove(2);
                    let a2 = v.remove(1);
                    let a1 = v.remove(0);
                    action(a1, a2, a3)
                }),
            )
            .map(|_| panic!("Duplicate action registered for rule: {}", rule));
        self
    }

    pub fn action4(mut self, rule: &str, action: impl Fn(T, T, T, T) -> T + 'static) -> Self {
        self.actions
            .insert(
                rule.to_string(),
                Rc::new(move |mut v| {
                    let a4 = v.remove(3);
                    let a3 = v.remove(2);
                    let a2 = v.remove(1);
                    let a1 = v.remove(0);
                    action(a1, a2, a3, a4)
                }),
            )
            .map(|_| panic!("Duplicate action registered for rule: {}", rule));
        self
    }

    pub fn default_action(mut self, action: impl Fn(&str, Vec<T>) -> T + 'static) -> Self {
        self.default_action = Some(Rc::new(action));
        self
    }

    pub fn optional_empty(mut self, action: impl Fn() -> T + 'static) -> Self {
        self.optional_empty = Some(Rc::new(action));
        self
    }

    pub fn list_empty(mut self, action: impl Fn() -> T + 'static) -> Self {
        self.list_empty = Some(Rc::new(action));
        self
    }

    pub fn list_action(mut self, action: impl Fn(T, Vec<T>) -> T + 'static) -> Self {
        self.list_action = Some(Rc::new(action));
        self
    }

    pub fn build(mut self) -> Result<Parser<'a, T>, String> {
        // Parse the user's EBNF like grammar and build an internal representation.
        let mut ebnf_parser = EbnfGrammarParser::new(&self.grammar_str, &self.start_rule);
        for (name, pred) in self.terminal_preds.into_iter() {
            ebnf_parser = ebnf_parser.plug_terminal(&name, move |s| pred(s));
        }
        for name in self.literals.keys() {
            let n = name.clone();
            ebnf_parser = ebnf_parser.plug_terminal(&name, move |s| s == n);
        }
        let grammar = ebnf_parser.into_grammar()?;

        // Validate that all symbols in the grammar have been registered.
        for rule in &grammar.rules {
            for sym in &rule.spec {
                if sym.is_terminal()
                    && !self.terminals.contains_key(sym.name())
                    && !self.literals.contains_key(sym.name())
                    && self.unmapped_literal.is_none()
                {
                    return Err(format!("Missing terminal parser for: {}", sym.name()));
                }
            }
        }
        // Validate explicitly registered actions match a real rule
        for k in self.actions.keys() {
            if !grammar.rules.iter().any(|r| &r.id == k) {
                return Err(format!("Action registered for unknown rule: {}", k));
            }
        }
        // Setup the lifting of terminals and semantic actions.
        let terminals = self.terminals;
        let literals = self.literals;
        let unmapped_literal = self.unmapped_literal;
        let mut forest = EarleyForest::<'a, T>::new(move |sym, tok| {
            if let Some(parse) = terminals.get(sym) {
                parse(tok).expect("Terminal matched but failed to parse")
            } else if let Some(val) = literals.get(sym) {
                val.clone()
            } else if let Some(ref fallback) = unmapped_literal {
                fallback(tok)
            } else {
                panic!("Missing terminal parser for {}", sym);
            }
        });
        for rule in &grammar.rules {
            // map semantic action for each rule in the grammar.
            if let Some(action) = self.actions.remove(&rule.id) {
                forest.action(&rule.id, move |v| action(v));
                continue;
            }
            // Auto pass-through for length-1 rules.
            if rule.spec.len() == 1 {
                forest.action(&rule.id, |mut v| v.swap_remove(0));
                continue;
            }
            // Fallback to empty actions if the rule is an epsilon production.
            if rule.spec.is_empty() {
                if rule.head.starts_with('{') {
                    if let Some(ref list_empty) = self.list_empty {
                        let list_empty = list_empty.clone();
                        forest.action(&rule.id, move |_| list_empty());
                        continue;
                    }
                } else if rule.head.starts_with('[') {
                    if let Some(ref optional_empty) = self.optional_empty {
                        let optional_empty = optional_empty.clone();
                        forest.action(&rule.id, move |_| optional_empty());
                        continue;
                    }
                }
                return Err(format!("Missing action for empty rule: {}", rule.id));
            }
            // Repetition generates auxiliary rules. Eg: x -> { a | b c } ;
            // {a|bc} -> {a|bc} a | {a|bc} b c | [] ;
            // The user can provide a 'list_action' to combine multiple items into a list.
            if rule.head.starts_with('{') && rule.spec.len() >= 2 {
                #[allow(clippy::collapsible_if)]
                if let Some(ref list_action) = self.list_action {
                    let list_action = list_action.clone();
                    forest.action(&rule.id, move |mut arglist| {
                        let list = arglist.remove(0);
                        list_action(list, arglist)
                    });
                    continue;
                }
            }
            // Allow catching generic sequences, eg: "A -> B C D".
            // For example for putting all items into a list or struct
            if let Some(ref default_a) = self.default_action {
                let a = default_a.clone();
                let head = rule.head.clone();
                forest.action(&rule.id, move |v| a(&head, v));
                continue;
            }
            // If no default action, provide a better error for missing list_action
            if rule.head.starts_with('{') && rule.spec.len() >= 2 {
                return Err(format!(
                    "EBNF repetition rule '{}' has no action. \
                     When using {{...}} repetition in your grammar, you must provide \
                     either `list_action` or `default_action` to combine repeated items.",
                    rule.id
                ));
            }
            return Err(format!("Missing action for rule: {}", rule.id));
        }

        Ok(Parser {
            earley_parser: EarleyParser::new(grammar),
            forest,
        })
    }
}

impl<'a> ParserBuilder<'a, Sexpr> {
    pub fn for_sexpr(grammar_str: &str, start_rule: &str) -> Self {
        Self::new(grammar_str, start_rule)
            .unmapped_literal(|tok| Sexpr::Atom(tok.to_string()))
            .optional_empty(|| Sexpr::List(vec![]))
            .list_empty(|| Sexpr::List(vec![]))
            .list_action(|mut list, items| {
                if let Sexpr::List(ref mut v) = list {
                    v.extend(items);
                }
                list
            })
            .default_action(|_, mut nodes| {
                if nodes.len() == 1 {
                    nodes.swap_remove(0)
                } else {
                    Sexpr::List(nodes)
                }
            })
    }
}

pub struct Parser<'a, T: Clone + 'a> {
    earley_parser: EarleyParser,
    forest: EarleyForest<'a, T>,
}

impl<'a, T: Clone + 'a> Parser<'a, T> {
    pub fn parse<I, S>(&self, tokenizer: I) -> Result<T, String>
    where
        I: Iterator<Item = S>,
        S: AsRef<str> + std::fmt::Debug,
    {
        let trees = self.earley_parser.parse(tokenizer)?;
        let mut results = self.forest.eval_all(&trees)?;
        if results.len() > 1 {
            return Err("Ambiguous grammar: multiple parse trees found".to_string());
        }
        results
            .pop()
            .ok_or_else(|| "No parse tree found".to_string())
    }

    pub fn parse_all<I, S>(&self, tokenizer: I) -> Result<Vec<T>, String>
    where
        I: Iterator<Item = S>,
        S: AsRef<str> + std::fmt::Debug,
    {
        let trees = self.earley_parser.parse(tokenizer)?;
        self.forest.eval_all(&trees)
    }

    pub fn parse_sexpr<I, S>(&self, tokenizer: I) -> Result<Vec<Sexpr>, String>
    where
        I: Iterator<Item = S>,
        S: AsRef<str> + std::fmt::Debug,
    {
        let trees = self.earley_parser.parse(tokenizer)?;
        let mut sexpr_forest = EarleyForest::<Sexpr>::new(|_sym, tok| Sexpr::Atom(tok.to_string()));
        for rule in &self.earley_parser.grammar.rules {
            sexpr_forest.action(&rule.id, |mut nodes| {
                if nodes.len() == 1 {
                    nodes.swap_remove(0)
                } else {
                    Sexpr::List(nodes)
                }
            });
        }
        sexpr_forest.eval_all(&trees)
    }
}
