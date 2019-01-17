#![deny(warnings)]

use crate::items::{Item, Trigger};
use crate::parser::{ParseTrees, Error};
use std::collections::HashMap;
use std::rc::Rc;


// Semantic actions to execute when walking the tree
type SemAction<'a, ASTNode> = Box<Fn(Vec<ASTNode>) -> ASTNode + 'a>;
// Given a Rule and a Token build an ASTNode
type LeafBuilder<'a, ASTNode> = Box<Fn(&str, &str) -> ASTNode + 'a>;

pub struct EarleyForest<'a, ASTNode: Clone> {
    actions: HashMap<String, SemAction<'a, ASTNode>>,
    leaf_builder: LeafBuilder<'a, ASTNode>,
    debug: bool,
}

impl<'a, ASTNode: Clone> EarleyForest<'a, ASTNode> {
    pub fn new<Builder>(leaf_builder: Builder) -> Self
            where Builder: Fn(&str, &str) -> ASTNode + 'a {
        EarleyForest{
            actions: HashMap::new(),
            leaf_builder: Box::new(leaf_builder),
            debug: false}
    }

    // Register semantic actions to act when rules are matched
    pub fn action<Action>(&mut self, rule: &str, action: Action)
            where Action: Fn(Vec<ASTNode>) -> ASTNode + 'a {
        self.actions.insert(rule.to_string(), Box::new(action));
    }
}

impl<'a, ASTNode: Clone> EarleyForest<'a, ASTNode> {
    fn reduce(&self, root: &Rc<Item>, args: Vec<ASTNode>)
            -> Result<Vec<ASTNode>, Error> {
        // if item is not complete, keep collecting args
        if !root.complete() { return Ok(args) }
        let rulename = root.rule.to_string();
        match self.actions.get(&rulename) {
            None => Err(Error::MissingAction(rulename)),
            Some(action) => {
                if self.debug { eprintln!("Reduction: {}", rulename); }
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
    fn walker(&self, root: &Rc<Item>) -> Result<Vec<ASTNode>, Error> {
        let mut args = Vec::new();
        // collect arguments for semantic actions
        let source = root.source();
        if let Some((ref prediction, ref trigger)) = source.iter().next() {
            // explore left side of the root
            args.extend(self.walker(prediction)?);
            // explore right side of the root
            args.extend(match *trigger {
                Trigger::Complete(ref item) => self.walker(item)?,
                Trigger::Scan(ref token) => {
                    let symbol = prediction.next_symbol()
                        .expect("BUG: missing scan trigger symbol").name();
                    vec![(self.leaf_builder)(&symbol, token)]
                }
            });
        }
        self.reduce(root, args)
    }

    // for non-ambiguous grammars this retreieves the only possible parse
    pub fn eval(&self, ptrees: &ParseTrees) -> Result<ASTNode, Error> {
        // walker will always return a Vec of size 1 because root.complete
        Ok(self.walker(ptrees.0.first().expect("BUG: ParseTrees empty"))?
           .swap_remove(0))
    }
}


impl<'a, ASTNode: Clone> EarleyForest<'a, ASTNode> {

    fn walker_all(&self, root: &Rc<Item>) -> Result<Vec<Vec<ASTNode>>, Error> {
        let source = root.source();
        if source.len() == 0 {
            return Ok(vec![self.reduce(root, Vec::new())?]);
        }
        let mut trees = Vec::new();
        for (ref prediction, ref trigger) in source.iter() {
            // get left-side-tree of each source
            for mut args in self.walker_all(prediction)? {
                match *trigger {
                    Trigger::Complete(ref itm) => {
                        // collect right-side-tree of each source
                        for trig in self.walker_all(itm)? {
                            let mut args = args.clone();
                            args.extend(trig);
                            trees.push(self.reduce(root, args)?);
                        }
                    },
                    Trigger::Scan(ref token) => {
                        let symbol = prediction.next_symbol()
                            .expect("BUG: missing scan trigger symbol").name();
                        args.push((self.leaf_builder)(&symbol, token));
                        trees.push(self.reduce(root, args)?);
                    }
                }
            }
        }
        Ok(trees)
    }

    // Retrieves all parse trees
    pub fn eval_all(&self, ptrees: &ParseTrees) -> Result<Vec<ASTNode>, Error> {
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
