#![deny(warnings)]

use items::{Item, Trigger};
use parser::ParseTrees;
use std::collections::HashMap;
use std::rc::Rc;


#[derive(Clone,Debug)]
pub enum EvalError {
    MissingAction(String),
}

pub struct EarleyForest<'a, ASTNode: Clone> {
    // semantic actions to execute when walking the tree
    actions: HashMap<String, Box<Fn(Vec<ASTNode>)->ASTNode + 'a>>,
    // leaf_builder creates ASTNodes given a rule-string + a token
    leaf_builder: Box<Fn(&str, &str)->ASTNode + 'a>,
    debug: bool,
}

///////////////////////////////////////////////////////////////////////////////

impl<'a, ASTNode: Clone> EarleyForest<'a, ASTNode> {
    pub fn new<F>(leaf_builder: F) -> Self
            where F: 'a + Fn(&str, &str) -> ASTNode {
        EarleyForest{
            actions: HashMap::new(),
            leaf_builder: Box::new(leaf_builder),
            debug: false,
        }
    }

    pub fn debug<F>(leaf_builder: F) -> Self
            where F: 'a + Fn(&str, &str) -> ASTNode {
        EarleyForest{
            actions: HashMap::new(),
            leaf_builder: Box::new(leaf_builder),
            debug: true,
        }
    }

    // Register semantic actions to act when rules are matched
    pub fn action<F>(&mut self, rule: &str, action: F)
            where F: 'a + Fn(Vec<ASTNode>) -> ASTNode {
        self.actions.insert(rule.to_string(), Box::new(action));
    }

    // Source is always a prediction, can't be anything else cause it's on the
    // left side. Trigger is either a scan or a completion, only those can
    // advance a prediction. To write this helper just draw a tree of the
    // backpointers and see how they link
    fn walker(&self, root: &Rc<Item>) -> Result<Vec<ASTNode>, EvalError> {
        let mut args = Vec::new();
        // 1. collect arguments for semantic actions
        let bp = root.source();
        if let Some(&(ref prediction, ref trigger)) = bp.iter().next() {
            // explore left side of the root
            // TODO: remove try!
            args.extend(try!(self.walker(prediction)));
            // explore right side of the root
            args.extend(match *trigger {
                Trigger::Complete(ref item) => try!(self.walker(item)),
                Trigger::Scan(ref token) => {
                    let symbol = prediction.next_symbol().unwrap().name();
                    vec![(self.leaf_builder)(&symbol, token)]
                }
            });
        }

        // 2.if rule is complete, execute semantic action, else keep collecting
        if root.complete() {
            let rulename = root.rule.to_string();
            return match self.actions.get(&rulename) {
                None => Err(EvalError::MissingAction(rulename)),
                Some(action) => {
                    if self.debug { eprintln!("Reduction: {}", rulename); }
                    Ok(vec![action(args)])
                }
            };
        }
        Ok(args)
    }

    // for non-ambiguous grammars this retreieves the only possible parse
    pub fn eval(&self, ptrees: &ParseTrees) -> Result<ASTNode, EvalError> {
        // walker will always return a Vec of size 1 because root.complete
        Ok(self.walker(ptrees.0.first().unwrap())?.swap_remove(0))
    }

    fn walker_all(&self, root: &Rc<Item>)
            -> Vec<Result<Vec<ASTNode>, EvalError>> {
        // reduce function to call on complete items
        let rulename = root.rule.to_string();
        let reduce = |args: Vec<ASTNode>| -> Result<Vec<ASTNode>, EvalError> {
            if root.complete() {
                match self.actions.get(&rulename) {
                    None => Err(EvalError::MissingAction(rulename.clone())),
                    Some(action) => {
                        if self.debug { eprintln!("Reduction: {}", rulename); }
                        Ok(vec![action(args)])
                    }
                }
            } else {
                Ok(args)
            }
        };
        // explore treespace
        let mut trees = Vec::new();
        let source = root.source();
        if source.len() == 0 {
            return vec![reduce(Vec::new())];
        }
        for &(ref prediction, ref trigger) in source.iter() {
            // get left-side-tree of each source
            for args in self.walker_all(prediction) {
                let mut args = match args {
                    Ok(args) => args, // unpack args
                    Err(e) => return vec![Err(e)]
                };
                match *trigger {
                    Trigger::Complete(ref itm) => {
                        // collect right-side-tree of each source
                        for trig in self.walker_all(itm) {
                            let trig = match trig {
                                Ok(trig) => trig,
                                Err(e) => return vec![Err(e)]
                            };
                            let mut args = args.clone();
                            args.extend(trig);
                            trees.push(reduce(args));
                        }
                    },
                    Trigger::Scan(ref token) => {
                        let symbol = prediction.next_symbol().unwrap().name();
                        args.push((self.leaf_builder)(&symbol, token));
                        trees.push(reduce(args));
                    }
                };
            }
        }
        trees
    }

    // Retrieves all parse trees
    pub fn eval_all(&self, ptrees: &ParseTrees)
            -> Result<Vec<ASTNode>, EvalError> {
        let maybe_trees = ptrees.0.iter()
            .flat_map(|root| self.walker_all(root).into_iter());
        let mut trees = Vec::new();
        for tree in maybe_trees {
            trees.push(try!(tree).swap_remove(0));
        }
        Ok(trees)
    }
}
