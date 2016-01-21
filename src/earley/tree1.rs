use earley::types::{Item, Trigger};
use earley::grammar::Grammar;
use earley::parser::ParseState;

#[derive(Debug)]
pub enum Subtree {
    Node(String, String),       // ("[+-]", "+")
    SubT(String, Vec<Subtree>), // ("E + E", [("n", "5"), ("[+-]", "+"), ("E * E", [...])])
}

// for non-ambiguous grammars this retreieve the only possible parse

pub fn build_tree(grammar: &Grammar, pstate: &ParseState) -> Option<Subtree> {
    // get an item that spans the whole input and the rule matches the start
    let mut root = pstate.states.last().unwrap()
                    .filter_by_rule(grammar.start())
                    .filter(|it| it.start() == 0 && it.complete());
    if let Some(root) = root.next() {
        return bt_helper(pstate, root);
    }
    None
}

// source is always a prediction, can't be anything else cause it's on the left side
// trigger is either a scan or a completion, only those can advance a prediction

fn bt_helper(pstate: &ParseState, root: &Item) -> Option<Subtree> {
    if let Some(&(ref bp_prediction, ref bp_trigger)) = root.back_pointers().next() {
        // source/left-side is always a prediction (completions/scans are right side of bp)
        // flat-accumulate all left-side back-pointers
        let mut prediction = match bt_helper(pstate, bp_prediction) {
            Some(n @ Subtree::Node(_, _)) => vec!(n),
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
