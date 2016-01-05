use earley::symbol::Symbol;
use earley::items::{Item, StateSet};
use earley::grammar::Grammar;

use std::collections::VecDeque;
use std::cmp::Ordering;
//use std::ops;
use std::rc::Rc;

#[derive(Debug)]
pub struct Subtree {
    pub value: Rc<Symbol>,
    //pub children: Vec<Subtree>,
    pub children: VecDeque<Subtree>,
}

/*
// TODO: get rid of first elem, already in Item
struct RevTable(Vec<(usize, Item, usize)>);

impl RevTable {
    // Return a list of (start, rule, end)
    // * Flip the earley items so we can search forward
    // * Only completed items are put on the final list
    // * Sort rules acording to order of apearance in grammar (resolve ambiguities)
    fn new(grammar: &Grammar, states: Vec<StateSet>) -> RevTable {
        let mut items = Vec::new();
        for (idx, stateset) in states.iter().enumerate() {
            items.extend(stateset.iter().filter(|item| item.complete())
                                 .map(|item| (item.start, item.clone(), idx)));
        }
        // sort by start-point, then rule appearance in grammar, then longest
        items.sort_by(|a, b| {
            match a.0.cmp(&b.0) {
                Ordering::Equal => {
                    // sort according to appearance in grammar
                    let ax = grammar.rules.iter().position(|r| *r == a.1.rule);
                    let bx = grammar.rules.iter().position(|r| *r == b.1.rule);
                    match ax.unwrap().cmp(&bx.unwrap()) {
                        // sort by longest match first
                        Ordering::Equal => b.2.cmp(&a.2),
                        other => other,
                    }
                },
                other => other,
            }
        });
        RevTable(items)
    }
}
*/

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

//impl ops::Deref for RevTable {
    //type Target = Vec<(usize, Item, usize)>;
    //fn deref<'a>(&'a self) -> &'a Self::Target { &self.0 }
//}

//impl ops::DerefMut for RevTable {
    //fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target { &mut self.0 }
//}


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
    for (depth, needle) in theroot.rule.spec.iter().enumerate().rev() {
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

                //let subsubtree = bt_helper(table, item, item.start);
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

/*
pub fn build_tree(grammar: &Grammar, states: Vec<StateSet>) -> Subtree {
    let last = states.len() - 1;
    let revtable = RevTable::new(grammar, states);
    for i in revtable.iter() {
        println!("{:?}", i);
    }
    let root = revtable.iter().filter(|it|
                    it.0 == 0 && // rule starts at 0
                    it.2 == last && // rule covers all input
                    it.1.rule.name() == grammar.start.name()) // named like start
                .next().unwrap(); // just grab one parse
    println!("Picked {:?}", root);
    let tree = bt_helper(&revtable, &root.1, 0);
    tree
}

fn bt_helper(revtable: &RevTable, root: &Item, mut start: usize) -> Subtree {
    let mut subtree = Subtree{value: root.rule.name.clone(), children: Vec::new()};
    for needle in root.rule.spec.iter() {
        match &**needle {
            &Symbol::NonTerm(_) => {
                // we're picking the first item sorted per grammar order
                let item = revtable.iter()
                    .filter(|entry| entry.1 != *root && // WRONG, need whole entry avoid infinite left recursion
                                    entry.0 == start &&
                                    entry.1.rule.name() == needle.name())
                    .next().unwrap();
                println!("Picked {:?}", item);
                let subsubtree = bt_helper(revtable, &item.1, start);
                subtree.children.push(subsubtree);
                start = item.2;
            },
            &Symbol::Terminal(_, _) => {
                // TODO: put the actual token here
                subtree.children.push(
                    Subtree{value: needle.clone(), children: Vec::new()});
                start += 1;
            }
        }
    }
    subtree
}
*/
