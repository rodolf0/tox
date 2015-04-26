#![cfg(test)]

use lexer::{LexComp, MathToken, MathLexer};

#[test]
fn test1() {
    let mut ml = MathLexer::lex_str("3+4*2/-(1-5)^2^3");
    let expect = [
        ("3", LexComp::Number),
        ("+", LexComp::Plus),
        ("4", LexComp::Number),
        ("*", LexComp::Times),
        ("2", LexComp::Number),
        ("/", LexComp::Divide),
        ("-", LexComp::UMinus),
        ("(", LexComp::OParen),
        ("1", LexComp::Number),
        ("-", LexComp::Minus),
        ("5", LexComp::Number),
        (")", LexComp::CParen),
        ("^", LexComp::Power),
        ("2", LexComp::Number),
        ("^", LexComp::Power),
        ("3", LexComp::Number),
    ];
    for &(ref lexeme, ref lexcomp) in expect.iter() {
        let MathToken{lexeme: lx, lexcomp: lc} = ml.next().unwrap();
        assert_eq!(lx, *lexeme);
        assert_eq!(lc, *lexcomp);
    }
    assert_eq!(ml.next(), None);
}

#[test]
fn test2() {
    let mut ml = MathLexer::lex_str("3.4e-2 * sin(x)/(7! % -4) * max(2, x)");
    let expect = [
        ("3.4e-2", LexComp::Number),
        ("*", LexComp::Times),
        ("sin", LexComp::Function),
        ("(", LexComp::OParen),
        ("x", LexComp::Variable),
        (")", LexComp::CParen),
        ("/", LexComp::Divide),
        ("(", LexComp::OParen),
        ("7", LexComp::Number),
        ("!", LexComp::Factorial),
        ("%", LexComp::Modulo),
        ("-", LexComp::UMinus),
        ("4", LexComp::Number),
        (")", LexComp::CParen),
        ("*", LexComp::Times),
        ("max", LexComp::Function),
        ("(", LexComp::OParen),
        ("2", LexComp::Number),
        (",", LexComp::Comma),
        ("x", LexComp::Variable),
        (")", LexComp::CParen),
    ];
    for &(ref lexeme, ref lexcomp) in expect.iter() {
        let MathToken{lexeme: lx, lexcomp: lc} = ml.next().unwrap();
        assert_eq!(lx, *lexeme);
        assert_eq!(lc, *lexcomp);
    }
    assert_eq!(ml.next(), None);
}

#[test]
fn test3() {
    let mut ml = MathLexer::lex_str("sqrt(-(1i-x^2) / (1 + x^2))");
    let expect = [
        ("sqrt", LexComp::Function),
        ("(", LexComp::OParen),
        ("-", LexComp::UMinus),
        ("(", LexComp::OParen),
        ("1i", LexComp::Number),
        ("-", LexComp::Minus),
        ("x", LexComp::Variable),
        ("^", LexComp::Power),
        ("2", LexComp::Number),
        (")", LexComp::CParen),
        ("/", LexComp::Divide),
        ("(", LexComp::OParen),
        ("1", LexComp::Number),
        ("+", LexComp::Plus),
        ("x", LexComp::Variable),
        ("^", LexComp::Power),
        ("2", LexComp::Number),
        (")", LexComp::CParen),
        (")", LexComp::CParen),
    ];
    for &(ref lexeme, ref lexcomp) in expect.iter() {
        let MathToken{lexeme: lx, lexcomp: lc} = ml.next().unwrap();
        assert_eq!(lx, *lexeme);
        assert_eq!(lc, *lexcomp);
    }
    assert_eq!(ml.next(), None);
}

#[test]
fn test4() {
    let mut ml = MathLexer::lex_str("x---y");
    let expect = [
        ("x", LexComp::Variable),
        ("-", LexComp::Minus),
        ("-", LexComp::UMinus),
        ("-", LexComp::UMinus),
        ("y", LexComp::Variable),
    ];
    for &(ref lexeme, ref lexcomp) in expect.iter() {
        let MathToken{lexeme: lx, lexcomp: lc} = ml.next().unwrap();
        assert_eq!(lx, *lexeme);
        assert_eq!(lc, *lexcomp);
    }
    assert_eq!(ml.next(), None);
}

#[test]
fn test5() {
    let mut ml = MathLexer::lex_str("max(0, 1, 3)");
    let expect = [
        ("max", LexComp::Function),
        ("(", LexComp::OParen),
        ("0", LexComp::Number),
        (",", LexComp::Comma),
        ("1", LexComp::Number),
        (",", LexComp::Comma),
        ("3", LexComp::Number),
        (")", LexComp::CParen),
    ];
    for &(ref lexeme, ref lexcomp) in expect.iter() {
        let MathToken{lexeme: lx, lexcomp: lc} = ml.next().unwrap();
        assert_eq!(lx, *lexeme);
        assert_eq!(lc, *lexcomp);
    }
    assert_eq!(ml.next(), None);
}
