use earley::{Terminal, NonTerminal, Symbol, Rule, Item, StateSet};
use std::rc::Rc;

#[test]
fn test1() {
    fn ops(c: &str) -> bool {
        let ops = "+-";
        c.len() == 1 && ops.contains(c)
    }

    let term_ops = Symbol::terminal(ops);
    let term_num = Symbol::terminal(|c: &str| {
        let nums = "1234567890";
        c.len() == 1 && nums.contains(c)
    });

    let rule1 = Rule::new("Sum", vec![
      Symbol::nonterm("Sum"),
      term_ops,
      Symbol::nonterm("Number")]);

    let rule2 = Rule::new("Sum", vec![
      Symbol::nonterm("Sum"),
      term_ops,
      Symbol::nonterm("Number")]);

    assert_eq!(rule1, rule2);

        /*
    // can only be used once, will be moved
    let numeric = |c: &str| {
        let nums = "1234567890";
        c.len() == 1 && nums.contains(c)
    };

    // Sum    -> Sum [+-] Number | Number
    // Number -> [0-9] Number | [0-9] (fancy [0-9]+)
    let rules = vec![
        Rule::new("Sum", vec![
                  Symbol::nonterm("Sum"),
                  Symbol::terminal(ops),
                  Symbol::nonterm("Number")]),
        Rule::new("Sum", vec![
                  Symbol::nonterm("Number")]),
        Rule::new("Number", vec![
                  Symbol::terminal(numeric),
                  Symbol::nonterm("Number")]),
        Rule::new("Number", vec![
                  Symbol::terminal(numeric)]),
    ];
    */
}
