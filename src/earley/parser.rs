use earley::symbol::Symbol;
use earley::items::{Item, StateSet, Trigger};
use earley::grammar::Grammar;
use earley::Lexer;

#[derive(PartialEq, Debug)]
pub enum ParseError {
    BadInput,
    PartialParse,
}

pub struct EarleyParser {
    pub g: Grammar
}

#[derive(Debug)]
pub struct ParseState {
    pub states: Vec<StateSet>,
    pub input: Vec<String>,
}

impl EarleyParser {
    pub fn new(grammar: Grammar) -> EarleyParser { EarleyParser{g: grammar} }

    // TODO: leave scan loop for the end
    pub fn parse(&self, tok: &mut Lexer) -> Result<ParseState, ParseError> {
        let mut tokens = Vec::new();
        // Populate S0 by building items for each start rule
        let mut states = Vec::new();
        states.push(self.g.rules(self.g.start())
                          .map(|rule| Item::new(rule.clone(), 0, 0, 0))
                          .collect::<StateSet>());
        let mut i = 0;
        while i < states.len() {
            let input = tok.next();
            // accumulate tokens
            if let Some(ref input) = input {
                tokens.push(input.to_string());
            }

            let mut item_idx = 0;
            while item_idx < states[i].len() {
                let item = states[i][item_idx].clone();

                match item.next_symbol() {
                    // prediction, insert items for all rules named like this nonterm
                    Some(&Symbol::NonTerm(ref name)) => {
                        for rule in self.g.rules(&name) {
                            states[i].push(Item::new(rule.clone(), 0, i, i));
                            // trigger magical completion for nullable rules
                            if self.g.is_nullable(rule.name()) {
                                states[i].push(
                                    Item::new( // TODO: should use new2 ?
                                    item.rule.clone(), item.dot + 1, item.start, i));
                            }
                        }
                    },

                    // Found terminal, check input and populate S[i+1]
                    Some(&Symbol::Terminal(_, ref testfn)) => if let Some(ref input) = input {
                        if testfn(&input) {
                            if states.len() <= i + 1 {
                                assert_eq!(states.len(), i + 1);
                                states.push(StateSet::new());
                            }
                            states[i+1].push(Item::new2(
                                item.rule.clone(), item.dot+1, item.start, i+1,
                                (item.clone(), Trigger::Scan(input.to_string()))));
                        }
                    },

                    // we reached the end of the item's rule, trigger completion
                    None => {
                        // go back to state where 'item' started and advance
                        // any item if its next symbol matches the current one's name
                        let parent_state = states[item.start].clone();
                        let parent_items = parent_state.iter().filter_map(|pitem|
                            match pitem.next_symbol() {
                                Some(sym) if sym.is_nonterm() &&
                                             *sym == *item.rule.name => Some(pitem),
                                _ => None
                            });
                        states[i].extend(parent_items.map(|pitem|
                            Item::new2(pitem.rule.clone(), pitem.dot + 1, pitem.start, i,
                                       (pitem.clone(), Trigger::Completion(item.clone())))
                        ));
                    },
                }
                item_idx += 1;
            }
            i += 1;
        }
        {
            if tokens.len() + 1 != states.len() {
                return Err(ParseError::PartialParse);
            }
            // Check that at least one item is a. complete, b. starts at the beginning
            // and c. that the name of the rule matches the starting symbol. It spans
            // the whole input because we search at the last stateset
            let last = try!(states.last().ok_or(ParseError::BadInput));
            if last.iter().filter(|item| item.start == 0 &&
                                         item.complete() &&
                                         item.rule.name == self.g.start
                ).count() < 1 {
                return Err(ParseError::BadInput);
            }
        }
        Ok(ParseState{states: states, input: tokens})
    }
}
