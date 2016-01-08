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


// * Only completed items are put on the final list
// * Sort rules acording to order of apearance in grammar (resolve ambiguities)
//fn purge_items(grammar: &Grammar, states: Vec<StateSet>) -> Vec<Item> {
    //let mut items = Vec::new();
    //for stateset in states.iter() {
        //items.extend(stateset.iter().cloned().filter(|item| item.complete()));
    //}
    //// sort by start-point, then rule appearance in grammar, then longest
    //items.sort_by(|a, b| {
        //match a.start.cmp(&b.start) {
            //Ordering::Equal => {
                //// sort according to appearance in grammar
                //let ax = grammar.rules.iter().position(|r| *r == a.rule);
                //let bx = grammar.rules.iter().position(|r| *r == b.rule);
                //match ax.unwrap().cmp(&bx.unwrap()) {
                    //// sort by longest match first
                    ////Ordering::Equal => (b.end - b.start).cmp(&(a.end - a.start)),
                    //Ordering::Equal => (a.end - a.start).cmp(&(b.end - b.start)),
                    //other => other,
                //}
            //},
            //other => other,
        //}
    //});
    //items
//}


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



/*
fn bt_helper(table: &Vec<Item>, theroot: &Item, mut depth: usize) -> Subtree {
    let mut subtree = Subtree{value: theroot.rule.name.clone(),
                              children: VecDeque::new()};
    for (depth, needle) in theroot.rule.spec.iter().enumerate().rev() {
    //for needle in theroot.rule.spec.iter().rev() {
        match &**needle {
            &Symbol::NonTerm(_) => {
                let ond = end;
                // look for items completed at 'end' state with rule named 'needle'
                let mut items = table.iter().filter(|it| it.end == ond &&  // items completed at 'end'
                                                    it.rule.name == *needle && // match rule name
                                                    **it != *theroot && // avoid self-recursion
                                                    it.start >= theroot.start && // don't spill search outside parent's boundaries

                                                    (depth != 0 || it.start == theroot.start)) // is this necesary ? anchor to first item?
                    ;
                                       //.next().unwrap();

                let item = items.next();
                if item.is_some() {
                    println!("Searching for {:?} completed at {}: {:?}", needle, end, item);
                    let subsubtree = bt_helper(table, item.unwrap(), end);
                    subtree.children.push_front(subsubtree); // cause rev-iter
                    end = item.unwrap().start;
                }

                //println!("{}: {:?}", end, completed);
            },
            &Symbol::Terminal(ref t, _) => {
                subtree.children.push_front(
                    Subtree{value: needle.clone(), children: VecDeque::new()});
                end -= 1;
                println!("hit {:?} at {}", t, end);
            }
        }
    }
    subtree
}
*/
