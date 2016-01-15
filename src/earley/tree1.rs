use earley::symbol::Symbol;
use earley::items::{Item, StateSet, Trigger};
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

// if incomplete, lo que esta a la izquierda acumular en la lista y devolver None en el valor
// si esta completo, stringify spec en 'valor' y meter la lista

// source is always a prediction, can't be anything else cause it's on the left side
// trigger is either a scan or a completion, only those can advance a prediction

fn bt_helper(pstate: &ParseState, root: &Item) -> Option<Subtree> {
    if let Some(&(ref bp_source, ref bp_trigger)) = root.bp.iter().next() {

        let source = bt_helper(pstate, bp_source);

        match bp_trigger {
            &Trigger::Completion(ref bp_trigger) => {
                let trigger = bt_helper(pstate, bp_trigger);

                // acumulate source
                match source {
                    Some(n @ Subtree::Node(_, _)) => {
                        let mut nodes = Vec::new();
                        nodes.push(n);
                        if let Some(trigger) = trigger {
                            nodes.push(trigger);
                        }
                        Some(Subtree::SubT(root.rule.spec(), nodes))
                    },
                    Some(Subtree::SubT(_, childs)) => {
                        let mut nodes = Vec::new();
                        nodes.extend(childs);
                        if let Some(trigger) = trigger {
                            nodes.push(trigger);
                        }
                        Some(Subtree::SubT(root.rule.spec(), nodes))
                    },
                    // trigger in Completion can't be None ?
                    None => {
                        let mut nodes = Vec::new();
                        if let Some(trigger) = trigger {
                            nodes.push(trigger);
                        }
                        Some(Subtree::SubT(root.rule.spec(), nodes))
                    }
                }
            },
            &Trigger::Scan(ref input) => { // Eg: E -> E + . E  // source is E, trigger +
                // acumulate source
                match source {
                    Some(n @ Subtree::Node(_, _)) => {
                        let mut nodes = Vec::new();
                        nodes.push(n);
                        nodes.push(Subtree::Node(format!("!"), input.to_string()));
                        Some(Subtree::SubT(format!("@"), nodes))
                    },
                    Some(Subtree::SubT(_, childs)) => {
                        let mut nodes = Vec::new();
                        nodes.extend(childs);
                        nodes.push(Subtree::Node(format!("!"), input.to_string()));
                        Some(Subtree::SubT(format!("@"), nodes))
                    },
                    None => Some(Subtree::Node(format!("#"), input.to_string()))
                }
            }
        }

    } else {
        None
    }
}
