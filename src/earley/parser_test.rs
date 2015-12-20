use earley::symbol::Symbol;
use earley::items::{Rule, Item, StateSet};
use earley::grammar::{GrammarBuilder, Grammar};
use std::rc::Rc;

#[test]
fn symbol_uniqueness() {
    // test equalty operators
    fn testfn(o: &str) -> bool { o.len() == 1 && "+-".contains(o) }
    assert_eq!(Symbol::nonterm("sym1"), Symbol::nonterm("sym1"));
    // comparing Fn trait (data, vtable) so shouldn't match
    assert!(Symbol::terminal("+-", testfn) != Symbol::terminal("+-", testfn));
    assert!(Symbol::terminal("X", |_| true) != Symbol::terminal("X", |_| true));

    let rule = {
        let s = Rc::new(Symbol::nonterm("S"));
        let add_op = Rc::new(Symbol::terminal("+-", testfn));
        let num = Rc::new(Symbol::terminal("[0-9]", |n: &str| {
                            n.len() == 1 && "1234567890".contains(n)}));
        Rc::new(Rule::new(s.clone(), vec![s, add_op, num]))
    };

    // test item comparison
    assert_eq!(Item::new(rule.clone(), 0, 0), Item::new(rule.clone(), 0, 0));
    assert!(Item::new(rule.clone(), 0, 0) != Item::new(rule.clone(), 0, 1));

    // check that items are deduped in statesets
    let mut ss = StateSet::new();
    ss.push(Item::new(rule.clone(), 0, 0));
    ss.push(Item::new(rule.clone(), 0, 0));
    assert_eq!(ss.len(), 1);
    ss.push(Item::new(rule.clone(), 1, 0));
    assert_eq!(ss.len(), 2);

    let ix = Item::new(rule.clone(), 2, 3);
    let vi = vec![ix.clone(), ix.clone(), ix.clone(), ix.clone()];
    ss.extend(vi.into_iter());
    assert_eq!(ss.len(), 3);
}

#[test]
fn test_nullable() {
    let mut gb = GrammarBuilder::new();
    gb.symbol(Symbol::nonterm("A"))
      .symbol(Symbol::nonterm("B"));
    gb.rule("A", Vec::new())
      .rule("A", vec!["B"])
      .rule("B", vec!["A"]);
    let g = gb.into_grammar("A");
    assert_eq!(g.start, g.symbols["A"]);
    assert_eq!(g.symbols.len(), 2);
    assert_eq!(g.nullable.len(), 2);
}

fn build_grammar() -> Grammar {
    let mut gb = GrammarBuilder::new();

    gb.symbol(Symbol::nonterm("Sum"))
      .symbol(Symbol::nonterm("Product"))
      .symbol(Symbol::nonterm("Factor"))
      .symbol(Symbol::terminal("Number", |n: &str| {
          n.chars().all(|c| "1234567890".contains(c))
        }))
      .symbol(Symbol::terminal("[+-]", |n: &str| {
          n.len() == 1 && "+-".contains(n)
        }))
      .symbol(Symbol::terminal("[*/]", |n: &str| {
          n.len() == 1 && "*/".contains(n)
        }))
      .symbol(Symbol::terminal("(", |n: &str| { n == "(" }))
      .symbol(Symbol::terminal(")", |n: &str| { n == ")" }));

    gb.rule("Sum",     vec!["Sum", "[+-]", "Product"])
      .rule("Sum",     vec!["Product"])
      .rule("Product", vec!["Product", "[*/]", "Factor"])
      .rule("Product", vec!["Factor"])
      .rule("Factor",  vec!["(", "Sum", ")"])
      .rule("Factor",  vec!["Number"]);

    let g = gb.into_grammar("Sum");
    assert_eq!(g.start, g.symbols["Sum"]);
    assert_eq!(g.symbols.len(), 8);
    assert_eq!(g.rules("Sum").count(), 2);
    assert_eq!(g.rules("Product").count(), 2);
    assert_eq!(g.rules("Factor").count(), 2);
    g
}

///////////////////////////////////////////////////////////////////////////////

use earley::{Lexer, EarleyParser};

#[test]
fn print_statesets() {
    let p = EarleyParser::new(build_grammar());
    let mut input = Lexer::from_str("1+(2*3-4)", "+*-/()");
    let state = p.parse(&mut input).unwrap();
    assert_eq!(state.len(), 10);

    for (idx, stateset) in state.iter().enumerate() {
        println!("=== {} ===", idx);
        for i in stateset.iter() {
            println!("{:?}", i);
        }
    }
}

#[test]
fn test_ambiguous_grammar() {
    // S -> SS | b
    // Earley's corner case that generates spurious trees for bbb
    let mut gb = GrammarBuilder::new();
    gb.symbol(Symbol::nonterm("S"))
      .symbol(Symbol::terminal("b", |n: &str| n == "b"))
      .rule("S", vec!["S", "S"])
      .rule("S", vec!["b"]);
    let mut input = Lexer::from_str("b b b", " ");
    let p = EarleyParser::new(gb.into_grammar("S"));
    let states = p.parse(&mut input).unwrap();
    assert_eq!(states.len(), 4);
}

#[test]
fn test_badparse() {
    let g = build_grammar();
    let mut input = Lexer::from_str("1+", "+*-/()");
    let p = EarleyParser::new(g);
    assert!(p.parse(&mut input).is_err());
}

//#[test]
//fn test3() {
    //let mut gb = GrammarBuilder::new();
    //// Build bogus grammar
    //gb.symbol(NonTerminal::new("A"));
    //gb.symbol(NonTerminal::new("B"));
    //gb.rule("A", Vec::new());
    //gb.rule("A", vec!["B"]);
    //gb.rule("B", vec!["A"]);

    //let g = gb.into_grammar("A");

    //assert_eq!(g.start, g.symbols["A"]);
    //assert_eq!(g.symbols.len(), 2);

    //let mut input = Lexer::from_str("", "-");
    //let p = EarleyParser::new(g);

    //let state = p.parse(&mut input).unwrap();
    //for (idx, stateset) in state.iter().enumerate() {
        //println!("=== {} ===", idx);
        //for i in stateset.iter() {
            //println!("{}|{}  {:?} -> {:?}", i.start, i.dot, i.rule.name, i.rule.spec);
        //}
    //}
//}

//#[test]
//fn test4() {
    //let mut gb = GrammarBuilder::new();
    //gb.symbol(NonTerminal::new("Sum"))
      //.symbol(Terminal::new("[+]", |n: &str| { n == "+" }))
      //.symbol(Terminal::new("Number", |n: &str| {
          //n.chars().all(|c| "1234567890".contains(c))
        //}));
    //gb.rule("Sum", vec!["Sum", "[+]", "Number"]);
    //gb.rule("Sum", vec!["Number", "[+]", "Sum"]);
    //gb.rule("Sum", vec!["Number"]);

    //let g = gb.into_grammar("Sum");

    //let mut input = Lexer::from_str("3+4", "+");
    //let p = EarleyParser::new(g);

    //let state = p.parse(&mut input).unwrap();
    //for (idx, stateset) in state.iter().enumerate() {
        //println!("=== {} ===", idx);
        //for i in stateset.iter() {
            //println!("{}|{}  {:?} -> {:?}", i.start, i.dot, i.rule.name, i.rule.spec);
        //}
    //}
    //println!("==========================================");
    //let state = p.build_revtable(&state);
    //for &(start, ref rule, end) in state.iter() {
        //println!("{}|{}  {:?}", start, end, rule);
    //}
//}


 ////Sum -> Sum + Mul | Mul
 ////Mul -> Mul * Pow | Pow
 ////Pow -> Num ^ Pow | Num
 ////Num -> Number | ( Sum )


//fn build_grammar2() -> Grammar {
    //let mut gb = GrammarBuilder::new();
    //// register some symbols
    //gb.symbol(NonTerminal::new("Sum"))
      //.symbol(NonTerminal::new("Mul"))
      //.symbol(NonTerminal::new("Pow"))
      //.symbol(NonTerminal::new("Num"))
      //.symbol(Terminal::new("Number", |n: &str| {
          //n.chars().all(|c| "1234567890".contains(c))
        //}))
      //.symbol(Terminal::new("[+-]", |n: &str| {
          //n.len() == 1 && "+-".contains(n)
        //}))
      //.symbol(Terminal::new("[*/]", |n: &str| {
          //n.len() == 1 && "*/".contains(n)
        //}))
      //.symbol(Terminal::new("[^]", |n: &str| { n == "^" }))
      //.symbol(Terminal::new("(", |n: &str| { n == "(" }))
      //.symbol(Terminal::new(")", |n: &str| { n == ")" }));

    //// add grammar rules
    //gb.rule("Sum",     vec!["Sum", "[+-]", "Mul"])
      //.rule("Sum",     vec!["Mul"])
      //.rule("Mul", vec!["Mul", "[*/]", "Pow"])
      //.rule("Mul", vec!["Pow"])
      //.rule("Pow", vec!["Num", "[^]", "Pow"])
      //.rule("Pow", vec!["Num"])
      //.rule("Num",  vec!["(", "Sum", ")"])
      //.rule("Num",  vec!["Number"]);

    //gb.into_grammar("Sum")
//}

//#[test]
//fn test5() {
    //let g = build_grammar2();
    //let mut input = Lexer::from_str("1+2^3^4*5/6+7*8^9", "+*-/()^");
    //let p = EarleyParser::new(g);

    //let state = p.parse(&mut input).unwrap();
    //for (idx, stateset) in state.iter().enumerate() {
        //println!("=== {} ===", idx);
        //for i in stateset.iter() {
            //println!("{}|{}  {:?} -> {:?}", i.start, i.dot, i.rule.name, i.rule.spec);
        //}
    //}
    //println!("==========================================");
    //p.build_tree(state);
//}


