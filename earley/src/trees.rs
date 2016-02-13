use types::{Item, Trigger};
use parser::EarleyState;

#[derive(Debug, Clone)]
pub enum Subtree {
    Node(String, String),       // ("[+-]", "+")
    SubT(String, Vec<Subtree>), // ("E + E", [("n", "5"), ("[+-]", "+"), ("E * E", [...])])
}

// for non-ambiguous grammars this retreieve the only possible parse

pub fn build_trees(startsym: &str, pstate: &EarleyState) -> Vec<Subtree> {
    // TODO: missing start -> rule (top-most derivation missing) (root node)
    pstate.states.last().unwrap()
                 .filter_by_rule(startsym)
                 .filter(|it| it.start() == 0 && it.complete())
                 .flat_map(|root| bt_helper(pstate, root).into_iter())
                 .collect()
}

// source is always a prediction, can't be anything else cause it's on the left side
// trigger is either a scan or a completion, only those can advance a prediction

// TODO: return iterator so we don't bust memory
fn bt_helper(pstate: &EarleyState, root: &Item) -> Vec<Subtree> {
    let mut trees = Vec::new();
    for &(ref bp_prediction, ref bp_trigger) in root.back_pointers().iter() {
        // source/left-side is always a prediction (completions/scans are right side of bp)
        // flat-accumulate all left-side back-pointers

        let mut predictions = Vec::new();
        for left_tree in bt_helper(pstate, bp_prediction) {
            predictions.push(match left_tree {
                n @ Subtree::Node(_, _) => vec!(n),
                Subtree::SubT(_, childs) => childs,
            });
        }

        match bp_trigger {
            // Eg: E -> E + E .  // prediction is E +, trigger E
            &Trigger::Completion(ref bp_trigger) => {
                // can predictions/left-sides (which are never complete) have more than one origin ?
                for predtree in bt_helper(pstate, bp_prediction) {
                    let prediction = match predtree {
                        n @ Subtree::Node(_, _) => vec!(n),
                        Subtree::SubT(_, childs) => childs,
                    };
                    for trigger in bt_helper(pstate, bp_trigger) {
                        let mut p = prediction.clone();
                        p.push(trigger.clone());
                        trees.push(Subtree::SubT(root.rule_spec(), p));
                    }
                }
            },
            // Eg: E -> E + . E  // prediction is E, trigger +
            &Trigger::Scan(ref input) => {
                let label = bp_prediction.next_symbol().unwrap().name().to_string();
                for predtree in bt_helper(pstate, bp_prediction) {
                    let mut prediction = match predtree {
                        n @ Subtree::Node(_, _) => vec!(n),
                        Subtree::SubT(_, childs) => childs,
                    };
                    prediction.push(Subtree::Node(label.clone(), input.to_string()));
                    trees.push(Subtree::SubT(root.rule_spec(), prediction));
                }
            }
        };
    }
    if root.back_pointers().len() == 0 {
        trees.push(Subtree::SubT(String::new(), Vec::new()));
    }
    trees
}
