use earley::types::*;

pub struct EarleyParser {
    grammar: Grammar,
}

impl EarleyParser {
    //pub fn parse(grammar: &Grammar, input: &mut Lexer) {
    pub fn new(grammar: Grammar) -> EarleyParser {
        EarleyParser{grammar: grammar}
    }

    pub fn parse(&self) {
        let mut state = Vec::new();

        // 1. Create S0
        if let Some(s0) = self.grammar.rules.get(&self.grammar.start) {
            let it = s0.iter().map(|r| Item{rule: r.clone(), start: 0, dot: 0});
            state.push(it.collect::<StateSet>());
        } else {
            panic!("Shit!");
        }

        let mut i = 0;
        while i < state.len() {
            let cur_state = &mut state[i];
            let mut j = 0;
            while j < cur_state.len() {
                let cur_item = cur_state[j].clone();

                match cur_item.next_symbol() {
                    Some(&Symbol::NonTerminal(ref nt)) => {
                        //self.prediction(nt, cur_state, i);
                    },
                        /*
                    &Symbol::Terminal(ref term) => {
                        //self.scan(&cur_item, term, "", next_state);
                    },
                    None => () //completion(),
                    */
                    _ => panic!("todo")
                }
            }
        }
    }

    // Symbol after fat-dot is NonTerm. Add the derived rules to current set
    fn prediction(&self, sym: &NonTerminal, s_i: &mut StateSet, i: usize) {
        if let Some(rules) = self.grammar.rules.get(sym) {
            s_i.extend(rules.iter()
                       .map(|r| Item{rule: r.clone(), start: i, dot: 0}));
        }
    }

    // Symbol after fat-dot is Term. If input matches symbol add to next state
    fn scan(&self, item: &Item,
            sym: &Terminal, input: &str, s_next: &mut StateSet) {
        if sym.check(input) {
            s_next.push(Item{
                rule: item.rule.clone(),
                start: item.start,
                dot: item.dot+1
            });
        }
    }

    // fat-dot at end of rule. Successful partial parse. Add parents to current
    fn completion(&self, item: &Item, s_i: &mut StateSet, s_parent: &StateSet) {
        // check next symbol for each item and keep ones that next sym matches
        let copy_items = s_parent.iter()
            .filter_map(|orig_item| match orig_item.next_symbol() {
                Some(&Symbol::NonTerminal(ref next))
                    if *next == item.rule.name => Some(orig_item),
                // TODO: what about Terminals ?
                _ => None
            });
        // copy over matching items to new state
        s_i.extend(copy_items.map(|orig_item| Item{
            rule: orig_item.rule.clone(),
            start: orig_item.start,
            dot: orig_item.dot+1
        }));
    }
}
