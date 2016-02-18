use types::{Item, Trigger};
use parser::EarleyState;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Subtree {
    Node(String, String),       // ("[+-]", "+")
    SubT(String, Vec<Subtree>), // ("E + E", [("n", "5"), ("[+-]", "+"), ("E * E", [...])])
}

// for non-ambiguous grammars this retreieve the only possible parse
pub fn one_tree(startsym: &str, pstate: &EarleyState) -> Subtree {
    pstate.states.last().unwrap()
                 .filter_by_rule(startsym)
                 .filter(|it| it.start() == 0 && it.complete())
                 .map(|root| one_helper(pstate, root))
                 .next().unwrap()
}

// source is always a prediction, can't be anything else cause it's on the left side,
// trigger is either a scan or a completion, only those can advance a prediction,
// to write this helper just draw a tree of the backpointers and see how they link
fn one_helper(pstate: &EarleyState, root: &Rc<Item>) -> Subtree {
    let mut tree = Vec::new();
    if let Some(&(ref bp_prediction, ref bp_trigger)) = root.back_pointers().iter().next() {
        // source/left-side is always a prediction (completions/scans are right side of bp)
        // flat-accumulate all left-side back-pointers that lead to the trigger
        let mut prediction = match one_helper(pstate, bp_prediction) {
            n @ Subtree::Node(_, _) => tree.push(n),
            Subtree::SubT(_, childs) => tree.extend(childs),
        };
        match bp_trigger {
            // Eg: E -> E + E .  // prediction is E +, trigger E
            &Trigger::Completion(ref bp_trigger) => tree.push(one_helper(pstate, bp_trigger)),
            // Eg: E -> E + . E  // prediction is E, trigger +
            &Trigger::Scan(ref input) => {
                let label = bp_prediction.next_symbol().unwrap().name().to_string();
                tree.push(Subtree::Node(label, input.to_string()));
            }
        }
    }
    Subtree::SubT(root.str_rule(), tree)
}


pub fn all_trees(startsym: &str, pstate: &EarleyState) -> Vec<Subtree> {
    pstate.states.last().unwrap()
                 .filter_by_rule(startsym)
                 .filter(|it| it.start() == 0 && it.complete())
                 .flat_map(|root| all_helper(pstate, root).into_iter())
                 .collect()
}

// TODO: return iterator so we don't bust memory
fn all_helper(pstate: &EarleyState, root: &Rc<Item>) -> Vec<Subtree> {
    let mut trees = Vec::new();
    for &(ref bp_prediction, ref bp_trigger) in root.back_pointers().iter() {
        // source/left-side is always a prediction (completions/scans are right side of bp)
        // flat-accumulate all left-side back-pointers

        match bp_trigger {
            // Eg: E -> E + E .  // prediction is E +, trigger E
            &Trigger::Completion(ref bp_trigger) => {
                for predtree in all_helper(pstate, bp_prediction) {
                    let prediction = match predtree {
                        n @ Subtree::Node(_, _) => vec![n],
                        Subtree::SubT(_, childs) => childs,
                    };
                    for trigger in all_helper(pstate, bp_trigger) {
                        let mut p = prediction.clone();
                        p.push(trigger.clone());
                        trees.push(Subtree::SubT(root.str_rule(), p));
                    }
                }
            },
            // Eg: E -> E + . E  // prediction is E, trigger +
            &Trigger::Scan(ref input) => {
                let label = bp_prediction.next_symbol().unwrap().name().to_string();
                for predtree in all_helper(pstate, bp_prediction) {
                    let mut prediction = match predtree {
                        n @ Subtree::Node(_, _) => vec![n],
                        Subtree::SubT(_, childs) => childs,
                    };
                    prediction.push(Subtree::Node(label.clone(), input.to_string()));
                    trees.push(Subtree::SubT(root.str_rule(), prediction));
                }
            }
        };
    }
    if root.back_pointers().len() == 0 {
        trees.push(Subtree::SubT(String::new(), Vec::new()));
    }
    trees
}
