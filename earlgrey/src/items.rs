#![deny(warnings)]

use grammar::{Symbol, Rule};
use std::{cell, fmt, hash, iter, slice};
use std::collections::HashSet;
use std::rc::Rc;


#[derive(PartialEq,Eq,Hash)]
pub enum Trigger {
    Complete(Rc<Item>),
    Scan(String),
}

// Earley items
pub struct Item {
    pub rule: Rc<Rule>,  // LR0item (dotted rule)
    pub dot: usize,      // dot position within the rule
    pub start: usize,    // stream position where item starts
    pub end: usize,      // stream position where item ends
    // backpointers leading to this item: (source-item, Scan/Complete)
    bp: cell::RefCell<HashSet<(Rc<Item>, Trigger)>>
}


// Items are deduped only by rule, dot, start, end (ie: not bp)
// This is needed to insert into StateSet merging back-pointers
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

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pre = self.rule.spec.iter().take(self.dot)
            .map(|s| s.name()).collect::<Vec<_>>().join(" ");
        let post = self.rule.spec.iter().skip(self.dot)
            .map(|s| s.name()).collect::<Vec<_>>().join(" ");
        write!(f, "({} - {}) {} -> {} \u{00b7} {} #bp: {}",
               self.start, self.end, self.rule.head, pre, post,
               self.bp.borrow().len())
    }
}

impl Item {
    pub fn complete(&self) -> bool { self.dot >= self.rule.spec.len() }

    pub fn next_symbol(&self) -> Option<&Symbol> {
        self.rule.symbol_at(self.dot).map(|s| &**s)
    }

    pub fn source(&self) -> cell::Ref<HashSet<(Rc<Item>, Trigger)>> {
        self.bp.borrow()
    }

    // check if other item's next-symbol matches our rule's name
    fn can_complete(&self, other: &Rc<Item>) -> bool {
        self.complete() && match other.next_symbol() {
            Some(&Symbol::NonTerm(ref name)) => name == &self.rule.head,
            _ => false
        }
    }

    // check item's next symbol is a temrinal that scans lexeme
    pub fn can_scan(&self, lexeme: &str) -> bool {
        match self.next_symbol() {
            Some(&Symbol::Terminal(_, ref f)) => f(lexeme),
            _ => false
        }
    }

    // build a new Item for a prediction
    pub fn predict_new(rule: &Rc<Rule>, start: usize) -> Item {
        Item{rule: rule.clone(), dot: 0, start, end: start,
             bp: cell::RefCell::new(HashSet::new())}
    }

    // produce an Item after scanning a token
    pub fn scan_new(source: &Rc<Item>, end: usize, input: &str) -> Item {
        let mut _bp = HashSet::new();
        _bp.insert((source.clone(), Trigger::Scan(input.to_string())));
        Item{rule: source.rule.clone(), dot: source.dot+1,
             start: source.start, end, bp: cell::RefCell::new(_bp)}
    }

    // produce an Item by completing another one
    fn complete_new(source: &Rc<Item>, trigger: &Rc<Item>, end: usize) -> Item {
        let mut _bp = HashSet::new();
        _bp.insert((source.clone(), Trigger::Complete(trigger.clone())));
        Item{rule: source.rule.clone(), dot: source.dot+1,
             start: source.start, end, bp: cell::RefCell::new(_bp)}
    }
}

///////////////////////////////////////////////////////////////////////////////

// StateSets keep deduped elements tracking insertion order
pub struct StateSet {
    order: Vec<Rc<Item>>,
    dedup: HashSet<Rc<Item>>,
}


impl Extend<Item> for StateSet {
    fn extend<I: IntoIterator<Item=Item>>(&mut self, iterable: I) {
        for item in iterable { self.push(item); }
    }
}

impl iter::FromIterator<Item> for StateSet {
    fn from_iter<I: IntoIterator<Item=Item>>(iterable: I) -> Self {
        let mut ss = StateSet::new();
        ss.extend(iterable.into_iter());
        ss
    }
}

impl fmt::Debug for StateSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.order.fmt(f) }
}

// Statesets are filled with Items via push/extend, these are boxed to share BP
// See implementations of Hash + PartialEq + Eq for Item excluding Item::bp
impl StateSet {
    pub fn new() -> StateSet {
        StateSet{order: Vec::new(), dedup: HashSet::new()}
    }

    // Add Earley Items into the set. If the Item already exists we merge bp
    fn push(&mut self, item: Item) {
        if let Some(existent) = self.dedup.get(&item) {
            existent.bp.borrow_mut().extend(item.bp.into_inner());
            return;
        }
        let item = Rc::new(item);
        self.order.push(item.clone());
        self.dedup.insert(item);
    }

    pub fn len(&self) -> usize { self.dedup.len() }

    // Iterate on insertion order
    pub fn iter<'a>(&'a self) -> slice::Iter<'a, Rc<Item>> { self.order.iter() }

    // get all items whose rule head matches
    pub fn filter_rule_head<'a, S: Into<String>>(&'a self, head: S) ->
           Box<Iterator<Item=&'a Rc<Item>> + 'a> {
        let head = head.into();
        Box::new(self.order.iter().filter(move |it| it.rule.head == head))
    }

    // Produce new items by advancing the dot on items completed by 'item' trig
    pub fn completed_by(&self, item: &Rc<Item>, at: usize) -> Vec<Item> {
        self.order.iter()
            .filter(|source| item.can_complete(source))
            .map(|source| Item::complete_new(source, item, at))
            .collect::<Vec<_>>()
    }

    // Produce new items by advancing the dot on items that can 'scan' lexeme
    pub fn advanced_by_scan(&self, lexeme: &str, end: usize) -> Vec<Item> {
        self.order.iter()
            .filter(|item| item.can_scan(lexeme))
            .map(|item| Item::scan_new(item, end, lexeme))
            .collect::<Vec<_>>()
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::collections::HashSet;
    use std::cell::RefCell;
    use super::{Rule, Item, Symbol, StateSet, Trigger};

    fn gen_rule1() -> Rc<Rule> {
        fn testfn(o: &str) -> bool { o.len() == 1 && "+-".contains(o) }
        Rc::new(Rule{
            head: "S".to_string(),
            spec: vec![
                Rc::new(Symbol::NonTerm("S".to_string())),
                Rc::new(Symbol::Terminal("+-".to_string(), Box::new(testfn))),
                Rc::new(Symbol::Terminal("d".to_string(), Box::new(|n|
                                      n.chars().all(|c| "123".contains(c))))),
            ]})
    }

    fn gen_rule2() -> Rc<Rule> {
        fn testfn(o: &str) -> bool { o.len() == 1 && "*/".contains(o) }
        Rc::new(Rule{
            head: "M".to_string(),
            spec: vec![
                Rc::new(Symbol::NonTerm("M".to_string())),
                Rc::new(Symbol::Terminal("*/".to_string(), Box::new(testfn))),
                Rc::new(Symbol::Terminal("d".to_string(), Box::new(|n|
                                      n.chars().all(|c| "123".contains(c))))),
            ]})
    }

    fn item(rule: Rc<Rule>, dot: usize, start: usize, end: usize) -> Item {
        Item{rule: rule, dot: dot, start: start, end: end,
             bp: RefCell::new(HashSet::new())}
    }

    #[test]
    fn item_eq() {
        let rule1 = gen_rule1();
        let rule2 = gen_rule2();
        let i = Item::predict_new(&rule1, 0);
        let j = Item::predict_new(&rule2, 0);
        assert_eq!(i, Item::predict_new(&rule1, 0));
        assert_eq!(j, Item::predict_new(&rule2, 0));
        assert_ne!(i, j);
        assert_ne!(i, Item::predict_new(&rule1, 1));
    }

    #[test]
    fn scan_eq() {
        let i = Rc::new(item(gen_rule1(), 2, 0, 0));
        // i Item is doted after '/*', so it can scan a digit
        assert!(i.can_scan("1"));
        let i2 = Item::scan_new(&i, 1, "3");
        assert_eq!(i2, item(gen_rule1(), 3, 0, 1));
        // Assert i2 has back pointer
        assert_eq!(i2.source().len(), 1);
        assert!(i2.source().contains(&(i, Trigger::Scan("3".to_string()))));
    }

    #[test]
    fn stateset_dups() {
        let rule = gen_rule2();
        //check that items are deduped in statesets
        let mut ss = StateSet::new();
        ss.push(item(rule.clone(), 0, 0, 0));
        ss.push(item(rule.clone(), 0, 0, 0));
        assert_eq!(ss.len(), 1);
        ss.push(item(rule.clone(), 1, 0, 1));
        assert_eq!(ss.len(), 2);
        ss.push(item(rule.clone(), 1, 0, 1));
        assert_eq!(ss.len(), 2);
        ss.push(item(rule.clone(), 2, 0, 1));
        assert_eq!(ss.len(), 3);
    }
}
