use types::{Symbol, Item, StateSet, Grammar};
use lexers::Scanner;
use std::iter::FromIterator;

#[derive(PartialEq, Debug)]
pub enum ParseError {
    BadInput,
    PartialParse,
}

pub struct EarleyParser {
    pub g: Grammar
}

#[derive(Debug)]
pub struct EarleyState {
    pub states: Vec<StateSet>,
    pub lexemes: Vec<String>,
}

impl EarleyParser {
    pub fn new(grammar: Grammar) -> EarleyParser { EarleyParser{g: grammar} }

    pub fn parse(&self, tok: &mut Scanner<String>) -> Result<EarleyState, ParseError> {
        let mut lexemes = Vec::new();
        let mut states = Vec::new();
        // Populate S0: add items for each rule matching the start symbol
        states.push(self.g.rules(self.g.start())
                          .map(|rule| Item::new(rule.clone(), 0, 0, 0))
                          .collect::<StateSet>());
        let mut state_idx = 0;
        while states.len() > state_idx {
            loop {
                // predict/complete until the stateset stops growing
                let num_items = states[state_idx].len();
                // iterate over all items in this stateset predict/complete
                let mut item_idx = 0;
                while states[state_idx].len() > item_idx {
                    let item = states[state_idx][item_idx].clone();
                    match item.next_symbol() {
                        // Prediction
                        Some(&Symbol::NonTerm(ref name)) => {
                            let predictions = self.g.rules(&name)
                                  .map(|rule| Item::predict_new(rule, state_idx));
                            states[state_idx].extend(predictions);
                        },
                        // Completion
                        None => {
                            // go back to state where 'item' started and advance
                            // any item if its next symbol matches the current one's name
                            let completions = states[item.start()].iter()
                                .filter(|source| item.can_complete(source))
                                .map(|source| Item::complete_new(source, &item, state_idx))
                                .collect::<Vec<_>>();
                            states[state_idx].extend(completions);
                        },
                        _ => () // process Scans later
                    }
                    item_idx += 1;
                }
                if num_items == states[state_idx].len() { break; } // no new items we're OK
            }
            // Scan input to populate Si+1
            if let Some(lexeme) = tok.next() {
                lexemes.push(lexeme.clone());
                let scans = states[state_idx].iter()
                    .filter(|item| item.can_scan(&lexeme))
                    .map(|item| Item::scan_new(&item, state_idx+1, &lexeme))
                    .collect::<Vec<_>>();
                states.push(StateSet::from_iter(scans));
                assert_eq!(states.len(), state_idx + 2);
            }
            state_idx += 1;
        }
        {
            // Check that at least one item is a. complete, b. starts at the beginning
            // and c. that the name of the rule matches the starting symbol. It spans
            // the whole input because we search at the last stateset
            let last = try!(states.last().ok_or(ParseError::BadInput));
            if last.filter_by_rule(self.g.start())
                   .filter(|item| item.start() == 0 && item.complete())
                   .count() < 1 {
                return Err(ParseError::BadInput);
            }
        }
        Ok(EarleyState{states: states, lexemes: lexemes})
    }
}
