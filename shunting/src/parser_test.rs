use parser::{ShuntingParser, ParseError};
use lexers::MathToken;

#[test]
fn test_parse1() {
    let rpn = ShuntingParser::parse_str("3+4*2/-(1-5)^2^3").unwrap();
    let expect = [
        MathToken::Number(3.0),
        MathToken::Number(4.0),
        MathToken::Number(2.0),
        MathToken::Op(format!("*"), 2),
        MathToken::Number(1.0),
        MathToken::Number(5.0),
        MathToken::Op(format!("-"), 2),
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::Op(format!("^"), 2),
        MathToken::Op(format!("^"), 2),
        MathToken::Op(format!("-"), 1),
        MathToken::Op(format!("/"), 2),
        MathToken::Op(format!("+"), 2),
    ];
    for (i, token) in expect.iter().enumerate() {
        assert_eq!(rpn[i], *token);
    }
}
#[test]
fn test_parse2() {
    let rpn = ShuntingParser::parse_str("3.4e-2 * sin(x)/(7! % -4) * max(2, x)").unwrap();
    let expect = [
        MathToken::Number(3.4e-2),
        MathToken::Variable(format!("x")),
        MathToken::Function(format!("sin"), 1),
        MathToken::Op(format!("*"), 2),
        MathToken::Number(7.0),
        MathToken::Op(format!("!"), 1),
        MathToken::Number(4.0),
        MathToken::Op(format!("-"), 1),
        MathToken::Op(format!("%"), 2),
        MathToken::Op(format!("/"), 2),
        MathToken::Number(2.0),
        MathToken::Variable(format!("x")),
        MathToken::Function(format!("max"), 2),
        MathToken::Op(format!("*"), 2),
    ];
    for (i, token) in expect.iter().enumerate() {
        assert_eq!(rpn[i], *token);
    }
}

#[test]
fn test_parse3() {
    let rpn = ShuntingParser::parse_str("sqrt(-(1-x^2) / (1 + x^2))").unwrap();
    let expect = [
        MathToken::Number(1.0),
        MathToken::Variable(format!("x")),
        MathToken::Number(2.0),
        MathToken::Op(format!("^"), 2),
        MathToken::Op(format!("-"), 2),
        MathToken::Op(format!("-"), 1),
        MathToken::Number(1.0),
        MathToken::Variable(format!("x")),
        MathToken::Number(2.0),
        MathToken::Op(format!("^"), 2),
        MathToken::Op(format!("+"), 2),
        MathToken::Op(format!("/"), 2),
        MathToken::Function(format!("sqrt"), 1),
    ];
    for (i, token) in expect.iter().enumerate() {
        assert_eq!(rpn[i], *token);
    }
}

#[test]
fn bad_parse() {
    let rpn = ShuntingParser::parse_str("sqrt(-(1-x^2) / (1 + x^2)");
    assert_eq!(rpn, Err(ParseError::MissingCParen));

    let rpn = ShuntingParser::parse_str("-(1-x^2) / (1 + x^2))");
    assert_eq!(rpn, Err(ParseError::MissingOParen));

    let rpn = ShuntingParser::parse_str("max 4, 6, 4)");
    assert_eq!(rpn, Err(ParseError::MissingOParen));
}

#[test]
fn check_arity() {
    use std::collections::HashMap;
    let rpn = ShuntingParser::parse_str(
        "sin(1)+(max(2, gamma(3.5), gcd(24, 8))+sum(i,0,10))").unwrap();
    let mut expect = HashMap::new();
    expect.insert("sin", 1);
    expect.insert("max", 3);
    expect.insert("gamma", 1);
    expect.insert("gcd", 2);
    expect.insert("sum", 3);

    for token in rpn.iter() {
        match *token {
            MathToken::Function(ref func, arity) => {
                let expected_arity = expect.get(&func[..]);
                assert_eq!(*expected_arity.unwrap(), arity);
            },
            _ => ()
        }
    }
}
