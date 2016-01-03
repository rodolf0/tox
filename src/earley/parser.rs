use earley::symbol::Symbol;
use earley::items::{Item, StateSet, Rule};
use earley::grammar::Grammar;
use earley::Lexer;

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
            if last.iter().filter(|item|
                    item.start == 0 && item.completes(self.g.start.name())
                ).count() < 1 {
                return Err(ParseError::BadInput);
            }
        }
        Ok(states)
    }
}

use std::collections::VecDeque;
use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Debug)]
pub struct Subtree {
    pub value: Rc<Symbol>,
    pub children: VecDeque<Subtree>,
}

impl EarleyParser {
    //pub fn build_tree(&self, states: Vec<StateSet>) -> Subtree {
        //// get a complete item from the last stateset
        //let root = states.last().unwrap().iter()
            //.filter(|item| item.completes(self.g.start.name()) && item.start == 0)
            //.next().unwrap(); // assuming 1 parse
        //println!("Start: {:?}", root);
        //let tree = self.bt_helper(&states, root, states.len() - 1);
        //println!("{:?}", &tree);
        //tree
    //}

    pub fn build_tree(&self, states: Vec<StateSet>) -> Subtree {
        //let last = states.len() - 1;
        //let revtable = purge_items(states);
        //let root = revtable.iter().filter(|it|
                    //it.0 == 0 && // rule starts at 0
                    //it.2 == last && // rule covers all input
                    //it.1.name() == self.g.start.name()); // named like start

        // get a complete item from the last stateset
        let root = states.last().unwrap().iter()
            .filter(|item| item.completes(self.g.start.name()) && item.start == 0)
            .next().unwrap(); // assuming 1 parse
        println!("Start: {:?}", root);
        let tree = self.bt_helper(&states, root, states.len() - 1);
        println!("{:?}", &tree);
        tree
    }

    fn bt_helper(&self, states: &Vec<StateSet>, theroot: &Item, mut end: usize) -> Subtree {
        let mut subtree = Subtree{value: theroot.rule.name.clone(),
                                  children: VecDeque::new()};
        for needle in theroot.rule.spec.iter().rev() {
            match &**needle {
                &Symbol::NonTerm(_) => {
                    println!("Searching for {:?} completed at {}", needle, end);
                    // look for items completed at 'end' state with rule named 'needle'
                    let mut items = states[end].iter()
                        .filter(|item| item.completes(needle.name()));
                    // we're _randomly_ picking the first item
                    // if the grammar is non-ambig then it's the only option
                    //
                    // should pick the top priority one, sort items per rule precedence
                    let item = items.next().unwrap();

                    let subsubtree = self.bt_helper(states, item, end);
                    subtree.children.push_front(subsubtree); // cause rev-iter

                    end = item.start;
                    //println!("{}: {:?}", end, completed);
                },
                &Symbol::Terminal(ref t, _) => {
                    println!("hit {:?} at {}", t, end);
                    subtree.children.push_front(
                        Subtree{value: needle.clone(), children: VecDeque::new()});
                    end -= 1; // we'll search...
                }
            }
        }
        subtree
    }

    // Return a list of (start, rule, end)
    // * Flip the earley items so we can search forward
    // * Only completed items are put on the final list
    // * Sort rules acording to order of apearance in grammar (resolve ambiguities)
    fn purge_items(&self, states: Vec<StateSet>) -> Vec<(usize, Rc<Rule>, usize)> {
        let mut items = Vec::new();
        for (idx, stateset) in states.iter().enumerate() {
            items.extend(stateset.iter().filter(|item| item.complete())
                                 .map(|item| (item.start, item.rule.clone(), idx)));
        }
        // sort by start-point, then rule appearance in grammar, then longest
        items.sort_by(|a, b| {
            match a.0.cmp(&b.0) {
                Ordering::Equal => {
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
        items
    }
}
