#![deny(warnings)]

use crate::grammar::{Symbol, Grammar};
use crate::items::{Item, StateSet};
use std::rc::Rc;


#[derive(Debug,PartialEq)]
pub enum Error {
    ParseError,
    MissingAction(String),
    MissingSym(String),
    DuplicateSym(String),
    DuplicateRule(String),
}

pub struct EarleyParser {
    pub g: Grammar,
    debug: bool,
}

#[derive(Debug)]
pub struct ParseTrees(pub Vec<Rc<Item>>);

///////////////////////////////////////////////////////////////////////////////

impl EarleyParser {
    pub fn new(grammar: Grammar) -> EarleyParser {
        EarleyParser{g: grammar, debug: false}
    }

    pub fn parse<S, SI>(&self, mut tok: SI) -> Result<ParseTrees, Error>
            where S: AsRef<str>, SI: Iterator<Item=S> {

        // 0. Populate S0, add items for each rule matching the start symbol
        let s0: StateSet = self.g.rules_for(&self.g.start).into_iter()
                    .map(|r| Item::predict_new(&r, 0))
                    .collect();

        let mut states = vec![s0];

        // New states are generated from input stream (Scans)
        for idx in 0.. {
            if states.len() <= idx { break; }

            // Predic/Complete until no new Items are added to the StateSet
            loop {
                let (new_items, prev_item_count) = {
                    let state = &states[idx];
                    let new_items: Vec<Item> = state.iter()
                        .flat_map(|item| match item.next_symbol() {

                            // Prediction: add rules starting with next symbol
                            Some(Symbol::NonTerm(ref name)) =>
                                self.g.rules_for(name).into_iter()
                                    .map(|rule| Item::predict_new(&rule, idx))
                                    .collect(),

                            // Completion: add items with rules that completed
                            None =>
                                states[item.start].completed_at(item, idx),

                            // Scans: these will populate next state, ignore
                            Some(Symbol::Terminal(_, _)) => Vec::new(),

                        }).collect();

                    (new_items, state.len())
                };

                // keep adding new items to this StateSet until no more changes
                let state = &mut states[idx];
                state.extend(new_items);
                if state.len() == prev_item_count { break; }
            }

            // Bootstrap Si+1 next state with rules that accept the next token
            if let Some(lexeme) = tok.next() {
                let scans = states[idx]
                    .advanced_by_scan(lexeme.as_ref(), idx+1)
                    .into_iter()
                    .collect();
                states.push(scans);
            }
        }

        // Verbose, debug state-sets
        if self.debug {
            for (idx, stateset) in states.iter().enumerate() {
                eprintln!("=== {} ===", idx);
                for item in stateset.iter() { eprintln!("{:?}", item); }
            }
            eprintln!("=========");
        }

        // Check that at least one item is a. complete, b. starts at the idx 0,
        // and c. that the name of the rule matches the starting symbol.
        // It spans the whole input because we search at the last stateset
        let parse_trees: Vec<_> = states.pop().ok_or(Error::ParseError)?
            .into_iter()
            .filter(|item| item.start == 0 && item.complete() &&
                           item.rule.head == self.g.start)
            .collect();

        if parse_trees.is_empty() {
            return Err(Error::ParseError);
        }
        Ok(ParseTrees(parse_trees))
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::grammar::GrammarBuilder;
    use super::{EarleyParser, Error};

    fn good(parser: &EarleyParser, input: &str) {
        assert!(parser.parse(input.split_whitespace()).is_ok());
    }

    fn fail(parser: &EarleyParser, input: &str) {
        assert_eq!(parser.parse(input.split_whitespace()).unwrap_err(),
                   Error::ParseError);
    }

    #[test]
    fn partial_parse() {
        let grammar = GrammarBuilder::default()
            .nonterm("Start")
            .terminal("+", |n| n == "+")
            .rule("Start", &["+", "+"])
            .into_grammar("Start")
            .expect("Bad Grammar");
        let p = EarleyParser::new(grammar);
        fail(&p, "+ + +");
        good(&p, "+ +");
    }

    #[test]
    fn badparse() {
        let grammar = GrammarBuilder::default()
          .nonterm("Sum")
          .nonterm("Num")
          .terminal("Number", |n| n.chars().all(|c| "1234".contains(c)))
          .terminal("[+-]", |n| n.len() == 1 && "+-".contains(n))
          .rule("Sum", &["Sum", "[+-]", "Num"])
          .rule("Sum", &["Num"])
          .rule("Num", &["Number"])
          .into_grammar("Sum")
          .expect("Bad Grammar");
        let p = EarleyParser::new(grammar);
        fail(&p, "1 +");
    }

    #[test]
    fn grammar_ambiguous() {
        // Earley's corner case that generates spurious trees for bbb
        // S -> SS | b
        let grammar = GrammarBuilder::default()
          .nonterm("S")
          .terminal("b", |n| n == "b")
          .rule("S", &["S", "S"])
          .rule("S", &["b"])
          .into_grammar("S")
          .expect("Bad Grammar");
        let p = EarleyParser::new(grammar);
        good(&p, "b b b b");
        good(&p, "b b b"); // tricky case
        good(&p, "b b");
        good(&p, "b");
    }

    #[test]
    fn left_recurse() {
        // S -> S + N | N
        // N -> [0-9]
        let grammar = GrammarBuilder::default()
          .nonterm("S")
          .nonterm("N")
          .terminal("[+]", |n| n == "+")
          .terminal("[0-9]", |n| "1234567890".contains(n))
          .rule("S", &["S", "[+]", "N"])
          .rule("S", &["N"])
          .rule("N", &["[0-9]"])
          .into_grammar("S")
          .expect("Bad grammar");
        let p = EarleyParser::new(grammar);
        good(&p, "1 + 2");
        good(&p, "1 + 2 + 3");
        fail(&p, "1 2 + 3");
        fail(&p, "+ 3");
    }

    #[test]
    fn right_recurse() {
        // P -> N ^ P | N
        // N -> [0-9]
        let grammar = GrammarBuilder::default()
          .nonterm("P")
          .nonterm("N")
          .terminal("[^]", |n| n == "^")
          .terminal("[0-9]", |n| "1234567890".contains(n))
          .rule("P", &["N", "[^]", "P"])
          .rule("P", &["N"])
          .rule("N", &["[0-9]"])
          .into_grammar("P")
          .expect("Bad grammar");
        let p = EarleyParser::new(grammar);
        good(&p, "1 ^ 2");
        fail(&p, "3 ^ ");
        good(&p, "1 ^ 2 ^ 4");
        good(&p, "1 ^ 2 ^ 4 ^ 5");
        fail(&p, "1 2 ^ 4");
    }

    #[test]
    fn bogus_empty() {
        // A -> <empty> | B
        // B -> A
        let grammar = GrammarBuilder::default()
          .nonterm("A")
          .nonterm("B")
          .rule::<_, &str>("A", &[])
          .rule("A", &vec!["B"])
          .rule("B", &vec!["A"])
          .into_grammar("A")
          .expect("Bad grammar");
        let p = EarleyParser::new(grammar);
        good(&p, "");
        good(&p, " ");
        fail(&p, "X");
    }

    #[test]
    fn bogus_epsilon() {
        // Grammar for balanced parenthesis
        // P  -> '(' P ')' | P P | <epsilon>
        let grammar = GrammarBuilder::default()
          .nonterm("P")
          .terminal("(", |l| l == "(")
          .terminal(")", |l| l == ")")
          .rule("P", &["(", "P", ")"])
          .rule("P", &["P", "P"])
          .rule::<_, &str>("P", &[])
          .into_grammar("P")
          .expect("Bad grammar");
        let p = EarleyParser::new(grammar);
        good(&p, "");
        good(&p, "( )");
        good(&p, "( ( ) )");
        fail(&p, "( ) )");
        fail(&p, ")");
    }
}
