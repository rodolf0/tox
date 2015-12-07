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

    let i1 = Item::new(r1.clone(), 0, 0);
    let i2 = Item::new(r1.clone(), 0, 0);
    assert_eq!(i1, i2);

    // Check that Items work correctly with UniqVecs
    let mut state_set = UniqVec::new();
    state_set.push(i1);
    state_set.push(i2);
    assert_eq!(state_set.len(), 1);

    state_set.push(Item::new(r1.clone(), 0, 1));
    assert_eq!(state_set.len(), 2);
    state_set.push(Item::new(r1.clone(), 0, 1));
    assert_eq!(state_set.len(), 2);
}

#[test]
fn build_grammar() {
    let mut gb = GrammarBuilder::new();
    gb.symbol(NonTerminal::new("Sum"))
      .symbol(NonTerminal::new("Number"))
      .symbol(Terminal::new("[+-]", ops))
      .symbol(Terminal::new("[0-9]", |n: &str| {
          let nums = "1234567890";
          n.len() == 1 && nums.contains(n)
      }));
    gb.rule("Sum",    vec!["Sum", "[+-]", "Number"])
      .rule("Sum",    vec!["Number"])
      .rule("Number", vec!["[0-9]", "Number"])
      .rule("Number", vec!["[0-9]"]);
    let g = gb.into_grammar("Sum");

    assert_eq!(g.start, g.symbols["Sum"]);
    assert_eq!(g.symbols.len(), 4);
    assert_eq!(g.rules.len(), 4);
    assert_eq!(g.nullable.len(), 0);
}

#[test]
fn test_nullable() {
    let mut gb = GrammarBuilder::new();
    gb.symbol(NonTerminal::new("A"))
      .symbol(NonTerminal::new("B"));
    gb.rule("A", Vec::new())
      .rule("A", vec!["B"])
      .rule("B", vec!["A"]);

    let g = gb.into_grammar("A");

    assert_eq!(g.start, g.symbols["A"]);
    assert_eq!(g.symbols.len(), 2);
    assert_eq!(g.nullable.len(), 2);
}
