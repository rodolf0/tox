#![deny(warnings)]

use crate::earley::{EarleyForest, EarleyParser, Grammar};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum Sexpr {
    Atom(String),
    List(Vec<Sexpr>),
}

impl Sexpr {
    pub fn print(&self) -> String {
        let mut out = String::new();
        self.print_helper("", &mut out);
        out
    }

    fn print_helper(&self, indent: &str, out: &mut String) {
        match *self {
            Sexpr::Atom(ref lexeme) => *out += &format!("\u{2500} {}\n", lexeme),
            Sexpr::List(ref subn) => {
                if let Some((first, rest)) = subn.split_first() {
                    if let Some((last, rest)) = rest.split_last() {
                        *out += "\u{252c}";
                        first.print_helper(&format!("{}\u{2502}", indent), out);
                        for mid in rest {
                            *out += &format!("{}\u{251c}", indent);
                            mid.print_helper(&format!("{}\u{2502}", indent), out);
                        }
                        *out += &format!("{}\u{2570}", indent);
                        last.print_helper(&format!("{} ", indent), out);
                    } else {
                        *out += "\u{2500} \u{03b5}\n";
                    }
                } else {
                    *out += "\u{2500} \u{03b5}\n";
                }
            }
        }
    }
}

pub fn sexpr_parser<InputIter>(
    grammar: Grammar,
) -> Result<impl Fn(InputIter) -> Result<Vec<Sexpr>, String>, String>
where
    InputIter: Iterator,
    InputIter::Item: AsRef<str> + std::fmt::Debug,
{
    let mut tree_builder = EarleyForest::new(|_, tok| Sexpr::Atom(tok.to_string()));

    for rule in &grammar.rules {
        tree_builder.action(&rule.to_string(), move |mut nodes| match nodes.len() {
            1 => nodes.swap_remove(0),
            _ => Sexpr::List(nodes),
        });
    }

    let parser = EarleyParser::new(grammar);
    Ok(move |tokenizer| tree_builder.eval_all(&parser.parse(tokenizer)?))
}
