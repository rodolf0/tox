use earley::uniqvec::UniqVec;
use earley::types::*;
use std::rc::Rc;

#[cfg(test)]
fn ops(o: &str) -> bool {
    let ops = "+-";
    o.len() == 1 && ops.contains(o)
}

#[test]
fn symbol_uniqueness() {
    let s_sum = Rc::new(Symbol::from(NonTerminal::new("Sum")));
    let s_num = Rc::new(Symbol::from(Terminal::new("Num", |n: &str| {
                    let nums = "1234567890";
                    n.len() == 1 && nums.contains(n)
                })));
    let s_ops = Rc::new(Symbol::from(Terminal::new("Ops", ops)));

    let r1 = Rc::new(Rule{
        name: s_sum.clone(),
        spec: vec![s_sum.clone(), s_num.clone(), s_ops.clone()],
    });

    let i1 = Item{rule: r1.clone(), start: 0, dot: 0};
    let i2 = Item{rule: r1.clone(), start: 0, dot: 0};
    assert_eq!(i1, i2);

    // Check that Items work correctly with UniqVecs
    let mut state_set = UniqVec::new();
    state_set.push(i1);
    state_set.push(i2);
    assert_eq!(state_set.len(), 1);

    state_set.push(Item{rule: r1.clone(), start: 0, dot: 1});
    assert_eq!(state_set.len(), 2);
    state_set.push(Item{rule: r1.clone(), start: 0, dot: 1});
    assert_eq!(state_set.len(), 2);
}

#[test]
fn build_grammar() {
    let mut gb = GrammarBuilder::new();

    // register some symbols
    gb.symbol(NonTerminal::new("Sum"));
    gb.symbol(NonTerminal::new("Number"));
    gb.symbol(Terminal::new("[+-]", ops));
    gb.symbol(Terminal::new("[0-9]", |n: &str| {
        let nums = "1234567890";
        n.len() == 1 && nums.contains(n)
    }));

    gb.rule("Sum",    vec!["Sum", "[+-]", "Number"]);
    gb.rule("Sum",    vec!["Number"]);
    gb.rule("Number", vec!["[0-9]", "Number"]);
    gb.rule("Number", vec!["[0-9]"]);

    let g = gb.build("Sum");

    assert_eq!(g.start, g.symbols["Sum"]);
    assert_eq!(g.symbols.len(), 4);
    assert_eq!(g.rules["Sum"].len(), 2);
    assert_eq!(g.rules["Number"].len(), 2);
}
