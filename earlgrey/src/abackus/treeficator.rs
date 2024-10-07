#![deny(warnings)]

use super::ebnf::ParserBuilder;
use crate::earley::{EarleyParser, EarleyForest};
use std::fmt::Debug;

#[derive(Clone,Debug)]
pub enum Sexpr {
    Atom(String),
    List(Vec<Sexpr>),
}

#[derive(Debug,Clone,PartialEq)]
pub enum Tree {
    // 1st element of each option is the matched rule
    // ("[+-]", "+")
    Leaf(String, String),
    // ("E -> E [+-] E", [...])
    Node(String, Vec<Tree>),
}

impl Sexpr {
    pub fn print(&self) -> String {
        let mut out = String::new();
        self.print_helper("", &mut out);
        out
    }

    fn print_helper(&self, indent: &str, out: &mut String) {
        match *self {
            Sexpr::Atom(ref lexeme) =>
                *out += &format!("\u{2500} {}\n", lexeme),
            Sexpr::List(ref subn) => {
                let (first, rest) = subn.split_first().unwrap();
                let (last, rest) = rest.split_last().unwrap();
                *out += &format!("\u{252c}");
                first.print_helper(&format!("{}\u{2502}", indent), out);
                for mid in rest {
                    *out += &format!("{}\u{251c}", indent);
                    mid.print_helper(&format!("{}\u{2502}", indent), out);
                }
                *out += &format!("{}\u{2570}", indent);
                last.print_helper(&format!("{} ", indent), out);
            }
        }
    }
}

impl ParserBuilder {
    pub fn treeficator<SI>(self, grammar: &str, start: &str)
        -> impl Fn(SI) -> Result<Vec<Tree>, String>
        where SI: Iterator, SI::Item: AsRef<str> + Debug
    {
        // User may pre-plug grammar (self.0) with terminals
        // 1. build a parser for user's grammar
        let grammar = ParserBuilder::parse_grammar(self.user_gb, grammar)
            .unwrap_or_else(|e| panic!("treeficator error: {:?}", e))
            .into_grammar(start)
            .unwrap_or_else(|e| panic!("treeficator error: {:?}", e));
        // 2. build evaler that builds trees when executing semantic actions
        let mut tree_builder = EarleyForest::new(
            |sym, tok| Tree::Leaf(sym.to_string(), tok.to_string()));
        for rule in grammar.rules.iter().map(|r| r.to_string()) {
            tree_builder.action(
                &rule.clone(), move |nodes| Tree::Node(rule.clone(), nodes));
        }
        // 3. make function that parses strings into trees
        let parser = EarleyParser::new(grammar);
        move |tokenizer| tree_builder.eval_all(&parser.parse(tokenizer)?)
    }

    pub fn sexprificator<SI>(self, grammar: &str, start: &str)
        -> impl Fn(SI) -> Result<Vec<Sexpr>, String>
        where SI: Iterator, SI::Item: AsRef<str> + Debug
    {
        // User may pre-plug grammar (self.0) with terminals
        // 1. build a parser for user's grammar
        let grammar = ParserBuilder::parse_grammar(self.user_gb, grammar)
            .unwrap_or_else(|e| panic!("treeficator error: {:?}", e))
            .into_grammar(start)
            .unwrap_or_else(|e| panic!("treeficator error: {:?}", e));
        // 2. build evaler that builds trees when executing semantic actions
        let mut tree_builder = EarleyForest::new(
            |_, tok| Sexpr::Atom(tok.to_string()));
        for rule in &grammar.rules {
            tree_builder.action(&rule.to_string(),
                move |mut nodes| match nodes.len() {
                    1 => nodes.swap_remove(0),
                    _ => Sexpr::List(nodes),
                });
        }
        // 3. make function that parses strings into trees
        let parser = EarleyParser::new(grammar);
        move |tokenizer| tree_builder.eval_all(&parser.parse(tokenizer)?)
    }
}
