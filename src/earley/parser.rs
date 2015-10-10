use earley::{Terminal, NonTerminal, Symbol};
use earley::{Grammar, Item, StateSet};

#[derive(PartialEq, Debug)]
pub enum ParseError {
    BadStartRule,
}

pub struct EarleyParser {
    grammar: Grammar,
}

impl EarleyParser {

//pub fn parse(grammar: &Grammar, input: &mut Lexer) {
pub fn new(grammar: Grammar) -> EarleyParser {
    EarleyParser{grammar: grammar}
}

pub fn parse(&self) {
    let mut states = Vec::new();
    // 1. Create S0
    if let Some(s0) = self.grammar.rules.get(&self.grammar.start) {
        let it = s0.iter().map(|r| Item{rule: r.clone(), start: 0, dot: 0});
        states.push(it.collect::<StateSet>());
    } else {
        panic!("Shit!");
    }

    let mut state_idx = 0;
    while state_idx < states.len() {
        let mut item_idx = 0;
        while item_idx < states[state_idx].len() {

            let item = states[state_idx][item_idx].clone();

            match item.next_symbol() {
                // Found non-terminal, do a prediction
                Some(&Symbol::NT(ref nonterm)) => {
                    self.prediction(nonterm, &mut states[state_idx], state_idx);
                },
                // Found terminal, scan the input to check if it matches
                Some(&Symbol::T(ref terminal)) => {
                    let next_idx = state_idx + 1;
                    if states.len() < next_idx {
                        states.push(StateSet::new());
                    }
                    let next_state = states.get_mut(next_idx).unwrap();
                    // TODO: get the input from the lexer
                    self.scan(&item, terminal, "", next_state);
                },
                // we reached the end of the item's rule, trigger completion
                None => {
                    let s_parent = states[item.start].clone();
                    self.completion(&item, &mut states[state_idx], &s_parent);
                }
            }
            item_idx += 1;
        }
        state_idx += 1;
    }
}

// fat-dot at end of rule. Successful partial parse. Add parents to current
fn completion(&self, item: &Item, s_i: &mut StateSet, s_parent: &StateSet) {
    // go over the parent state checking for items whose next symbol matches
    let copy_items = s_parent.iter()
        .filter_map(|orig_item| match orig_item.next_symbol() {
            Some(n @ &Symbol::NT(_)) if *n == *item.rule.name => Some(orig_item),
            _ => None
        });
    // copy over matching items to new state
    s_i.extend(copy_items.map(|orig_item| Item{
        rule: orig_item.rule.clone(),
        start: orig_item.start,
        dot: orig_item.dot+1
    }));
}

// Symbol after fat-dot is NonTerm. Add the derived rules to current set
fn prediction(&self, symbol: &NonTerminal, s_i: &mut StateSet, i: usize) {
    let &NonTerminal(ref symbol) = symbol;
    if let Some(rules) = self.grammar.rules.get(symbol) {
        s_i.extend(rules.iter()
                   .map(|r| Item{rule: r.clone(), start: i, dot: 0}));
    }
}

// Symbol after fat-dot is Term. If input matches symbol add to next state
fn scan(&self, item: &Item,
        symbol: &Terminal, input: &str, s_next: &mut StateSet) {
    if symbol.check(input) {
        s_next.push(Item{
            rule: item.rule.clone(),
            start: item.start,
            dot: item.dot+1
        });
    }
}

}
