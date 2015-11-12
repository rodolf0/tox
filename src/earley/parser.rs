use std::cmp::Ordering;
use earley::{NonTerminal, Symbol, Item, Grammar, RevTable};
use earley::Lexer;
use earley::uniqvec::UniqVec;

pub type StateSet = UniqVec<Item>;

#[derive(PartialEq, Debug)]
pub enum ParseError {
    BadStartRule,
    BadInput,
}

pub struct EarleyParser {
    g: Grammar
}

impl EarleyParser {
    pub fn new(grammar: Grammar) -> EarleyParser { EarleyParser{g: grammar} }

    pub fn parse(&self, tok: &mut Lexer) -> Result<Vec<StateSet>, ParseError> {
        // Build S0 state building items out of each start rule
        let mut states = Vec::new();
        states.push(self.g.rules(self.g.start.name())
                    .map(|r| Item{rule: r.clone(), start: 0, dot: 0})
                    .collect::<StateSet>());
        if states[0].len() < 1 {
            return Err(ParseError::BadStartRule);
        }

        // Outere loop goes over each stateset
        let mut state_idx = 0;
        while state_idx < states.len() {
            let input = tok.next();
            // Inner loop goes over each item in a stateset
            let mut item_idx = 0;
            while item_idx < states[state_idx].len() {
                // For each item check if we need to predict/scan/complete
                let item = states[state_idx][item_idx].clone();
                match item.next_symbol() {

                    // Found non-terminal, do a prediction
                    Some(&Symbol::NT(ref nonterm)) => {
                        self.prediction(&mut states[state_idx], nonterm, &item, state_idx);
                    },

                    // Found terminal, scan the input to check if it matches
                    Some(&Symbol::T(ref terminal)) => {
                        if let Some(input) = input.clone() {
                            if terminal.check(&input) {
                                let new_item = Item{
                                    rule: item.rule.clone(),
                                    start: item.start,
                                    dot: item.dot+1
                                };
                                if state_idx + 1 >= states.len() {
                                    assert_eq!(state_idx + 1, states.len());
                                    states.push(StateSet::new());
                                }
                                states[state_idx + 1].push(new_item);
                            }
                        }
                    },

                    // we reached the end of the item's rule, trigger completion
                    None => {
                        let s_parent = states[item.start].clone();
                        self.completion(&mut states[state_idx], &s_parent, &item);
                    }
                }
                item_idx += 1;
            }
            state_idx += 1;
        }
        assert!(states.len() == state_idx);
        self.check_states(states)
    }

    fn check_states(&self, states: Vec<StateSet>) -> Result<Vec<StateSet>, ParseError> {
        {
            let last = try!(states.last().ok_or(ParseError::BadInput));
            // Check that at least one item is a. complete, b. starts at the beginning
            // and c. that the name of the rule matches the starting symbol
            if last.iter().filter(|item| item.complete() && item.start == 0 &&
                                   item.rule.name == self.g.start).count() < 1 {
                return Err(ParseError::BadInput);
            }
            // if the last state didn't contain any valid completions and we're
            // interested in partial parses (eg: headers) we can check  states
        }
        Ok(states)
    }

    // Symbol after fat-dot is NonTerm. Add the derived rules to current set
    fn prediction(&self, s_i: &mut StateSet, next_sym: &NonTerminal, item: &Item, start: usize) {
        for rule in self.g.rules(next_sym.name()) {
            s_i.push(Item{rule: rule.clone(), start: start, dot: 0});
            // trigger magical completion for nullable rules
            if self.g.nullable.contains(rule.name.name()) {
                s_i.push(Item{rule: item.rule.clone(),
                              start: item.start, dot: item.dot + 1});
            }
        }
    }

    // fat-dot at end of rule. Successful partial parse. Add parents to current
    fn completion(&self, s_i: &mut StateSet, s_parent: &StateSet, item: &Item) {
        // go over the parent state checking for items whose next symbol matches
        let matching_items = s_parent.iter()
            .filter_map(|orig_item| match orig_item.next_symbol() {
                Some(n @ &Symbol::NT(_)) if *item.rule.name == *n => Some(orig_item),
                _ => None
            });
        // copy over matching items to new state
        s_i.extend(matching_items.map(|orig_item| Item{
            rule: orig_item.rule.clone(),
            start: orig_item.start,
            dot: orig_item.dot+1
        }));
    }

    pub fn build_forest(&self, state: &Vec<StateSet>, item: &Item) {
        let revtable = self.build_revtable(state);
    }

    fn _build_forest(&self) {
    }

    fn build_revtable(&self, state: &Vec<StateSet>) -> RevTable {
        let mut revtable = RevTable::new();
        // Reveres states so we can search for trees from the beginning.
        // We only care about complete items, we'll store their rule and match length
        // and we'll index them according to starting point
        for (state_idx, stateset) in state.iter().enumerate() {
            for item in stateset.iter() {
                if item.complete() {
                    revtable.push((item.start, item.rule.clone(), state_idx));
                }
            }
        }
        self.sort_rule_priorities(&mut revtable); // THIS MAY NOT BE NEEDED
        revtable
    }

    // THIS MAY NOT BE NEEDED
    fn sort_rule_priorities(&self, revtable: &mut RevTable) {
        // resolving ambiguities:
        revtable.sort_by(|a, b| {
            // sort by start-point
            match a.0.cmp(&b.0) {
                Ordering::Equal => {
                    // these rules are guaranteed to exist since we inserted them
                    // sort according to appearance in grammar
                    let ax = self.g.rules.iter().position(|r| *r == a.1);
                    let bx = self.g.rules.iter().position(|r| *r == b.1);
                    match ax.unwrap().cmp(&bx.unwrap()) {
                        // sort by longest match first
                        Ordering::Equal => b.2.cmp(&a.2),
                        other => other,
                    }
                },
                other => other,
            }
        });
    }
}
