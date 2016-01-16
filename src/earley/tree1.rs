use earley::items::{Item, Trigger};
use earley::grammar::Grammar;
use earley::parser::ParseState;

#[derive(Debug)]
pub enum Subtree {
    Node(String, String),       // ("[+-]", "+")
    SubT(String, Vec<Subtree>), // ("E + E", [("n", "5"), ("[+-]", "+"), ("E * E", [...])])
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

// source is always a prediction, can't be anything else cause it's on the left side
// trigger is either a scan or a completion, only those can advance a prediction

fn bt_helper(pstate: &ParseState, root: &Item) -> Option<Subtree> {
    if let Some(&(ref bp_prediction, ref bp_trigger)) = root.bp.iter().next() {
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
                Some(Subtree::SubT(root.rule.spec(), prediction))
            },
            // Eg: E -> E + . E  // prediction is E, trigger +
            &Trigger::Scan(ref input) => {
                let label = bp_prediction.next_symbol().unwrap()
                                         .name().to_string();
                prediction.push(Subtree::Node(label, input.to_string()));
                Some(Subtree::SubT(root.rule.spec(), prediction))
            }
        }
    } else {
        None
    }
}
