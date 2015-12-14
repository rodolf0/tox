use earley::symbol::Symbol;
use earley::items::{Item, StateSet};
use earley::grammar::Grammar;

use earley::Lexer;
//use std::collections::VecDeque;
//use std::rc::Rc;

//#[derive(Debug)]
//pub enum Subtree {
    //Node(Rc<Symbol>),
    //Children(VecDeque<Subtree>),
//}

#[derive(PartialEq, Debug)]
pub enum ParseError {
    BadInput,
}

pub struct EarleyParser {
    g: Grammar
}

impl EarleyParser {
    pub fn new(grammar: Grammar) -> EarleyParser { EarleyParser{g: grammar} }

    pub fn parse(&self, tok: &mut Lexer) -> Result<Vec<StateSet>, ParseError> {
        // Populate S0 by building items for each start rule
        let mut states = Vec::new();
        states.push(self.g.rules(self.g.start())
                          .map(|rule| Item::new(rule.clone(), 0, 0))
                          .collect::<StateSet>());
        let mut i = 0;
        while i < states.len() {
            let input = tok.next();

            let mut item_idx = 0;
            while item_idx < states[i].len() {
                let item = states[i][item_idx].clone();

                match item.next_symbol() {
                    // prediction, insert items for all rules named like this nonterm
                    Some(&Symbol::NonTerm(ref name)) => {
                        for rule in self.g.rules(&name) {
                            states[i].push(Item::new(rule.clone(), 0, i));
                            // trigger magical completion for nullable rules
                            if self.g.is_nullable(rule.name()) {
                                states[i].push(Item::new(
                                    item.rule.clone(), item.dot + 1, item.start));
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
                            states[i+1].push(Item::new(
                                item.rule.clone(), item.dot+1, item.start));
                        }
                    },

                    // we reached the end of the item's rule, trigger completion
                    None => {
                        // go back to state where 'item' started and advance
                        // any item if its next symbol matches the current one's name
                        let parent_state = states[item.start].clone();  // TODO: no need to clone
                        let parent_items = parent_state.iter().filter_map(|pitem|
                            match pitem.next_symbol() {
                                Some(sym) if sym.is_nonterm() &&
                                             *sym == *item.rule.name => Some(pitem),
                                _ => None
                            });
                        states[i].extend(parent_items.map(|pitem| Item::new(
                            pitem.rule.clone(), pitem.dot + 1, pitem.start)));
                    },
                }
                item_idx += 1;
            }
            i += 1;
        }

        {
            // Check that at least one item is a. complete, b. starts at the beginning
            // and c. that the name of the rule matches the starting symbol
            let last = try!(states.last().ok_or(ParseError::BadInput));
            if last.iter().filter(|item| item.complete() && item.start == 0 &&
                                   item.rule.name == self.g.start).count() < 1 {
                return Err(ParseError::BadInput);
            }
        }
        Ok(states)
    }

    /*
    fn helper(&self, states: &Vec<StateSet>, theroot: &Item, strt: usize) -> Subtree {
        let mut ret = VecDeque::new();
        let mut state_idx = strt;
        for needle in theroot.rule.spec.iter().rev() {
            println!("Searching for {:?} completed at {}", needle, state_idx);
            match &**needle {
                &Symbol::NT(ref nt) => {
                    let prevs = states[state_idx].iter()
                        .filter(|item| item.complete()
                                && item.rule.name == *needle);
                    for prev in prevs {
                        println!("{}: {:?}", state_idx, prev);
                        let subtree = self.helper(states, prev, state_idx);
                        state_idx = prev.start;
                        ret.push_front(subtree);
                    }
                },
                &Symbol::T(ref t) => {
                    state_idx -= 1;
                    println!("{}: needle={:?}", state_idx, t);
                    ret.push_front(Subtree::Node(needle.clone()));
                }
            }
        }
        Subtree::Children(ret)
    }

    pub fn build_tree(&self, states: Vec<StateSet>) {
        let root = states.last().unwrap().iter()
            .filter(|item| item.complete() && item.start == 0 &&
                    item.rule.name == self.g.start).next().unwrap(); // assuming 1 parse
        println!("Start: {:?}", root);
        let tree = self.helper(&states, root, states.len() - 1);
        println!("{:?}", tree);
    }
    */
}
