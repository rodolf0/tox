#![deny(warnings)]

extern crate earlgrey;

use ebnf::ParserBuilder;
use self::earlgrey::{EarleyParser, EarleyForest, Error};

#[derive(Clone,Debug)]
pub enum Sexpr {
    Atom(String),
    List(Vec<Sexpr>),
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

pub type Treeresult = Result<Vec<Sexpr>, Error>;

impl ParserBuilder {
    // Build an evaluator that accepts grammar and builds Sexpr's from input
    pub fn treeficator<'a>(self, start: &str, grammar: &'a str)
            -> Box<Fn(&mut Iterator<Item=String>)->Treeresult + 'a> {
        // 1. build a grammar builder for the user's grammar
        let grammar = ParserBuilder::builder(self.0, grammar, false)
            .unwrap_or_else(|e| panic!("treeficator error: {:?}", e))
            .into_grammar(start)
            .unwrap_or_else(|e| panic!("treeficator error: {:?}", e));

        // 2. Add semantic actions that flatten the parse tree
        let mut ev = EarleyForest::new(|_, tok| Sexpr::Atom(tok.to_string()));
        for rule in grammar.str_rules() {
            ev.action(&rule, move |mut nodes| match nodes.len() {
                1 => nodes.swap_remove(0),
                _ => Sexpr::List(nodes),
            });
        }

        // 3. return a function that applies the parser+evaler to any input
        let parser = EarleyParser::new(grammar);
        Box::new(move |tokenizer| ev.eval_all(&parser.parse(tokenizer)?))
    }
}
