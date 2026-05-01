use crate::earley::{EarleyForest, EarleyParser};
use crate::ebnf::EbnfGrammarParser;
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
    empty_action: Option<Rc<dyn Fn() -> T + 'a>>,
    list_action: Option<Rc<dyn Fn(T, T) -> T + 'a>>,
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
            empty_action: None,
            list_action: None,
            unmapped_literal: None,
        }
    }

    pub fn terminal(mut self, name: &str, parse: impl Fn(&str) -> Option<T> + 'static) -> Self {
        let p = Rc::new(parse);
        self.terminals.insert(name.to_string(), p.clone());
        self.terminal_preds.insert(name.to_string(), Rc::new(move |s| p(s).is_some()));
        self
    }

    pub fn literal(mut self, name: &str, value: T) -> Self {
        self.literals.insert(name.to_string(), value);
        self
    }

    pub fn unmapped_literal(mut self, parse: impl Fn(&str) -> T + 'static) -> Self {
        self.unmapped_literal = Some(Rc::new(parse));
        self
    }

    pub fn action(mut self, rule: &str, action: impl Fn(Vec<T>) -> T + 'static) -> Self {
        self.actions.insert(rule.to_string(), Rc::new(action));
        self
    }

    pub fn default_action(mut self, action: impl Fn(&str, Vec<T>) -> T + 'static) -> Self {
        self.default_action = Some(Rc::new(action));
        self
    }

    pub fn empty_action(mut self, action: impl Fn() -> T + 'static) -> Self {
        self.empty_action = Some(Rc::new(action));
        self
    }

    pub fn list_action(mut self, action: impl Fn(T, T) -> T + 'static) -> Self {
        self.list_action = Some(Rc::new(action));
        self
    }

    pub fn build(self) -> Result<Parser<'a, T>, String> {
        // Parse the user's EBNF like grammar and build an internal representation.
        let mut ebnf_parser = EbnfGrammarParser::new(&self.grammar_str, &self.start_rule);
        for (name, pred) in self.terminal_preds.iter() {
            let p = pred.clone();
            ebnf_parser = ebnf_parser.plug_terminal(name, move |s| p(s));
        }
        for name in self.literals.keys() {
            let n = name.clone();
            ebnf_parser = ebnf_parser.plug_terminal(name, move |s| s == n);
        }
        let grammar = ebnf_parser.into_grammar()?;

        // Validate that all symbols in the grammar have been registered.
        for rule in &grammar.rules {
            for sym in &rule.spec {
                if sym.is_terminal() && !self.terminals.contains_key(sym.name()) && !self.literals.contains_key(sym.name()) && self.unmapped_literal.is_none() {
                    return Err(format!("Missing terminal parser for: {}", sym.name()));
                }
            }
        }
        // Validate explicitly registered actions match a real rule
        for k in self.actions.keys() {
            if !grammar.rules.iter().any(|r| &r.to_string() == k) {
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
            let rule_str = rule.to_string();
            // map semantic action for each rule in the grammar.
            if let Some(action) = self.actions.get(&rule_str) {
                let a = action.clone();
                forest.action(&rule_str, move |v| a(v));
                continue;
            }
            // Fallback to 'empty_action' if the rule is an epsilon production.
            if rule.spec.is_empty() {
                if let Some(ref empty_a) = self.empty_action {
                    let a = empty_a.clone();
                    forest.action(&rule_str, move |_| a());
                    continue;
                } else {
                    return Err(format!("Missing action for empty rule: {}", rule_str));
                }
            }
            // Repetition like {x} generates 2 auxiliary rules: {x} -> x {x} and {x} -> [].
            // The user can provide a 'list_action' to combine multiple items into a list.
            if rule.head.starts_with('{') && rule.spec.len() >= 2 {
                #[allow(clippy::collapsible_if)]
                if let Some(ref list_a) = self.list_action {
                    let a = list_a.clone();
                    forest.action(&rule_str, move |mut v| {
                        let list = v.pop().unwrap();
                        let item = v.remove(0); // If there are middle elements, this only takes the first and last
                        a(item, list)
                    });
                    continue;
                }
            }
            // Rules like [x] generates aux rules [x] -> x and [x] -> [].
            // The length-1 case can be defaulted, but the epsilon case needs
            // to be handled by 'empty_action' because some representation of
            // "nothing" is needed. Eg: TimeValue::Empty, Sexpr::List(vec![])...
            if (rule.head.starts_with('[') 
                || rule.head.starts_with('(') 
                || rule.head.starts_with('{')) && rule.spec.len() == 1 {
                forest.action(&rule_str, |mut v| v.remove(0));
                continue;
            }
            // Allow catching generic sequences, eg: "A -> B C D".
            // For example for putting all items into a list or struct
            if let Some(ref default_a) = self.default_action {
                let a = default_a.clone();
                let head = rule.head.clone();
                forest.action(&rule_str, move |v| a(&head, v));
                continue;
            }
            return Err(format!("Missing action for rule: {}", rule_str));
        }

        Ok(Parser {
            earley_parser: EarleyParser::new(grammar),
            forest,
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
        results.pop().ok_or_else(|| "No parse tree found".to_string())
    }

    pub fn parse_all<I, S>(&self, tokenizer: I) -> Result<Vec<T>, String>
    where
        I: Iterator<Item = S>,
        S: AsRef<str> + std::fmt::Debug,
    {
        let trees = self.earley_parser.parse(tokenizer)?;
        self.forest.eval_all(&trees)
    }
}
