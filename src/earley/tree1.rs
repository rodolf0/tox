use earley::symbol::Symbol;
use earley::items::{Item, StateSet, Trigger};
use earley::grammar::Grammar;
use earley::parser::ParseState;

#[derive(Debug)]
pub enum Tree {
    Node(String),
    SubT(Vec<Tree>), // SubT((String, Vec<Tree>))
}

pub fn build_tree(grammar: &Grammar, pstate: &ParseState) -> Tree {
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


fn bt_helper(pstate: &ParseState, root: &Item) -> Tree {

    let (tree_source, tree_trigger) = match root.bp.iter().next() {
        Some(&(ref bp_source, ref bp_trigger)) => {
            let tree_source = bt_helper(pstate, bp_source);
            let tree_trigger = match bp_trigger {
                //&Trigger::Completion(ref bp_trigger) => bt_helper(pstate, bp_trigger),
                &Trigger::Completion(ref bp_trigger) =>
                    Tree::SubT(vec![bt_helper(pstate, bp_trigger)]),
                &Trigger::Scan(ref input) => Tree::Node(input.to_string()),
            };
            (tree_source, tree_trigger)
        },
        _ => (Tree::SubT(Vec::new()), Tree::SubT(Vec::new())) // yuck use None
    };

    //if root.complete() {
        //Tree::SubT(vec![tree_source, tree_trigger])
    //} else {
        let mut childs = Vec::new();
        match tree_source {
            n @ Tree::Node(_) => childs.push(n),
            Tree::SubT(l) => childs.extend(l),
        }
        match tree_trigger {
            n @ Tree::Node(_) => childs.push(n),
            Tree::SubT(l) => childs.extend(l),
        }
        Tree::SubT(childs)
    //}
}

/*
// TODO: this is a (probably non-general) prototype
// should return a list of children ?? see
// http://loup-vaillant.fr/tutorials/earley-parsing/semantic-actions
fn bt_helper(pstate: &ParseState, root: &Item) -> Subtree {

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
*/
