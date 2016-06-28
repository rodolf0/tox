use types::{Item, Trigger, StateSet};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Subtree {
    Node(String, String),       // ("[+-]", "+")
    SubT(String, Vec<Subtree>), // ("E + E", [("n", "5"), ("[+-]", "+"), ("E * E", [...])])
}

// for non-ambiguous grammars this retreieve the only possible parse
pub fn one_tree(startsym: &str, pstate: &Vec<StateSet>) -> Subtree {
    pstate.last().unwrap()
          .filter_by_rule(startsym)
          .filter(|it| it.start() == 0 && it.complete())
          .map(|root| one_helper(pstate, root))
          .next().unwrap()
}

// source is always a prediction, can't be anything else cause it's on the left side,
// trigger is either a scan or a completion, only those can advance a prediction,
// to write this helper just draw a tree of the backpointers and see how they link
fn one_helper(pstate: &Vec<StateSet>, root: &Rc<Item>) -> Subtree {
    let mut childs = Vec::new();
    if let Some(&(ref bp_pred, ref bp_trig)) = root.back_pointers().iter().next() {
        // source/left-side is always a prediction (completions/scans are right side of bp)
        // flat-accumulate all left-side back-pointers that lead to the trigger
        match one_helper(pstate, bp_pred) {
            n @ Subtree::Node(_, _) => childs.push(n),
            Subtree::SubT(_, c) => childs.extend(c),
        };
        match bp_trig {
            // Eg: E -> E + E .  // prediction is E +, trigger E
            &Trigger::Completion(ref bp_trig) =>
                childs.push(one_helper(pstate, bp_trig)),
            // Eg: E -> E + . E  // prediction is E, trigger +
            &Trigger::Scan(ref input) => {
                let label = bp_pred.next_symbol().unwrap().name().to_string();
                childs.push(Subtree::Node(label, input.to_string()));
            }
        }
    }
    Subtree::SubT(root.str_rule(), childs)
}


pub fn all_trees(startsym: &str, pstate: &Vec<StateSet>) -> Vec<Subtree> {
    pstate.last().unwrap()
          .filter_by_rule(startsym)
          .filter(|it| it.start() == 0 && it.complete())
          .flat_map(|root| all_helper(pstate, root).into_iter())
          .collect()
}

// Enhance: return iterators to avoid busting mem
fn all_helper(pstate: &Vec<StateSet>, root: &Rc<Item>) -> Vec<Subtree> {
    let back_pointers = root.back_pointers();
    let mut trees = Vec::new();
    if back_pointers.len() == 0 {
        trees.push(Subtree::SubT(root.str_rule(), Vec::new()));
    } else {
        for &(ref bp_pred, ref bp_trig) in back_pointers.iter() {
            for predtree in all_helper(pstate, bp_pred) {
                let mut prediction = match predtree {
                    n @ Subtree::Node(_, _) => vec![n],
                    Subtree::SubT(_, c) => c,
                };
                match bp_trig {
                    // Eg: E -> E + E .  // prediction is E +, trigger E
                    &Trigger::Completion(ref bp_trig) =>
                        for trigger in all_helper(pstate, bp_trig) {
                            let mut p = prediction.clone();
                            p.push(trigger.clone());
                            trees.push(Subtree::SubT(root.str_rule(), p));
                        },
                    // Eg: E -> E + . E  // prediction is E, trigger +
                    &Trigger::Scan(ref input) => {
                        let label = bp_pred.next_symbol().unwrap().name().to_string();
                        prediction.push(Subtree::Node(label.clone(), input.to_string()));
                        trees.push(Subtree::SubT(root.str_rule(), prediction));
                    }
                }
            }
        }
    }
    trees
}
