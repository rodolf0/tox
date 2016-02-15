use types::{Item, Trigger};
use parser::EarleyState;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Subtree {
    Node(String, String),       // ("[+-]", "+")
    SubT(String, Vec<Subtree>), // ("E + E", [("n", "5"), ("[+-]", "+"), ("E * E", [...])])
}

// for non-ambiguous grammars this retreieve the only possible parse
pub fn one_tree(startsym: &str, pstate: &EarleyState) -> Option<Subtree> {
    match pstate.states.last().unwrap()
                 .filter_by_rule(startsym)
                 .filter(|it| it.start() == 0 && it.complete())
                 .filter_map(|root| one_helper(pstate, root))
                 .last() {
        Some(subt) => Some(Subtree::SubT(startsym.to_string(), vec![subt])),
        _ => None
    }
}

// source is always a prediction, can't be anything else cause it's on the left side,
// trigger is either a scan or a completion, only those can advance a prediction,
// to write this helper just draw a tree of the backpointers and see how they link
fn one_helper(pstate: &EarleyState, root: &Rc<Item>) -> Option<Subtree> {
    match root.back_pointers().iter().last() {
        Some(&(ref bp_prediction, ref bp_trigger)) => {
            // source/left-side is always a prediction (completions/scans are right side of bp)
            // flat-accumulate all left-side back-pointers
            let mut prediction = match one_helper(pstate, bp_prediction) {
                Some(n @ Subtree::Node(_, _)) => vec![n],
                Some(Subtree::SubT(_, childs)) => childs,
                None =>  Vec::new()
            };
            match bp_trigger {
                // Eg: E -> E + E .  // prediction is E +, trigger E
                &Trigger::Completion(ref bp_trigger) => {
                    let trigger = one_helper(pstate, bp_trigger);
                    if let Some(trigger) = trigger { prediction.push(trigger); }
                },
                // Eg: E -> E + . E  // prediction is E, trigger +
                &Trigger::Scan(ref input) => {
                    let label = bp_prediction.next_symbol().unwrap().name().to_string();
                    prediction.push(Subtree::Node(label, input.to_string()));
                }
            }
            Some(Subtree::SubT(root.str_rule(), prediction))
        },
        _ => None
    }
}


pub fn all_trees(startsym: &str, pstate: &EarleyState) -> Vec<Subtree> {
    pstate.states.last().unwrap()
                 .filter_by_rule(startsym)
                 .filter(|it| it.start() == 0 && it.complete())
                 .flat_map(|root| all_helper(pstate, root).into_iter())
                 .map(|subt| Subtree::SubT(startsym.to_string(), vec![subt]))
                 .collect()
}

// TODO: return iterator so we don't bust memory
fn all_helper(pstate: &EarleyState, root: &Rc<Item>) -> Vec<Subtree> {
    let mut trees = Vec::new();
    for &(ref bp_prediction, ref bp_trigger) in root.back_pointers().iter() {
        // source/left-side is always a prediction (completions/scans are right side of bp)
        // flat-accumulate all left-side back-pointers

        let mut predictions = Vec::new();
        for left_tree in all_helper(pstate, bp_prediction) {
            predictions.push(match left_tree {
                n @ Subtree::Node(_, _) => vec![n],
                Subtree::SubT(_, childs) => childs,
            });
        }

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
