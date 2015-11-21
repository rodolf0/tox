use std::cmp::Ordering;
use earley::{NonTerminal, Symbol, Item, Grammar, RevTable};
use earley::Lexer;
use earley::Subtree;
use earley::uniqvec::UniqVec;
use std::rc::Rc;
use std::collections::VecDeque;

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

        // Outer loop goes over each stateset
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
        // if we want to pares partial input, we can't assert next->is_none,
        // we need to count how many tokens we've read and check the that the
        // start rule has reached the same length
        assert!(states.len() == state_idx && tok.next().is_none());
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

    fn helper(&self, states: &Vec<StateSet>, theroot: &Item, strt: usize) -> Subtree {
        let mut ret = VecDeque::new();
        let mut state_idx = strt;
        for needle in theroot.rule.spec.iter().rev() {
            //println!("Searching for {:?} completed at {}", needle, state_idx);
            match &**needle {
                &Symbol::NT(ref nt) => {
                    let prev = states[state_idx].iter()
                        .filter(|item| item.complete()
                                && item.rule.name == *needle).next().unwrap();
                    //println!("{}: {:?}", state_idx, prev);
                    let subtree = self.helper(states, prev, state_idx);
                    state_idx = prev.start;
                    ret.push_front(subtree);
                },
                &Symbol::T(ref t) => {
                    state_idx -= 1;
                    //println!("{}: needle={:?}", state_idx, t);
                    ret.push_front(Subtree::Node(needle.clone()));
                }
            }
        }
        Subtree::Children(ret)
    }

    pub fn build_tree(&self, states: Vec<StateSet>) {
        let root = states.last().unwrap().iter()
            .filter(|item| item.complete() && item.start == 0 &&
                    item.rule.name == self.g.start).next().unwrap();
        //println!("{:?}", root);
        let tree = self.helper(&states, root, states.len() - 1);
        println!("{:?}", tree);
    }

    //fn prediction(&self, s_i: &StateSet, sym: NonTerminal)

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
                Some(n @ &Symbol::NT(_)) if *n == *item.rule.name => Some(orig_item),
                _ => None
            });
        // copy over matching items to new state
        s_i.extend(matching_items.map(|orig_item| Item{
            rule: orig_item.rule.clone(),
            start: orig_item.start,
            dot: orig_item.dot+1
        }));
    }

    pub fn build_forest(&self, state: &Vec<StateSet>) {
        let revtable = self.build_revtable(state);

        //let mut forest = Vec::new();

        //let rules = revtable.get(0, 9, "Sum");
        //for rule in rules {
            //let mut tree = Subtree::Node(Vec::new());

            //// (0->9) Sum -> Sum [+-] Product
            //for sym in rule.spec.iter() {

                //let childs = build_forest_helper(&revtable, start, end, sym.name());
            //}
        //}
    }

    //fn build_forest_helper(&self, revtable: &RevTable, start: usize, max: usize, name: &str) -> Vec<_> {
        //let rows = revtable.get(start, name);
        //for (start, rule, end) in rows {
        //}
    //}

    pub fn build_revtable(&self, state: &Vec<StateSet>) -> RevTable {
        let mut revtable = RevTable::new();
        // Reveres states so we can search for trees from the beginning.
        // We only care about complete items, we'll store (start, rule, end)
        for (state_idx, stateset) in state.iter().enumerate() {
            for item in stateset.iter() {
                if item.complete() {
                    revtable.push((item.start, item.rule.clone(), state_idx));
                }
            }
        }
        // OPTIONAL: prioritize rules according to grammar, so ambiguous
        // grammars show parse trees in that order
        self.sort_rule_priorities(&mut revtable);
        revtable
    }

    // OPTIONAL: see build_revtable
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
