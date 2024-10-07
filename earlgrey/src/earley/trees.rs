#![deny(warnings)]

use super::spans::{Span, SpanSource};
use super::parser::ParseTrees;
use std::collections::HashMap;
use std::rc::Rc;

pub struct EarleyForest<'a, ASTNode: Clone> {
    // Semantic actions to apply when a production is completed
    actions: HashMap<String, Box<dyn Fn(Vec<ASTNode>) -> ASTNode + 'a>>,
    // How to lift a 'scanned' terminal into an AST node.
    terminal_parser: Box<dyn Fn(&str, &str) -> ASTNode + 'a>,
}

impl<'a, ASTNode: Clone> EarleyForest<'a, ASTNode> {
    pub fn new(terminal_parser: impl Fn(&str, &str) -> ASTNode + 'a) -> Self {
        EarleyForest{
            actions: HashMap::new(),
            terminal_parser: Box::new(terminal_parser)}
    }

    // Register semantic actions to act when rules are matched
    pub fn action(&mut self, rule: &str, action: impl Fn(Vec<ASTNode>) -> ASTNode + 'a) {
        self.actions.insert(rule.to_string(), Box::new(action));
    }
}


impl<'a, ASTNode: Clone> EarleyForest<'a, ASTNode> {
    fn reduce(&self, root: &Rc<Span>, args: Vec<ASTNode>)
            -> Result<Vec<ASTNode>, String> {
        // If span is not complete, reduce is a noop passthrough
        if !root.complete() { return Ok(args) }
        // Lookup semantic action to apply based on rule name
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

    // To write this helper draw a tree of the backpointers and see how they link.
    // - If a span has no sources then its rule progress is at the start.
    // - If it originates from a 'completion' there's a span (the source)
    // that was extended because another (the trigger) completed its rule.
    // Recurse both spans transitively until they have no sources to follow.
    // They will return the 'scans' that happened along the way.
    // - If a span originates from a 'scan' then lift the text into an ASTNode.
    fn walker(&self, root: &Rc<Span>) -> Result<Vec<ASTNode>, String> {
        let mut args = Vec::new();
        match root.sources().iter().next() {
            Some(SpanSource::Completion(source, trigger)) => {
                args.extend(self.walker(source)?);
                args.extend(self.walker(trigger)?);
            },
            Some(SpanSource::Scan(source, trigger)) => {
                let symbol = source.next_symbol()
                    .expect("BUG: missing scan trigger symbol").name();
                args.extend(self.walker(source)?);
                args.push((self.terminal_parser)(symbol, trigger));
            },
            None => (),
        }
        self.reduce(root, args)
    }

    // for non-ambiguous grammars this retreieves the only possible parse
    pub fn eval_recursive(&self, ptrees: &ParseTrees) -> Result<ASTNode, String> {
        // walker will always return a Vec of size 1 because root.complete
        Ok(self.walker(ptrees.0.first().expect("BUG: ParseTrees empty"))?
           .swap_remove(0))
    }

    fn walker_all(&self, root: &Rc<Span>) -> Result<Vec<Vec<ASTNode>>, String> {
        let source = root.sources();
        if source.len() == 0 {
            return Ok(vec![self.reduce(root, Vec::new())?]);
        }
        let mut trees = Vec::new();
        for backpointer in source.iter() {
            match backpointer {
                SpanSource::Completion(source, trigger) => {
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
                SpanSource::Scan(source, trigger) => {
                    for mut args in self.walker_all(source)? {
                        let symbol = source.next_symbol()
                            .expect("BUG: missing scan trigger symbol").name();
                        args.push((self.terminal_parser)(symbol, trigger));
                        trees.push(self.reduce(root, args)?);
                    }
                }
            }
        }
        Ok(trees)
    }

    // Retrieves all parse trees
    pub fn eval_all_recursive(&self, ptrees: &ParseTrees) -> Result<Vec<ASTNode>, String> {
        let mut trees = Vec::new();
        for root in &ptrees.0 {
            trees.extend(
                self.walker_all(root)?.into_iter()
                    .map(|mut treevec| treevec.swap_remove(0)));
        }
        Ok(trees)
    }
}


struct ForestIterator {
    // A stack of (span, current-source-idx). 
    // Each time the iterator is advanced we advance the source-idx for the top span.
    // When that span exhausted all sources, we pop the top span. This results in a 
    // reset if it ever comes back from a different path. At the same time advance
    // the new top-of-stack span. If this one is exhausted, then rinse, repeat.
    source_idx: Vec<(Rc<Span>, usize)>,
}

impl ForestIterator {
    fn source_index(&mut self, cursor: &Rc<Span>) -> usize {
        if let Some(itidx) = self.source_idx.iter().find(|s| s.0 == *cursor) {
            itidx.1
        } else {
            self.source_idx.push((cursor.clone(), 0));
            0
        }
    }

    fn advance(&mut self) -> bool {
        while let Some((span, idx)) = self.source_idx.pop() {
            if idx + 1 < span.sources().len() {
                self.source_idx.push((span, idx + 1));
                return true;
            }
        }
        false
    }
}

impl<'a, ASTNode: Clone> EarleyForest<'a, ASTNode> {
    /*
    ## S -> S + N | N
    ## N -> [0-9]
    ## "1 + 2"

                 S -> S + N. 
                    /  \
                   /    \
              S +.N     N -> [0-9].
               / \             / \
              /   \           /   \
           S.+ N   "+"    .[0-9]   "2"
             /\
            /  \
       .S + N   S -> N.
                  /\
                 /  \
               .N    N -> [0-9].
                       / \
                      /   \
                  .[0-9]   "1"
    */
    fn eval_one(&self, root: Rc<Span>, mut selector: impl FnMut(&Rc<Span>) -> usize) -> Result<ASTNode, String> {
        let mut args = Vec::new();
        let mut completions = Vec::new();
        let mut spans = vec![root];

        while let Some(cursor) = spans.pop() {
            // As Earley chart is unwound keep a record of semantic actions to apply
            if cursor.complete() {
                completions.push(cursor.clone());
            }

            // (Reachable) Spans with no sources mean we've unwound to the
            // begining of a production/rule. Apply the rule reducing args.
            if cursor.sources().len() == 0 {
                let completed_rule = &completions.pop().expect("BUG: span rule never completed").rule;
                assert_eq!(&cursor.rule, completed_rule);
                // Get input AST nodes for this reduction. Stored reversed.
                let num_rule_slots = completed_rule.spec.len();
                let rule_args = args.split_off(args.len() - num_rule_slots).into_iter().rev().collect();
                // Apply the reduction.
                let rulename = completed_rule.to_string();
                let action = self.actions.get(&rulename).ok_or(format!("Missing Action: {}", rulename))?;
                args.push(action(rule_args));
            } else {
                let span_source_idx = selector(&cursor);
                // Walk the chart following span sources (back-pointers) of the tree.
                match &cursor.sources()[span_source_idx] {
                    // Completion sources -> Walk the chart. 
                    SpanSource::Completion(source, trigger) => {
                        spans.push(source.clone());
                        spans.push(trigger.clone());
                    },
                    // Scan sources -> lift scanned tokens into AST nodes.
                    SpanSource::Scan(source, trigger) => {
                        let symbol = source.next_symbol()
                            .expect("BUG: missing scan trigger symbol").name();
                        args.push((self.terminal_parser)(symbol, trigger));
                        spans.push(source.clone());
                    },
                }
            }
        }
        assert_eq!(args.len(), 1);
        Ok(args.pop().expect("BUG: mismatched reduce args"))
    }

    pub fn eval(&self, ptrees: &ParseTrees) -> Result<ASTNode, String> {
        let root = ptrees.0.first().expect("BUG: ParseTrees empty").clone();
        self.eval_one(root, |_| 0)
    }

    pub fn eval_all(&self, ptrees: &ParseTrees) -> Result<Vec<ASTNode>, String> {
        let mut results = Vec::new();
        for root in &ptrees.0 {
            let mut fi = ForestIterator{source_idx: Vec::new()};
            let mut iterator_has_more_items = true;
            while iterator_has_more_items {
                results.push(self.eval_one(root.clone(), |s| fi.source_index(s))?);
                iterator_has_more_items = fi.advance();
            }
        }
        Ok(results)
    }
}
