use types::{Item, Trigger, Grammar};
use parser::ParseTrees;
use std::collections::HashMap;
use std::rc::Rc;


pub fn one_tree(ptrees: &ParseTrees) -> Subtree {
    ptrees.0.first().map(|root| one_helper(root)).unwrap()
}

// source is always a prediction, can't be anything else cause it's on the left side,
// trigger is either a scan or a completion, only those can advance a prediction,
// to write this helper just draw a tree of the backpointers and see how they link
fn one_helper(root: &Rc<Item>) -> Subtree {
    let mut childs = Vec::new();
    if let Some(&(ref bp_pred, ref bp_trig)) = root.source().iter().next() {
        // source/left-side is always a prediction (completions/scans are right side of bp)
        // flat-accumulate all left-side back-pointers that lead to the trigger
        match one_helper(bp_pred) {
            n @ Subtree::Leaf(_, _) => childs.push(n),
            Subtree::Node(_, c) => childs.extend(c),
        };
        match bp_trig {
            // Eg: E -> E + E .  // prediction is E +, trigger E
            &Trigger::Completion(ref bp_trig) =>
                childs.push(one_helper(bp_trig)),
            // Eg: E -> E + . E  // prediction is E, trigger +
            &Trigger::Scan(ref input) => {
                let label = bp_pred.next_symbol().unwrap().name();
                childs.push(Subtree::Leaf(label, input.clone()));
            }
        }
    }
    Subtree::Node(root.str_rule(), childs)
}


pub fn all_trees(ptrees: &ParseTrees) -> Vec<Subtree> {
    ptrees.0.iter().flat_map(|root| all_helper(root).into_iter()).collect()
}

// Enhance: return iterators to avoid busting mem
fn all_helper(root: &Rc<Item>) -> Vec<Subtree> {
    let source = root.source();
    let mut trees = Vec::new();
    if source.len() == 0 {
        trees.push(Subtree::Node(root.str_rule(), Vec::new()));
    } else {
        for &(ref bp_pred, ref bp_trig) in source.iter() {
            for predtree in all_helper(bp_pred) {
                let mut prediction = match predtree {
                    n @ Subtree::Leaf(_, _) => vec![n],
                    Subtree::Node(_, c) => c,
                };
                match bp_trig {
                    // Eg: E -> E + E .  // prediction is E +, trigger E
                    &Trigger::Completion(ref bp_trig) =>
                        for trigger in all_helper(bp_trig) {
                            let mut p = prediction.clone();
                            p.push(trigger.clone());
                            trees.push(Subtree::Node(root.str_rule(), p));
                        },
                    // Eg: E -> E + . E  // prediction is E, trigger +
                    &Trigger::Scan(ref input) => {
                        let label = bp_pred.next_symbol().unwrap().name();
                        prediction.push(Subtree::Leaf(label.clone(), input.clone()));
                        trees.push(Subtree::Node(root.str_rule(), prediction));
                    }
                }
            }
        }
    }
    trees
}

pub struct EarleyEvaler<ASTNode: Clone> {
    actions: HashMap<String, Box<Fn(&Vec<ASTNode>) -> ASTNode>>,
    tokenizer: Box<Fn(&str, &str)->ASTNode>,
}

impl<ASTNode: Clone> EarleyEvaler<ASTNode> {
    pub fn new<F>(tokenizer: F) -> EarleyEvaler<ASTNode>
            where F: 'static + Fn(&str, &str) -> ASTNode {
        EarleyEvaler{
            actions: HashMap::new(),
            tokenizer: Box::new(tokenizer),
        }
    }

    pub fn action<F>(&mut self, rule: &str, action: F)
            where F: 'static + Fn(&Vec<ASTNode>) -> ASTNode {
        self.actions.insert(rule.to_string(), Box::new(action));
    }

    fn walker(&self, root: &Rc<Item>) -> Vec<ASTNode> {
        // 1. collect arguments for semantic actions
        let mut args = Vec::new();
        let bp = root.source();
        if let Some(&(ref prediction, ref trigger)) = bp.iter().next() {
            // explore left side of the root
            args.extend(self.walker(prediction));
            // explore right side of the root
            match trigger {
                &Trigger::Completion(ref itm) => args.extend(self.walker(itm)),
                &Trigger::Scan(ref token) => {
                    let symbol = prediction.next_symbol().unwrap().name();
                    args.push((self.tokenizer)(&symbol, token));
                }
            };
        }

        // 2.if rule is complete, execute semantic action, else keep collecting
        if root.complete() {
            let rulename = root.str_rule();
            return match self.actions.get(&rulename) {
                None => panic!("No action for rule: {}", rulename),
                Some(action) => vec!(action(&args)),
            };
        }
        args
    }

    fn walker_all(&self, root: &Rc<Item>) -> Vec<Vec<ASTNode>> {
        // reduce function to call on complete items
        let rulename = root.str_rule();
        let reduce = |semargs| {
            match root.complete() {
                false => semargs,
                true => match self.actions.get(&rulename) {
                    None => panic!("No action for rule: {}", rulename),
                    Some(action) => vec!(action(&semargs)),
                }
            }
        };
        // explore treespace
        let mut trees = Vec::new();
        let source = root.source();
        if source.len() == 0 {
            return vec!(reduce(vec!()));
        }
        for &(ref prediction, ref trigger) in source.iter() {
            // get left-side-tree of each source
            for mut args in self.walker_all(prediction) {
                match trigger {
                    &Trigger::Completion(ref itm) => {
                        // collect right-side-tree of each source
                        for trig in self.walker_all(itm) {
                            let mut args = args.clone();
                            args.extend(trig);
                            trees.push(reduce(args));
                        }
                    },
                    &Trigger::Scan(ref token) => {
                        let symbol = prediction.next_symbol().unwrap().name();
                        args.push((self.tokenizer)(&symbol, token));
                        trees.push(reduce(args));
                    }
                };
            }
        }
        trees
    }

    // for non-ambiguous grammars this retreieve the only possible parse
    pub fn eval(&self, ptrees: &ParseTrees) -> Vec<ASTNode> {
        self.walker(ptrees.0.first().unwrap())
    }

    pub fn eval_all(&self, ptrees: &ParseTrees) -> Vec<Vec<ASTNode>> {
        ptrees.0.iter()
            .flat_map(|root| self.walker_all(root).into_iter())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Subtree {
    // ("[+-]", "+")
    Leaf(String, String),
    // ("E -> E [+-] E", [("n", "5"), ("[+-]", "+"), ("E -> E * E", [...])])
    Node(String, Vec<Subtree>),
}

impl Subtree {
    pub fn print(&self) {
        self.print_helper("")
    }
    fn print_helper(&self, level: &str) {
        match self {
            &Subtree::Leaf(ref sym, ref lexeme) => {
                println!("{}`-- {:?} ==> {:?}", level, sym, lexeme);
            },
            &Subtree::Node(ref spec, ref subn) => {
                println!("{}`-- {:?}", level, spec);
                if let Some((last, rest)) = subn.split_last() {
                    let l = format!("{}  |", level);
                    for n in rest { n.print_helper(&l); }
                    let l = format!("{}   ", level);
                    last.print_helper(&l);
                }
            }
        }
    }
}

pub fn subtree_evaler(g: Grammar) -> EarleyEvaler<Subtree> {
    let mut evaler = EarleyEvaler::<Subtree>::new(
        |sym, tok| Subtree::Leaf(sym.to_string(), tok.to_string())
    );
    for rule in g.rules() {
        evaler.action(&rule.clone(), move |nodes|
                      Subtree::Node(rule.clone(), nodes.clone()));
    }
    evaler
}
