use earley::symbol::Symbol;
use earley::items::{Item, StateSet};
use earley::grammar::Grammar;

use std::cmp::Ordering;
use std::ops;
use std::rc::Rc;

#[derive(Debug)]
pub struct Subtree {
    pub value: Rc<Symbol>,
    pub children: Vec<Subtree>,
}

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

impl ops::Deref for RevTable {
    type Target = Vec<(usize, Item, usize)>;
    fn deref<'a>(&'a self) -> &'a Self::Target { &self.0 }
}

impl ops::DerefMut for RevTable {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Self::Target { &mut self.0 }
}


pub fn build_tree(grammar: &Grammar, states: Vec<StateSet>) -> Subtree {
    let last = states.len() - 1;
    let revtable = RevTable::new(grammar, states);
    let root = revtable.iter().filter(|it|
                    it.0 == 0 && // rule starts at 0
                    it.2 == last && // rule covers all input
                    it.1.rule.name() == grammar.start.name()) // named like start
                .next().unwrap(); // just grab one parse
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
                    .filter(|entry| entry.1 != *root && // avoid infinite left recursion
                                    entry.0 == start &&
                                    entry.1.rule.name() == needle.name())
                    .next().unwrap();
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
