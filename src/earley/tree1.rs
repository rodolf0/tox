use earley::symbol::Symbol;
use earley::items::{Item, StateSet};
use earley::grammar::Grammar;

use std::collections::VecDeque;
use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Debug)]
pub struct Subtree {
    pub value: Rc<Symbol>,
    pub children: VecDeque<Subtree>,
}


// * Only completed items are put on the final list
// * Sort rules acording to order of apearance in grammar (resolve ambiguities)
fn purge_items(grammar: &Grammar, states: Vec<StateSet>) -> Vec<Item> {
    let mut items = Vec::new();
    for stateset in states.iter() {
        items.extend(stateset.iter().cloned().filter(|item| item.complete()));
    }
    // sort by start-point, then rule appearance in grammar, then longest
    items.sort_by(|a, b| {
        match a.start.cmp(&b.start) {
            Ordering::Equal => {
                // sort according to appearance in grammar
                let ax = grammar.rules.iter().position(|r| *r == a.rule);
                let bx = grammar.rules.iter().position(|r| *r == b.rule);
                match ax.unwrap().cmp(&bx.unwrap()) {
                    // sort by longest match first
                    //Ordering::Equal => (b.end - b.start).cmp(&(a.end - a.start)),
                    Ordering::Equal => (a.end - a.start).cmp(&(b.end - b.start)),
                    other => other,
                }
            },
            other => other,
        }
    });
    items
}


pub fn build_tree(grammar: &Grammar, states: Vec<StateSet>) -> Subtree {
    let last = states.len() - 1;
    let table = purge_items(&grammar, states);
    let root = table.iter().filter(|it| it.end == last && it.start == 0)
                           .next().unwrap(); // assume 1 parse
    println!("Start: {:?}", root);
    let tree = bt_helper(&table, root, last);
    tree
}

fn bt_helper(table: &Vec<Item>, theroot: &Item, mut end: usize) -> Subtree {
    let mut subtree = Subtree{value: theroot.rule.name.clone(),
                              children: VecDeque::new()};
    //for (depth, needle) in theroot.rule.spec.iter().enumerate().rev() {
    for needle in theroot.rule.spec.iter().rev() {
        match &**needle {
            &Symbol::NonTerm(_) => {
                // look for items completed at 'end' state with rule named 'needle'
                let item = table.iter().filter(|it| it.end == end &&
                                                    it.rule.name == *needle && // match rule name
                                                    **it != *theroot && // avoid self-recursion
                                                    it.start >= theroot.start)
                                                    //(depth != 0 || it.start == theroot.start))
                                       .next().unwrap();

                println!("Searching for {:?} completed at {}: {:?}", needle, end, item);

                let subsubtree = bt_helper(table, item, end);
                subtree.children.push_front(subsubtree); // cause rev-iter

                end = item.start;
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
