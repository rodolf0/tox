use types::{Symbol, StateSet, Grammar};
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

const DEBUG_STATESETS: bool = false;

impl EarleyParser {
    pub fn new(grammar: Grammar) -> EarleyParser { EarleyParser{g: grammar} }

    pub fn parse(&self, tok: &mut Scanner<String>) -> Result<Vec<StateSet>, ParseError> {
        let mut states = Vec::new();
        // Populate S0: add items for each rule matching the start symbol
        states.push(StateSet::from_iter(self.g.predict_new(&self.g.start(), 0)));

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
                        Some(&Symbol::NonTerm(ref name)) =>
                            states[state_idx].extend(
                                self.g.predict_new(name.as_ref(), state_idx)),
                        // Completion
                        None => {
                            // go back to state where 'item' started and advance
                            // any item if its next symbol matches the current one's name
                            let completions = states[item.start()].completed_by(&item, state_idx);
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
                let scans = states[state_idx].advanced_by_scan(&lexeme, state_idx+1);
                states.push(StateSet::from_iter(scans));
                assert_eq!(states.len(), state_idx + 2);
            }
            state_idx += 1;
        }

        // Verbose, debug state-sets
        if DEBUG_STATESETS {
            for (idx, stateset) in states.iter().enumerate() {
                println!("=== {} ===", idx);
                for item in stateset.iter() { println!("{:?}", item); }
            }
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
        // TODO: return Vec<Rc<Item>>
        Ok(states)
    }
}
