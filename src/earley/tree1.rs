use earley::symbol::Symbol;
use earley::items::{Item, StateSet};
use earley::grammar::Grammar;
use earley::parser::ParseState;

use std::collections::VecDeque;
//use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Debug)]
pub struct Subtree {
    pub value: String,
    pub children: VecDeque<Subtree>,
}


pub fn build_tree(grammar: &Grammar, pstate: &ParseState) -> Option<Subtree> {
    // get an item that spans the whole input and the rule matches the start
    let root = pstate.states.last().unwrap().iter()
                     .filter(|it| it.start == 0 &&
                                  it.complete() &&
                                  it.rule.name == grammar.start)
                     .next().unwrap(); // only building 1 subtree
    println!("Start: {:?}", root);
    let tree = bt_helper(pstate, root, pstate.states.len() - 1);
    tree
}

fn bt_helper(pstate: &ParseState, root: &Item, mut end: usize) -> Option<Subtree> {
    let mut children = VecDeque::new();

    for sym in root.rule.spec.iter().rev() {
        if sym.is_term() {
            end -= 1; // TODO overflow where end is already 0
            let token = pstate.input[end].clone();
            if sym.term_match(&token) {
                println!("hit '{}' at {} for {:?} from {:?}", token, end, sym, root);
                children.push_front(Subtree{
                    value: token, children: VecDeque::new()
                });
            } else {
                println!("discarding {:?} token {} doesn't match {:?}", root, token, sym);
                return None;
            }
        } else {
            let possible_expansions = pstate.states[end].iter() // item must end from where we search backwards
                          .filter(|it| it.complete() &&         // discard non-complete items
                                       it.rule.name == *sym &&  // subtree rule name must match current sym
                                       **it != *root &&         // avoid self-recursion
                                       it.start >= root.start); // this subtree needs to be within the root

            for item in possible_expansions {
                println!("searching for {:?} completed at {} ====> {:?}", sym, end, item);
                if let Some(subsubtree) = bt_helper(pstate, item, end) {
                    children.push_front(subsubtree);
                    end = item.start;
                    println!("ok using {:?}", item);
                    break;
                    // TODO: what about all options !? need to return them all
                } else {
                    println!("no good {:?}", item);
                }
            }
        }
    }
    if end == root.start {
        return Some(Subtree{value: root.rule.name().to_string(), children: children});
    }
    return None;
}
