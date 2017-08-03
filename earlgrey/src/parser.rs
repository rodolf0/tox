#![deny(warnings)]

use grammar::{Symbol, Grammar};
use items::{Item, StateSet};
use std::iter::FromIterator;
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
        let mut states = Vec::new();
        states.push(StateSet::from_iter(
                    self.g.rules_for(&self.g.start)
                        .into_iter()
                        .map(|r| Item::predict_new(&r, 0))));

        // 1. Go through states while there's more, will be one per input token
        let mut state_idx = 0;
        while states.len() > state_idx {

            // 2. predict/complete until this StateSet stops growing
            let mut prev_size = -1;
            while (states[state_idx].len() as isize) != prev_size {
                prev_size = states[state_idx].len() as isize;

                // 3. produce new items: run through all items in this StateSet
                let mut new_items = Vec::new();
                for itm in states[state_idx].iter() {
                    new_items.extend(match itm.next_symbol() {
                        // Prediction
                        Some(&Symbol::NonTerm(ref name)) =>
                            self.g.rules_for(name.as_ref())
                            .into_iter()
                            .map(|r| Item::predict_new(&r, state_idx))
                            .collect(),
                        // Completion
                        None => states[itm.start].completed_by(&itm, state_idx),
                        // Scans processed later
                        _ => Vec::new(),
                    });
                }
                states[state_idx].extend(new_items);
            }

            // 4. Scan input to populate Si+1
            if let Some(lexeme) = tok.next() {
                let scans =
                    states[state_idx].advanced_by_scan(&lexeme, state_idx+1);
                states.push(StateSet::from_iter(scans));
                assert_eq!(states.len(), state_idx + 2);
            }

            // 5. Mark this StateSet processed
            state_idx += 1;
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
        let parse_trees = states
            .last().ok_or(ParseError)?
            .filter_rule_head(self.g.start.as_ref())
            .filter(|item| item.start == 0 && item.complete())
            .cloned()
            .collect::<Vec<_>>();
        match parse_trees.is_empty() {
            true => Err(ParseError),
            false => Ok(ParseTrees(parse_trees)),
        }
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
        let g = GrammarBuilder::new()
            .symbol("Start")
            .symbol(("+", |n: &str| n == "+"))
            .rule("Start", &["+", "+"])
            .into_grammar("Start")
            .expect("Bad Grammar");
        let mut input = DelimTokenizer::from_str("+++", "+", false);
        let out = EarleyParser::new(g).parse(&mut input);
        assert_eq!(out.unwrap_err(), ParseError);
    }

    #[test]
    fn badparse() {
        let g = GrammarBuilder::new()
          .symbol("Sum")
          .symbol("Num")
          .symbol(("Number", |n: &str| n.chars().all(|c| "1234".contains(c))))
          .symbol(("[+-]", |n: &str| n.len() == 1 && "+-".contains(n)))
          .rule("Sum", &["Sum", "[+-]", "Num"])
          .rule("Sum", &["Num"])
          .rule("Num", &["Number"])
          .into_grammar("Sum")
          .expect("Bad Grammar");
        let mut input = DelimTokenizer::from_str("1+", "+*", false);
        let out = EarleyParser::new(g).parse(&mut input);
        assert_eq!(out.unwrap_err(), ParseError);
    }

    #[test]
    fn grammar_ambiguous() {
        // S -> SS | b
        let grammar = GrammarBuilder::new()
          .symbol("S")
          .symbol(("b", |n: &str| n == "b"))
          .rule("S", &["S", "S"])
          .rule("S", &["b"])
          .into_grammar("S")
          .expect("Bad Grammar");
        // Earley's corner case that generates spurious trees for bbb
        let mut input = DelimTokenizer::from_str("b b b", " ", true);
        EarleyParser::new(grammar).parse(&mut input)
            .expect("Broken Parser");
    }

    #[test]
    fn left_recurse() {
        // S -> S + N | N
        // N -> [0-9]
        let grammar = GrammarBuilder::new()
          .symbol("S")
          .symbol("N")
          .symbol(("[+]", |n: &str| n == "+"))
          .symbol(("[0-9]", |n: &str| "1234567890".contains(n)))
          .rule("S", &["S", "[+]", "N"])
          .rule("S", &["N"])
          .rule("N", &["[0-9]"])
          .into_grammar("S")
          .expect("Bad grammar");
        let mut input = DelimTokenizer::from_str("1+2", "+", false);
        EarleyParser::new(grammar).parse(&mut input)
            .expect("Broken Parser");
    }

    #[test]
    fn test_right_recurse() {
        // P -> N ^ P | N
        // N -> [0-9]
        let grammar = GrammarBuilder::new()
          .symbol("P")
          .symbol("N")
          .symbol(("[^]", |n: &str| n == "^"))
          .symbol(("[0-9]", |n: &str| "1234567890".contains(n)))
          .rule("P", &["N", "[^]", "P"])
          .rule("P", &["N"])
          .rule("N", &["[0-9]"])
          .into_grammar("P")
          .expect("Bad grammar");
        let mut input = DelimTokenizer::from_str("1^2", "^", false);
        EarleyParser::new(grammar).parse(&mut input)
            .expect("Broken Parser");
    }

    #[test]
    fn bogus_empty() {
        // A -> <empty> | B
        // B -> A
        let grammar = GrammarBuilder::new()
          .symbol("A")
          .symbol("B")
          .rule::<_, &str>("A", &[])
          .rule("A", &vec!["B"])
          .rule("B", &vec!["A"])
          .into_grammar("A")
          .expect("Bad grammar");
        let mut input = DelimTokenizer::from_str("", "-", false);
        EarleyParser::new(grammar).parse(&mut input)
            .expect("Broken Parser");
    }

    #[test]
    fn bogus_epsilon() {
        // Grammar for balanced parenthesis
        // P  -> '(' P ')' | P P | <epsilon>
        let grammar = GrammarBuilder::new()
          .symbol("P")
          .symbol(("(", |l: &str| l == "("))
          .symbol((")", |l: &str| l == ")"))
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
