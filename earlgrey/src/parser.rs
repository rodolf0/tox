#![deny(warnings)]

use grammar::{Symbol, Grammar};
use items::{Item, StateSet};
use std::rc::Rc;


#[derive(PartialEq,Debug)]
pub struct ParseError;

pub struct EarleyParser {
    pub g: Grammar,
}

#[derive(Debug)]
pub struct ParseTrees(pub Vec<Rc<Item>>);

///////////////////////////////////////////////////////////////////////////////

impl EarleyParser {
    pub fn new(grammar: Grammar) -> EarleyParser { EarleyParser{g: grammar} }

    pub fn parse<S>(&self, tok: S) -> Result<ParseTrees, ParseError>
            where S: Iterator<Item=String> { self._parse(tok, false) }

    pub fn debug<S>(&self, tok: S) -> Result<ParseTrees, ParseError>
            where S: Iterator<Item=String> { self._parse(tok, true) }

    fn _parse<S>(&self, mut tok: S, debug: bool)
            -> Result<ParseTrees, ParseError> where S: Iterator<Item=String> {

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
                    .advanced_by_scan(&lexeme, idx+1)
                    .into_iter()
                    .collect();
                states.push(scans);
            }
        }

        // Verbose, debug state-sets
        if debug {
            for (idx, stateset) in states.iter().enumerate() {
                eprintln!("=== {} ===", idx);
                for item in stateset.iter() { eprintln!("{:?}", item); }
            }
            eprintln!("=========");
        }

        // Check that at least one item is a. complete, b. starts at the idx 0,
        // and c. that the name of the rule matches the starting symbol.
        // It spans the whole input because we search at the last stateset
        let parse_trees: Vec<_> = states.pop().ok_or(ParseError)?
            .into_iter()
            .filter(|item| item.start == 0 && item.complete() &&
                           item.rule.head == self.g.start)
            .collect();

        if parse_trees.is_empty() {
            return Err(ParseError);
        }
        Ok(ParseTrees(parse_trees))
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    extern crate lexers;
    use grammar::GrammarBuilder;
    use self::lexers::{Scanner, DelimTokenizer};
    use super::{EarleyParser, ParseError};

    #[test]
    fn partial_parse() {
        let g = GrammarBuilder::default()
            .nonterm("Start")
            .terminal("+", |n| n == "+")
            .rule("Start", &["+", "+"])
            .into_grammar("Start")
            .expect("Bad Grammar");
        let mut input = DelimTokenizer::scanner("+++", "+", false);
        let out = EarleyParser::new(g).parse(&mut input);
        assert_eq!(out.unwrap_err(), ParseError);
    }

    #[test]
    fn badparse() {
        let g = GrammarBuilder::default()
          .nonterm("Sum")
          .nonterm("Num")
          .terminal("Number", |n| n.chars().all(|c| "1234".contains(c)))
          .terminal("[+-]", |n| n.len() == 1 && "+-".contains(n))
          .rule("Sum", &["Sum", "[+-]", "Num"])
          .rule("Sum", &["Num"])
          .rule("Num", &["Number"])
          .into_grammar("Sum")
          .expect("Bad Grammar");
        let mut input = DelimTokenizer::scanner("1+", "+*", false);
        let out = EarleyParser::new(g).parse(&mut input);
        assert_eq!(out.unwrap_err(), ParseError);
    }

    #[test]
    fn grammar_ambiguous() {
        // S -> SS | b
        let grammar = GrammarBuilder::default()
          .nonterm("S")
          .terminal("b", |n| n == "b")
          .rule("S", &["S", "S"])
          .rule("S", &["b"])
          .into_grammar("S")
          .expect("Bad Grammar");
        // Earley's corner case that generates spurious trees for bbb
        let mut input = DelimTokenizer::scanner("b b b", " ", true);
        EarleyParser::new(grammar).parse(&mut input)
            .expect("Broken Parser");
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
        let mut input = DelimTokenizer::scanner("1+2", "+", false);
        EarleyParser::new(grammar).parse(&mut input)
            .expect("Broken Parser");
    }

    #[test]
    fn test_right_recurse() {
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
        let mut input = DelimTokenizer::scanner("1^2", "^", false);
        EarleyParser::new(grammar).parse(&mut input)
            .expect("Broken Parser");
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
        let mut input = DelimTokenizer::scanner("", "-", false);
        EarleyParser::new(grammar).parse(&mut input)
            .expect("Broken Parser");
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
        let mut input = Scanner::from_buf("".split_whitespace()
                                          .map(|s| s.to_string()));
        EarleyParser::new(grammar).parse(&mut input)
            .expect("Broken Parser");
    }
}
