use earley::symbol::Symbol;
use earley::items::{Item, StateSet, Trigger};
use earley::grammar::Grammar;
use earley::parser::ParseState;

use std::collections::VecDeque;
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
                     .next().unwrap(); // there should only be 1 top level match
    bt_helper(pstate, &root)
}



// (1, 4) > tree1: None, tree2: Subtree{b, []}  ==>  Subtree{b, []}

fn bt_helper(pstate: &ParseState, root: &Item) -> Option<Subtree> {

    // only bp2 can be complete, bp1 is the item being advanced

    //if let Some(&(ref bp1, ref bp2)) = root.bp.iter().next() {
    if let Some(&(ref bp1, ref bp2)) = root.bp.iter().next() {

        let tree1 = bt_helper(pstate, bp1);

        let tree2 = match bp2 {
            &Trigger::Completion(ref bp2) => bt_helper(pstate, bp2),
            &Trigger::Scan(ref input) => None,
        };

        // if complete -> print rule.spec
        if root.complete() {
            let mut child = VecDeque::new();
            if let Some(t) = tree1 {
                child.push_back(t);
            }
            if let Some(t) = tree2 {
                child.push_back(t);
            }
            Some(Subtree{value: root.rule.spec(), children: child})
        } else {
            if tree2.is_none() {
                tree1
            } else {
                tree2
            }
            //Some(Subtree{value: root.rule.spec(), children: children})
            ////tree1
            ////tree2
        }

    } else {

        // if complete -> print rule.spec
        if root.complete() {
            Some(Subtree{value: root.rule.spec(), children: VecDeque::new()})
        } else {
            None
        }

    }

}
