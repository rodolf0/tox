#![deny(warnings)]

use crate::grammar::{Rule, Grammar, Symbol};
use crate::items::Item;
use std::collections::HashSet;
use std::rc::Rc;
use std::fmt::Debug;

pub struct EarleyParser {
    pub grammar: Grammar,
}

#[derive(Debug)]
pub struct ParseTrees(pub Vec<Rc<Item>>);

///////////////////////////////////////////////////////////////////////////////

impl EarleyParser {
    pub fn new(grammar: Grammar) -> EarleyParser {
        EarleyParser{grammar}
    }

    /// Build new `Prediction` items from `next_terminal` of some Symbol:
    fn predictions<'r>(
        rules: impl Iterator<Item=&'r Rc<Rule>> + 'r,
        next_terminal: &'r str,
        start_pos: usize,
    ) -> Box<dyn Iterator<Item=Item> + 'r>
    {
        Box::new(rules.filter(move |rule| rule.head == next_terminal)
            .map(move |rule| Item::predict_new(rule, start_pos)))
    }

    /// Build new `Completion` items based on `trigger` item having completed.
    /// When an item is completed it advances all items in the same starting
    /// StateSet whose next symbol matches its rule name.
    fn completions<'r>(
        starting_stateset: impl Iterator<Item=&'r Rc<Item>> + 'r,
        trigger: &'r Rc<Item>,
        complete_pos: usize,
    ) -> Box<dyn Iterator<Item=Item> + 'r>
    {
        assert!(trigger.complete(), "Incomplete `trigger` used for completions");
        Box::new(starting_stateset.filter(move |item| {
            match item.next_symbol() {
                Some(Symbol::NonTerm(name)) => name == &trigger.rule.head,
                _ => false
            }
        }).map(move |item| Item::complete_new(item, trigger, complete_pos)))
    }

    /// Build new `Scan` items for items in the current stateset whose next
    /// symbol is a Terminal that matches the input lexeme ahead in the stream.
    fn scans<'r>(
        current_stateset: impl Iterator<Item=&'r Rc<Item>> + 'r,
        lexeme: &'r str,
        end: usize,
    ) -> impl Iterator<Item=Rc<Item>> + 'r
    {
        current_stateset.filter(move |item| 
            // check item's next symbol is a temrinal that scans lexeme
            item.next_symbol().is_some_and(|s| s.matches(lexeme))
        ).map(move |item| Rc::new(Item::scan_new(item, end, lexeme)))
    }

    pub fn parse<T>(&self, mut tokenizer: T) -> Result<ParseTrees, String>
            where T: Iterator, T::Item: Debug + AsRef<str> {

        // Populate S0, add items for each rule matching the start symbol
        let s0: HashSet<_> = self.grammar.rules.iter()
            .filter(|rule| rule.head == self.grammar.start)
            .map(|rule| Rc::new(Item::predict_new(rule, 0)))
            .collect();

        let mut statesets = vec![s0];

        // New statesets are generated from input stream (Scans)
        for idx in 0.. {
            // Predict/Complete until no new Items are added to the StateSet
            // Instead of looping we could pre-populate completions of nullable symbols
            loop {
                let new_items: Vec<_> = statesets[idx].iter().flat_map(|trigger| {
                    let next_sym = trigger.next_symbol();
                    if let Some(Symbol::NonTerm(name)) = next_sym {
                        EarleyParser::predictions(self.grammar.rules.iter(), name, idx)
                    } else if trigger.complete() {
                        assert!(next_sym.is_none(), "Expected next symbol to be None");
                        EarleyParser::completions(statesets[trigger.start].iter(), trigger, idx)
                    } else {
                        // Scan items populate next stateset only when done with current state
                        assert!(matches!(next_sym, Some(&Symbol::Term(_, _))));
                        Box::new(std::iter::empty())
                    }
                }).collect();
                let stateset = statesets.get_mut(idx).unwrap();
                let prev_len = stateset.len();
                // Add new items to the current stateset merging existing ones
                for new_item in new_items {
                    if let Some(existent) = stateset.get(&new_item) {
                        existent.merge_sources(new_item);
                    } else {
                        stateset.insert(Rc::new(new_item));
                    }
                }
                // do precitions/completions until expansions are exhausted
                if prev_len == stateset.len() {
                    break;
                }
            }
            // Build Si+1 with items in the current state that accept the next token
            if let Some(lexeme) = tokenizer.next() {
                statesets.push(EarleyParser::scans(
                    statesets[idx].iter(), lexeme.as_ref(), idx + 1).collect());
            } else {
                break;
            }
        }

        // debug StateSets
        if cfg!(feature="debug") {
            for (idx, stateset) in statesets.iter().enumerate() {
                eprintln!("=== StateSet {} ===", idx);
                stateset.iter().inspect(|item| {
                    let src = item.sources().iter()
                        .map(|bp| format!("{:?}", bp))
                        .collect::<Vec<_>>().join(", ");
                    eprintln!("{:?} -- SRC: {}", item, src);
                }).count();
            }
        }

        // Check that at least one item is a. complete, b. starts at the idx 0,
        // and c. the name of the rule matches the starting symbol.
        // It spans the whole input because we search at the last stateset
        let parse_trees: Vec<_> = statesets.pop()
            .expect("No Statesets (even s0)")
            .iter()
            .filter(|item| item.start == 0 && item.complete() &&
                           item.rule.head == self.grammar.start)
            .cloned()
            .collect();
        if parse_trees.is_empty() {
            return Err("Parse Error: No Rule completes".to_string());
        }
        if cfg!(feature="debug") {
            eprintln!("=== Parse Trees ===");
            for t in &parse_trees {
                eprintln!("{}", t.stringify(0));
            }
        }
        Ok(ParseTrees(parse_trees))
    }
}
