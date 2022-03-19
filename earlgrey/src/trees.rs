#![deny(warnings)]

use crate::items::{Item, BackPointer};
use crate::parser::ParseTrees;
use std::collections::HashMap;
use std::rc::Rc;


// Semantic actions to execute when walking the tree
type SemAction<'a, ASTNode> = Box<dyn Fn(Vec<ASTNode>) -> ASTNode + 'a>;
// Given a Rule and a Token build an ASTNode
type LeafBuilder<'a, ASTNode> = Box<dyn Fn(&str, &str) -> ASTNode + 'a>;

pub struct EarleyForest<'a, ASTNode: Clone> {
    actions: HashMap<String, SemAction<'a, ASTNode>>,
    leaf_builder: LeafBuilder<'a, ASTNode>,
}

impl<'a, ASTNode: Clone> EarleyForest<'a, ASTNode> {
    pub fn new<Builder>(leaf_builder: Builder) -> Self
            where Builder: Fn(&str, &str) -> ASTNode + 'a {
        EarleyForest{
            actions: HashMap::new(),
            leaf_builder: Box::new(leaf_builder)}
    }

    // Register semantic actions to act when rules are matched
    pub fn action<Action>(&mut self, rule: &str, action: Action)
            where Action: Fn(Vec<ASTNode>) -> ASTNode + 'a {
        self.actions.insert(rule.to_string(), Box::new(action));
    }
}

impl<'a, ASTNode: Clone> EarleyForest<'a, ASTNode> {
    fn reduce(&self, root: &Rc<Item>, args: Vec<ASTNode>)
            -> Result<Vec<ASTNode>, String> {
        // if item is not complete, keep collecting args
        if !root.complete() { return Ok(args) }
        let rulename = root.rule.to_string();
        match self.actions.get(&rulename) {
            None => Err(format!("Missing Action: {}", rulename)),
            Some(action) => {
                if cfg!(feature="debug") {
                    eprintln!("Reduction: {}", rulename);
                }
                Ok(vec![action(args)])
            }
        }
    }
}

impl<'a, ASTNode: Clone> EarleyForest<'a, ASTNode> {

    // Source is always a prediction, can't be anything else cause it's on the
    // left side. Trigger is either a scan or a completion, only those can
    // advance a prediction. To write this helper just draw a tree of the
    // backpointers and see how they link
    fn walker(&self, root: &Rc<Item>) -> Result<Vec<ASTNode>, String> {
        let mut args = Vec::new();
        // collect arguments for semantic actions
        if let Some(backpointer) = root.sources().iter().next() {
            match backpointer {
                BackPointer::Complete(source, trigger) => {
                    args.extend(self.walker(source)?);
                    args.extend(self.walker(trigger)?);
                }
                BackPointer::Scan(source, trigger) => {
                    let symbol = source.next_symbol()
                        .expect("BUG: missing scan trigger symbol").name();
                    args.extend(self.walker(source)?);
                    args.push((self.leaf_builder)(symbol, trigger));
                }
            }
        }
        self.reduce(root, args)
    }

    // for non-ambiguous grammars this retreieves the only possible parse
    pub fn eval(&self, ptrees: &ParseTrees) -> Result<ASTNode, String> {
        // walker will always return a Vec of size 1 because root.complete
        Ok(self.walker(ptrees.0.first().expect("BUG: ParseTrees empty"))?
           .swap_remove(0))
    }
}


impl<'a, ASTNode: Clone> EarleyForest<'a, ASTNode> {

    fn walker_all(&self, root: &Rc<Item>) -> Result<Vec<Vec<ASTNode>>, String> {
        let source = root.sources();
        if source.len() == 0 {
            return Ok(vec![self.reduce(root, Vec::new())?]);
        }
        let mut trees = Vec::new();
        for backpointer in source.iter() {
            match backpointer {
                BackPointer::Complete(source, trigger) => {
                    // collect left-side-tree of each node
                    for args in self.walker_all(source)? {
                        // collect right-side-tree of each node
                        for trig in self.walker_all(trigger)? {
                            let mut args = args.clone();
                            args.extend(trig);
                            trees.push(self.reduce(root, args)?);
                        }
                    }
                }
                BackPointer::Scan(source, trigger) => {
                    for mut args in self.walker_all(source)? {
                        let symbol = source.next_symbol()
                            .expect("BUG: missing scan trigger symbol").name();
                        args.push((self.leaf_builder)(symbol, trigger));
                        trees.push(self.reduce(root, args)?);
                    }
                }
            }
        }
        Ok(trees)
    }

    // Retrieves all parse trees
    pub fn eval_all(&self, ptrees: &ParseTrees) -> Result<Vec<ASTNode>, String> {
        let mut trees = Vec::new();
        for root in &ptrees.0 {
            trees.extend(
                self.walker_all(root)?.into_iter()
                    .map(|mut treevec| treevec.swap_remove(0)));
        }
        Ok(trees)
    }

    // TODO: provide an estimate
    pub fn num_trees(&self) -> Option<u32> { None }
}
