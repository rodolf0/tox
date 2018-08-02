#![deny(warnings)]

extern crate earlgrey;

use ebnf::ParserBuilder;
use self::earlgrey::{EarleyForest, Error};

#[derive(Clone,Debug)]
pub enum Sexpr {
    Atom(String),
    List(Vec<Sexpr>),
}

#[derive(Debug,Clone,PartialEq)]
pub enum Tree {
    // ("[+-]", "+")
    Leaf(String, String),
    // ("E -> E [+-] E", [...])
    Node(String, Vec<Tree>),
}

impl Sexpr {
    pub fn print(&self) { self.print_helper("") }

    fn print_helper(&self, level: &str) {
        match self {
            &Sexpr::Atom(ref lexeme) => println!("{}`-- {:?}", level, lexeme),
            &Sexpr::List(ref subn) => {
                println!("{}`--", level);
                if let Some((last, rest)) = subn.split_last() {
                    let l = format!("{}  |", level);
                    for n in rest { n.print_helper(&l); }
                    let l = format!("{}   ", level);
                    last.print_helper(&l);
                }
            }
        }
    }
}

impl ParserBuilder {
    pub fn treeficator(grammar: &str, start: &str)
        -> impl Fn(&mut Iterator<Item=String>) -> Result<Vec<Sexpr>, Error>
    {
        // 1. build a parser for user's grammar
        let parser = Self::default().into_parser(&start, &grammar)
            .unwrap_or_else(|e| panic!("treeficator error: {:?}", e));
        // 2. build tree-evaler that consumes parser output and builds trees
        let mut tree_builder = EarleyForest::new(
            |_, tok| Sexpr::Atom(tok.to_string()));
        for rule in parser.g.str_rules() {
            tree_builder.action(&rule, move |mut nodes| match nodes.len() {
                1 => nodes.swap_remove(0),
                _ => Sexpr::List(nodes),
            });
        }
        // 3. make function that parses strings into trees
        move |tokenizer| tree_builder.eval_all(&parser.parse(tokenizer)?)
    }

    pub fn treeficator2<'a>(grammar: &str, start: &str)
        -> impl Fn(&mut Iterator<Item=String>) -> Result<Vec<Tree>, Error>
    {
        let parser = Self::default().into_parser(&start, &grammar)
            .unwrap_or_else(|e| panic!("treeficator error: {:?}", e));
        let mut tree_builder = EarleyForest::new(
            |sym, tok| Tree::Leaf(sym.to_string(), tok.to_string()));
        for rule in parser.g.str_rules() {
            tree_builder.action(&rule.to_string(),
                move |nodes| Tree::Node(rule.to_string(), nodes));
        }
        move |tokenizer| tree_builder.eval_all(&parser.parse(tokenizer)?)
    }
}
