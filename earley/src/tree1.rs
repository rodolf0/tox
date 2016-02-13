use types::{Item, Trigger};
use parser::EarleyState;
use std::rc::Rc;

#[derive(Debug)]
pub enum Subtree {
    Node(String, String),       // ("[+-]", "+")
    SubT(String, Vec<Subtree>), // ("E + E", [("n", "5"), ("[+-]", "+"), ("E * E", [...])])
}

// for non-ambiguous grammars this retreieve the only possible parse
pub fn build_tree(startsym: &str, pstate: &EarleyState) -> Option<Subtree> {
    // get an item that spans the whole input and the rule matches the start
    // TODO missing root node
    pstate.states.last().unwrap()
                 .filter_by_rule(startsym)
                 .filter(|it| it.start() == 0 && it.complete())
                 .filter_map(|root| bt_helper(pstate, root))
                 .last()
}

// source is always a prediction, can't be anything else cause it's on the left side,
// trigger is either a scan or a completion, only those can advance a prediction,
// to write this helper just draw a tree of the backpointers and see how they link
fn bt_helper(pstate: &EarleyState, root: &Rc<Item>) -> Option<Subtree> {
    if let Some(&(ref bp_prediction, ref bp_trigger)) = root.back_pointers().iter().last() {
        // source/left-side is always a prediction (completions/scans are right side of bp)
        // flat-accumulate all left-side back-pointers
        let mut prediction = match bt_helper(pstate, bp_prediction) {
            Some(n @ Subtree::Node(_, _)) => vec![n],
            Some(Subtree::SubT(_, childs)) => childs,
            None =>  Vec::new()
        };
        match bp_trigger {
            // Eg: E -> E + E .  // prediction is E +, trigger E
            &Trigger::Completion(ref bp_trigger) => {
                let trigger = bt_helper(pstate, bp_trigger);
                if let Some(trigger) = trigger { prediction.push(trigger); }
            },
            // Eg: E -> E + . E  // prediction is E, trigger +
            &Trigger::Scan(ref input) => {
                let label = bp_prediction.next_symbol().unwrap().name().to_string();
                prediction.push(Subtree::Node(label, input.to_string()));
            }
        };
        Some(Subtree::SubT(root.rule_spec(), prediction))
    } else {
        None
    }
}
