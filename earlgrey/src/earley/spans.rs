#![deny(warnings)]

use super::grammar::{Rule, Symbol};
use std::rc::Rc;
use std::{cell, fmt, hash};

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum SpanSource {
    Completion(Rc<Span>, Rc<Span>),
    Scan(Rc<Span>, String),
}

/// An Span is a partially matched `Rule`. `dot` shows the match progress.
pub struct Span {
    pub rule: Rc<Rule>, // LR0item (dotted rule)
    pub dot: usize,     // dot position within the rule
    pub start: usize,   // input stream position where item starts
    pub end: usize,     // input stream position where item ends

    // Need a RefCell to update existing Spans. A replacement with the union
    // of backpointers would invalidate other Spans already pointing to this one.
    // Those invalidated items wouldn't have the whole back-pointer list.
    /// backpointers leading to this item: (source-item, Scan/Completion)
    backpointers: cell::RefCell<Vec<SpanSource>>,
}

// Spans are deduped only by rule, dot, start, end (ie: not bp)
// The intention is that 2 Spans are the same and can be merged ignoring bp.
impl hash::Hash for Span {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.dot.hash(state);
        self.start.hash(state);
        self.end.hash(state);
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Span) -> bool {
        self.rule == other.rule
            && self.dot == other.dot
            && self.start == other.start
            && self.end == other.end
    }
}

impl Eq for Span {}

impl Span {
    // Unwind backpointers recursively
    pub fn stringify(&self, nest: usize) -> String {
        let pre = self
            .rule
            .spec
            .iter()
            .take(self.dot)
            .map(|s| s.name())
            .collect::<Vec<_>>()
            .join(" ");
        let post = self
            .rule
            .spec
            .iter()
            .skip(self.dot)
            .map(|s| s.name())
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "({} - {}) {} -> {} \u{00b7} {} #bp: {}{}",
            self.start,
            self.end,
            self.rule.head,
            pre,
            post,
            self.backpointers.borrow().len(),
            self.stringify_bp(nest + 1)
        )
    }

    fn stringify_bp(&self, nest: usize) -> String {
        let mut out = String::new();
        let pfx = "   ".repeat(nest);
        for bp in self.backpointers.borrow().iter() {
            match bp {
                SpanSource::Completion(a, b) => {
                    out += format!(
                        "\n{}Complete(\n{}   {}, \n{}   {}\n{})",
                        pfx,
                        pfx,
                        a.stringify(nest + 1),
                        pfx,
                        b.stringify(nest + 1),
                        pfx
                    )
                    .as_str();
                }
                SpanSource::Scan(a, b) => {
                    out += format!(
                        "\n{}Scan(\n{}   {}, \n{}   {}\n{})",
                        pfx,
                        pfx,
                        a.stringify(nest + 1),
                        pfx,
                        b,
                        pfx
                    )
                    .as_str();
                }
            }
        }
        out
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pre = self
            .rule
            .spec
            .iter()
            .take(self.dot)
            .map(|s| s.name())
            .collect::<Vec<_>>()
            .join(" ");
        let post = self
            .rule
            .spec
            .iter()
            .skip(self.dot)
            .map(|s| s.name())
            .collect::<Vec<_>>()
            .join(" ");
        write!(
            f,
            "({} - {}) {} -> {} \u{00b7} {} #bp: {}",
            self.start,
            self.end,
            self.rule.head,
            pre,
            post,
            self.backpointers.borrow().len()
        )
    }
}

impl Span {
    /// Span is complete if Rule has being fully matched
    pub fn complete(&self) -> bool {
        self.dot >= self.rule.spec.len()
    }

    /// Exposes the next symbol in the progress of the Rule
    pub fn next_symbol(&self) -> Option<&Symbol> {
        self.rule.spec.get(self.dot).map(|sym| &**sym)
    }

    /// Scans or Completions that led to the creation of this Span.
    /// Only ever borrowed non-mutable ref returned for public consumption
    pub fn sources(&self) -> cell::Ref<Vec<SpanSource>> {
        self.backpointers.borrow()
    }

    /// Merge other Span into this one moving over its backpointers
    // NOTE: backpointers used to be a HashSet for dedup but this container
    // being small we can use a Vec with O(n) seach without a real perf hit.
    // This gives us ordered+indexable iteration over back-pointers.
    pub fn merge_sources(&self, other: Span) {
        assert_eq!(*self, other, "Spans to merge should be Eq");
        let mut dest_bp = self.backpointers.borrow_mut();
        for bp in other.backpointers.into_inner() {
            if !dest_bp.contains(&bp) {
                dest_bp.push(bp);
            }
        }
    }

    pub fn new(rule: &Rc<Rule>, start: usize) -> Span {
        Span {
            rule: rule.clone(),
            dot: 0,
            start,
            end: start,
            backpointers: cell::RefCell::new(Vec::new()),
        }
    }

    pub fn extend(extension: SpanSource, end: usize) -> Span {
        let source = match &extension {
            SpanSource::Completion(span, _) => span,
            SpanSource::Scan(span, _) => span,
        };
        Span {
            rule: source.rule.clone(),
            dot: source.dot + 1,
            start: source.start,
            end,
            backpointers: cell::RefCell::new([extension].into()),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn terminal(name: &str, pred: impl Fn(&str) -> bool + 'static) -> Rc<Symbol> {
        Rc::new(Symbol::Term(name.to_string(), Box::new(pred)))
    }

    fn nonterm(name: &str) -> Rc<Symbol> {
        Rc::new(Symbol::NonTerm(name.to_string()))
    }

    fn gen_rule1() -> Rc<Rule> {
        fn testfn(o: &str) -> bool {
            o.len() == 1 && "+-".contains(o)
        }
        // S -> S +- d
        Rc::new(Rule::new(
            "S",
            &[
                nonterm("S"),
                terminal("+-", testfn),
                terminal("d", |n| n.chars().all(|c| "123".contains(c))),
            ],
        ))
    }

    fn gen_rule2() -> Rc<Rule> {
        fn testfn(o: &str) -> bool {
            o.len() == 1 && "*/".contains(o)
        }
        // S -> S */ d
        Rc::new(Rule::new(
            "S",
            &[
                nonterm("S"),
                terminal("*/", testfn),
                terminal("d", |n| n.chars().all(|c| "123".contains(c))),
            ],
        ))
    }

    fn item(rule: Rc<Rule>, dot: usize, start: usize, end: usize) -> Span {
        Span {
            rule,
            dot,
            start,
            end,
            backpointers: RefCell::new(Vec::new()),
        }
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
        assert!(!item(rule1.clone(), 0, 0, 5)
            .next_symbol()
            .unwrap()
            .is_terminal());
        assert!(item(rule1.clone(), 2, 0, 5)
            .next_symbol()
            .unwrap()
            .is_terminal());
    }

    #[test]
    fn item_predict() {
        let rule1 = gen_rule1();
        let predict = Span::new(&rule1, 23);
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
        let scan = Span::extend(SpanSource::Scan(source.clone(), "+".to_string()), 2);
        assert_eq!(item(rule1.clone(), 2, 0, 2), scan);
        // Check scan item backpointers
        let scan_src = scan.sources();
        assert!(scan_src.contains(&SpanSource::Scan(source, "+".to_string())));
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
        let complete_based =
            Span::extend(SpanSource::Completion(source.clone(), trigger.clone()), 3);
        assert_eq!(item(rule1.clone(), 1, 0, 3), complete_based);
        // Check completion item backpointers
        let src = complete_based.sources();
        assert!(src.contains(&SpanSource::Completion(source, trigger)));
        assert_eq!(src.len(), 1);
    }

    #[test]
    fn item_merge_sources() {
        // Source: S -> . S + d
        let rule1 = gen_rule1();
        let source = Rc::new(item(rule1.clone(), 0, 0, 0));
        // rule3: S -> d
        let rule3 = Rc::new(Rule::new(
            "S",
            &[terminal("d", |n| n.chars().all(|c| "123".contains(c)))],
        ));
        // S -> d .
        let trigger1 = Rc::new(item(rule3, 1, 0, 1));
        // S -> S . + d
        let complete1 = Span::extend(SpanSource::Completion(source.clone(), trigger1.clone()), 1);
        assert_eq!(complete1, item(rule1.clone(), 1, 0, 1));
        // rule4: S -> hex
        let rule4 = Rc::new(Rule::new("S", &[terminal("hex", |n| n == "0x3")]));
        // S -> hex .
        let trigger3 = Rc::new(item(rule4, 1, 0, 1));
        // S -> S . + d
        let complete3 = Span::extend(SpanSource::Completion(source.clone(), trigger3.clone()), 1);
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
