#![deny(warnings)]

use crate::grammar::{Rule, Symbol};
use std::collections::HashSet;
use std::{cell, fmt, hash};
use std::rc::Rc;


#[derive(PartialEq,Eq,Hash,Debug,Clone)]
pub enum BackPointer {
    Complete(Rc<Item>, Rc<Item>),
    Scan(Rc<Item>, String),
}

/// An Item is a partially matched `Rule`. `dot` shows the match progress.
pub struct Item {
    pub rule: Rc<Rule>,  // LR0item (dotted rule)
    pub dot: usize,      // dot position within the rule
    pub start: usize,    // input stream position where item starts
    pub end: usize,      // input stream position where item ends

    // Need a RefCell to update existing Items. A replacement with the union
    // of backpointers would invalidate other Items already pointing to this one.
    // Those invalidated items wouldn't have the whole back-pointer list.
    /// backpointers leading to this item: (source-item, Scan/Complete)
    backpointers: cell::RefCell<HashSet<BackPointer>>,
}


// Items are deduped only by rule, dot, start, end (ie: not bp)
// The intention is that 2 Items are the same and can be merged ignoring bp.
impl hash::Hash for Item {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.dot.hash(state);
        self.start.hash(state);
        self.end.hash(state);
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.rule == other.rule &&
        self.dot == other.dot &&
        self.start == other.start &&
        self.end == other.end
    }
}

impl Eq for Item {}

impl Item {
    // Unwind backpointers recursively
    pub fn stringify(&self, nest: usize) -> String {
        let pre = self.rule.spec.iter().take(self.dot)
            .map(|s| s.name()).collect::<Vec<_>>().join(" ");
        let post = self.rule.spec.iter().skip(self.dot)
            .map(|s| s.name()).collect::<Vec<_>>().join(" ");
        format!("({} - {}) {} -> {} \u{00b7} {} #bp: {}{}",
               self.start, self.end, self.rule.head, pre, post,
               self.backpointers.borrow().len(),
               self.stringify_bp(nest + 1))
    }

    fn stringify_bp(&self, nest: usize) -> String {
        let mut out = String::new();
        let pfx = "   ".repeat(nest);
        for bp in self.backpointers.borrow().iter() {
            match bp {
                BackPointer::Complete(a, b) => {
                    out += format!("\n{}Complete(\n{}   {}, \n{}   {}\n{})",
                        pfx,
                        pfx, a.stringify(nest + 1),
                        pfx, b.stringify(nest + 1),
                        pfx).as_str();
                },
                BackPointer::Scan(a, b) => {
                    out += format!("\n{}Scan(\n{}   {}, \n{}   {}\n{})",
                        pfx,
                        pfx, a.stringify(nest + 1),
                        pfx, b,
                        pfx).as_str();
                }
            }
        }
        out
    }
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pre = self.rule.spec.iter().take(self.dot)
            .map(|s| s.name()).collect::<Vec<_>>().join(" ");
        let post = self.rule.spec.iter().skip(self.dot)
            .map(|s| s.name()).collect::<Vec<_>>().join(" ");
        write!(f, "({} - {}) {} -> {} \u{00b7} {} #bp: {}",
               self.start, self.end, self.rule.head, pre, post,
               self.backpointers.borrow().len())
    }
}

impl Item {
    /// Item is complete if Rule has being fully matched
    pub fn complete(&self) -> bool {
        self.dot >= self.rule.spec.len()
    }

    /// Exposes the next symbol in the progress of the Rule
    pub fn next_symbol(&self) -> Option<&Symbol> {
        self.rule.spec.get(self.dot).map(|sym| &**sym)
    }

    /// Scans or Completions that led to the creation of this Item.
    /// only ever borrowed non-mutable ref returned for public consumption
    pub fn sources(&self) -> cell::Ref<HashSet<BackPointer>> {
        self.backpointers.borrow()
    }

    /// Merge other Item into this one moving over its backpointers
    pub fn merge_sources(&self, other: Item) {
        assert_eq!(*self, other, "Items to merge should be Eq");
        let other_bp = other.backpointers.into_inner();
        self.backpointers.borrow_mut().extend(other_bp);
    }

    /// Build a new `Prediction` based Item.
    pub fn predict_new(rule: &Rc<Rule>, start: usize) -> Item {
        Item{
            rule: rule.clone(),
            dot: 0,
            start,
            end: start,
            backpointers: cell::RefCell::new(HashSet::new()),
        }
    }

    /// Build `Scan` based Items.
    /// An item where the rule is advanced by matching a terminal.
    pub fn scan_new(source: &Rc<Item>, end: usize, input: &str) -> Item {
        let mut _bp = HashSet::new();
        _bp.insert(BackPointer::Scan(source.clone(), input.to_string()));
        Item{
            rule: source.rule.clone(),
            dot: source.dot + 1,
            start: source.start,
            end,
            backpointers: cell::RefCell::new(_bp),
        }
    }

    /// Build `Completion` based Items.
    /// `Rule` is advanced because its next symbol matches the completed `trigger`.
    pub fn complete_new(source: &Rc<Item>, trigger: &Rc<Item>, end: usize) -> Item {
        let mut _bp = HashSet::new();
        _bp.insert(BackPointer::Complete(source.clone(), trigger.clone()));
        Item{
            rule: source.rule.clone(),
            dot: source.dot + 1,
            start: source.start,
            end,
            backpointers: cell::RefCell::new(_bp),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::collections::HashSet;
    use std::cell::RefCell;
    use super::{Rule, Item, Symbol, BackPointer};

    fn terminal(name: impl Into<String>, pred: impl Fn(&str) -> bool + 'static) -> Rc<Symbol> {
        Rc::new(Symbol::Term(name.into(), Box::new(pred)))
    }

    fn nonterm(name: impl Into<String>) -> Rc<Symbol> {
        Rc::new(Symbol::NonTerm(name.into()))
    }

    fn gen_rule1() -> Rc<Rule> {
        fn testfn(o: &str) -> bool { o.len() == 1 && "+-".contains(o) }
        // S -> S +- d
        Rc::new(Rule::new("S", &[
            nonterm("S"),
            terminal("+-", testfn),
            terminal("d", |n| n.chars().all(|c| "123".contains(c))),
        ]))
    }

    fn gen_rule2() -> Rc<Rule> {
        fn testfn(o: &str) -> bool { o.len() == 1 && "*/".contains(o) }
        // S -> S */ d
        Rc::new(Rule::new("S", &[
            nonterm("S"),
            terminal("*/", testfn),
            terminal("d", |n| n.chars().all(|c| "123".contains(c))),
        ]))
    }

    fn item(rule: Rc<Rule>, dot: usize, start: usize, end: usize) -> Item {
        Item{rule, dot, start, end, backpointers: RefCell::new(HashSet::new())}
    }

    #[test]
    fn item_basics() {
        // Check item equality
        let rule1 = gen_rule1();
        let rule2 = gen_rule2();
        assert_eq!(item(rule1.clone(), 0, 0, 0), item(rule1.clone(), 0, 0, 0));
        assert_ne!(item(rule2.clone(), 0, 0, 0), item(rule1.clone(), 0, 0, 0));
        assert_ne!(item(rule1.clone(), 1, 0, 0), item(rule1.clone(), 0, 0, 0));
        // Check item complete
        assert!(!item(rule2.clone(), 2, 0, 5).complete());
        assert!(item(rule2.clone(), 3, 0, 4).complete());
        // Check next symbol
        assert!(!item(rule1.clone(), 0, 0, 5).next_symbol().unwrap().is_terminal());
        assert!(item(rule1.clone(), 2, 0, 5).next_symbol().unwrap().is_terminal());
    }

    #[test]
    fn item_predict() {
        let rule1 = gen_rule1();
        let predict = Item::predict_new(&rule1, 23);
        assert_eq!(item(rule1.clone(), 0, 23, 23), predict);
        assert_eq!(predict.start, predict.end);
        assert_eq!(predict.sources().len(), 0);
    }

    #[test]
    fn item_scan() {
        // Source: S -> S . + d
        let rule1 = gen_rule1();
        let source = Rc::new(item(rule1.clone(), 1, 0, 1));
        // Scan a '+' token
        let scan = Item::scan_new(&source, 2, "+");
        assert_eq!(item(rule1.clone(), 2, 0, 2), scan);
        // Check scan item backpointers
        let scan_src = scan.sources();
        assert!(scan_src.contains(&BackPointer::Scan(source, "+".to_string())));
        assert_eq!(scan_src.len(), 1);
    }

    #[test]
    fn item_complete() {
        // Input could be: 2 * 3 + 1
        // Source: S -> . S + d
        let rule1 = gen_rule1();
        let source = Rc::new(item(rule1.clone(), 0, 0, 0));
        // A trigger reaches completion (2 * 3) - S -> S * d .
        let rule2 = gen_rule2();
        let trigger = Rc::new(item(rule2.clone(), 0, 0, 3));
        // generate completion
        let complete_based = Item::complete_new(&source, &trigger, 3);
        assert_eq!(item(rule1.clone(), 1, 0, 3), complete_based);
        // Check completion item backpointers
        let src = complete_based.sources();
        assert!(src.contains(&BackPointer::Complete(source, trigger)));
        assert_eq!(src.len(), 1);
    }

    #[test]
    fn item_merge_sources() {
        // Source: S -> . S + d
        let rule1 = gen_rule1();
        let source = Rc::new(item(rule1.clone(), 0, 0, 0));
        // rule3: S -> d
        let rule3 = Rc::new(Rule::new("S", &[
            terminal("d", |n| n.chars().all(|c| "123".contains(c))),
        ]));
        // S -> d .
        let trigger1 = Rc::new(item(rule3, 1, 0, 1));
        // S -> S . + d
        let complete1 = Item::complete_new(&source, &trigger1, 1);
        assert_eq!(complete1, item(rule1.clone(), 1, 0, 1));
        // rule4: S -> hex
        let rule4 = Rc::new(Rule::new("S", &[terminal("hex", |n| n == "0x3")]));
        // S -> hex .
        let trigger3 = Rc::new(item(rule4, 1, 0, 1));
        // S -> S . + d
        let complete3 = Item::complete_new(&source, &trigger3, 1);
        assert_eq!(complete3, item(rule1.clone(), 1, 0, 1));
        // Merge complete1 / complete3
        assert!(complete1.sources().len() == 1);
        assert!(complete3.sources().len() == 1);
        complete1.merge_sources(complete3);
        assert!(complete1.sources().len() == 2);
    }

    #[test]
    #[should_panic]
    fn item_failed_merge_1() {
        let rule1 = gen_rule1();
        let item1 = item(rule1.clone(), 0, 0, 0);
        let item2 = item(rule1.clone(), 0, 1, 0);
        item1.merge_sources(item2);
    }

    #[test]
    #[should_panic]
    fn item_failed_merge_2() {
        let item3 = item(gen_rule2(), 0, 0, 0);
        let item4 = item(gen_rule2(), 0, 0, 0);
        item3.merge_sources(item4);
    }
}
