#![cfg(test)]
use lexer::{LexComp, MathToken};
use shunting::{Token, ParseError};
use shunting::MathParser;

#[test]
fn test_parse1() {
    let rpn = MathParser::parse("3+4*2/-(1-5)^2^3").ok().unwrap();
    let expect = [
        ("3", LexComp::Number),
        ("4", LexComp::Number),
        ("2", LexComp::Number),
        ("*", LexComp::Times),
        ("1", LexComp::Number),
        ("5", LexComp::Number),
        ("-", LexComp::Minus),
        ("2", LexComp::Number),
        ("3", LexComp::Number),
        ("^", LexComp::Power),
        ("^", LexComp::Power),
        ("-", LexComp::UMinus),
        ("/", LexComp::Divide),
        ("+", LexComp::Plus),
    ];
    for (i, &(ref lexeme, ref lexcomp)) in expect.iter().enumerate() {
        let Token{lxtoken: MathToken{lexeme: ref lx, lexcomp: ref lc }, arity: _} = rpn[i];
        assert_eq!(*lexcomp, *lc);
        assert_eq!(*lexeme, *lx);
    }
}

#[test]
fn test_parse2() {
    let rpn = MathParser::parse("3.4e-2 * sin(x)/(7! % -4) * max(2, x)").ok().unwrap();
    let expect = [
        ("3.4e-2", LexComp::Number),
        ("x", LexComp::Variable),
        ("sin", LexComp::Function),
        ("*", LexComp::Times),
        ("7", LexComp::Number),
        ("!", LexComp::Factorial),
        ("4", LexComp::Number),
        ("-", LexComp::UMinus),
        ("%", LexComp::Modulo),
        ("/", LexComp::Divide),
        ("2", LexComp::Number),
        ("x", LexComp::Variable),
        ("max", LexComp::Function),
        ("*", LexComp::Times),
    ];
    for (i, &(ref lexeme, ref lexcomp)) in expect.iter().enumerate() {
        let Token{lxtoken: MathToken{lexeme: ref lx, lexcomp: ref lc }, arity: _} = rpn[i];
        assert_eq!(*lexcomp, *lc);
        assert_eq!(*lexeme, *lx);
    }
}

#[test]
fn test_parse3() {
    let rpn = MathParser::parse("sqrt(-(1i-x^2) / (1 + x^2))").ok().unwrap();
    let expect = [
        ("1i", LexComp::Number),
        ("x", LexComp::Variable),
        ("2", LexComp::Number),
        ("^", LexComp::Power),
        ("-", LexComp::Minus),
        ("-", LexComp::UMinus),
        ("1", LexComp::Number),
        ("x", LexComp::Variable),
        ("2", LexComp::Number),
        ("^", LexComp::Power),
        ("+", LexComp::Plus),
        ("/", LexComp::Divide),
        ("sqrt", LexComp::Function),
    ];
    for (i, &(ref lexeme, ref lexcomp)) in expect.iter().enumerate() {
        let Token{lxtoken: MathToken{lexeme: ref lx, lexcomp: ref lc }, arity: _} = rpn[i];
        assert_eq!(*lexcomp, *lc);
        assert_eq!(*lexeme, *lx);
    }
}

#[test]
fn bad_parse() {
    let rpn = MathParser::parse("sqrt(-(1i-x^2) / (1 + x^2)");
    assert_eq!(rpn, Err(ParseError::MissingCParen));

    let rpn = MathParser::parse("-(1i-x^2) / (1 + x^2))");
    assert_eq!(rpn, Err(ParseError::MissingOParen));

    let rpn = MathParser::parse("max 4, 6, 4)");
    assert_eq!(rpn, Err(ParseError::MisplacedComma));
}

#[test]
fn check_arity() {
    use std::collections::HashMap;
    let rpn = MathParser::parse("sin(1)+(max(2, gamma(3.5), gcd(24, 8))+sum(i,0,10))");
    let mut rpn = rpn.ok().unwrap();
    let mut expect = HashMap::new();
    expect.insert("sin", 1);
    expect.insert("max", 3);
    expect.insert("gamma", 1);
    expect.insert("gcd", 2);
    expect.insert("sum", 3);

    while let Some(tok) = rpn.pop() {
        if tok.lxtoken.lexcomp == LexComp::Function {
            let expected_arity = expect.get(&tok.lxtoken.lexeme[..]);
            assert_eq!(*expected_arity.unwrap(), tok.arity);
        }
    }
}
