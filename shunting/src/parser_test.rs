use crate::parser::{RPNExpr, ShuntingParser};
use lexers::MathToken;

#[test]
fn test_associativity() {
    let rpn = ShuntingParser::parse_str("2^3^4");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::Number(4.0),
        MathToken::BOp("^".to_string()),
        MathToken::BOp("^".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("2*3*4");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::BOp("*".to_string()),
        MathToken::Number(4.0),
        MathToken::BOp("*".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
}

#[test]
fn test_precedence() {
    let rpn = ShuntingParser::parse_str("2+3*4");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::Number(4.0),
        MathToken::BOp("*".to_string()),
        MathToken::BOp("+".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("2*3+4");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::BOp("*".to_string()),
        MathToken::Number(4.0),
        MathToken::BOp("+".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("2+3*4^5");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::Number(4.0),
        MathToken::Number(5.0),
        MathToken::BOp("^".to_string()),
        MathToken::BOp("*".to_string()),
        MathToken::BOp("+".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("2^3+4*5");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::BOp("^".to_string()),
        MathToken::Number(4.0),
        MathToken::Number(5.0),
        MathToken::BOp("*".to_string()),
        MathToken::BOp("+".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
}

#[test]
fn test_unary_ops() {
    let rpn = ShuntingParser::parse_str("2/-1");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(1.0),
        MathToken::UOp("-".to_string()),
        MathToken::BOp("/".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("-2/1");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::UOp("-".to_string()),
        MathToken::Number(1.0),
        MathToken::BOp("/".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("-2!");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::UOp("!".to_string()),
        MathToken::UOp("-".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("-2^3");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::UOp("-".to_string()),
        MathToken::Number(3.0),
        MathToken::BOp("^".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("2^-3");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::UOp("-".to_string()),
        MathToken::BOp("^".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("2^3!");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::UOp("!".to_string()),
        MathToken::BOp("^".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("(-2)^3");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::UOp("-".to_string()),
        MathToken::Number(3.0),
        MathToken::BOp("^".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("-(1-5)");
    let expect = vec![
        MathToken::Number(1.0),
        MathToken::Number(5.0),
        MathToken::BOp("-".to_string()),
        MathToken::UOp("-".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
}

#[test]
fn test_parens() {
    let rpn = ShuntingParser::parse_str("(2+3)*4");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::BOp("+".to_string()),
        MathToken::Number(4.0),
        MathToken::BOp("*".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("2*(3*4)");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::Number(4.0),
        MathToken::BOp("*".to_string()),
        MathToken::BOp("*".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("(2^3)^4");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::BOp("^".to_string()),
        MathToken::Number(4.0),
        MathToken::BOp("^".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("((2+3)*4)^5");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::BOp("+".to_string()),
        MathToken::Number(4.0),
        MathToken::BOp("*".to_string()),
        MathToken::Number(5.0),
        MathToken::BOp("^".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
}

#[test]
fn test_mixed_ops() {
    let rpn = ShuntingParser::parse_str("3+4*2/-(1-5)^2^3");
    let expect = vec![
        MathToken::Number(3.0),
        MathToken::Number(4.0),
        MathToken::Number(2.0),
        MathToken::BOp("*".to_string()),
        MathToken::Number(1.0),
        MathToken::Number(5.0),
        MathToken::BOp("-".to_string()),
        MathToken::UOp("-".to_string()),
        MathToken::Number(2.0),
        MathToken::Number(3.0),
        MathToken::BOp("^".to_string()),
        MathToken::BOp("^".to_string()),
        MathToken::BOp("/".to_string()),
        MathToken::BOp("+".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("3.4e-2 * sin(x)/(7! % -4) * max(2, x)");
    let expect = vec![
        MathToken::Number(3.4e-2),
        MathToken::Variable("x".to_string()),
        MathToken::Function("sin".to_string(), 1),
        MathToken::BOp("*".to_string()),
        MathToken::Number(7.0),
        MathToken::UOp("!".to_string()),
        MathToken::Number(4.0),
        MathToken::UOp("-".to_string()),
        MathToken::BOp("%".to_string()),
        MathToken::BOp("/".to_string()),
        MathToken::Number(2.0),
        MathToken::Variable("x".to_string()),
        MathToken::Function("max".to_string(), 2),
        MathToken::BOp("*".to_string()),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("sqrt(-(1-x^2) / (1 + x^2))");
    let expect = vec![
        MathToken::Number(1.0),
        MathToken::Variable("x".to_string()),
        MathToken::Number(2.0),
        MathToken::BOp("^".to_string()),
        MathToken::BOp("-".to_string()),
        MathToken::UOp("-".to_string()),
        MathToken::Number(1.0),
        MathToken::Variable("x".to_string()),
        MathToken::Number(2.0),
        MathToken::BOp("^".to_string()),
        MathToken::BOp("+".to_string()),
        MathToken::BOp("/".to_string()),
        MathToken::Function("sqrt".to_string(), 1),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
}

#[test]
fn bad_parse() {
    let rpn = ShuntingParser::parse_str("1-x^2)");
    assert_eq!(rpn, Err("Missing Opening Paren".to_string()));
    let rpn = ShuntingParser::parse_str("max 4, 6, 4)");
    assert_eq!(rpn, Err("Missing Opening Paren".to_string()));
    let rpn = ShuntingParser::parse_str("sqrt(-(1-x^2)");
    assert_eq!(rpn, Err("Missing Closing Paren".to_string()));
    let rpn = ShuntingParser::parse_str("(2, 3)");
    assert_eq!(rpn, Err("Comma outside function arglist".to_string()));
    let rpn = ShuntingParser::parse_str("3 # 4");
    assert_eq!(rpn, Err("Bad token: #".to_string()));
}

#[test]
fn test_functions() {
    let rpn = ShuntingParser::parse_str("sin(pi)");
    let expect = vec![
        MathToken::Variable("pi".to_string()),
        MathToken::Function("sin".to_string(), 1),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("max(2, x)");
    let expect = vec![
        MathToken::Number(2.0),
        MathToken::Variable("x".to_string()),
        MathToken::Function("max".to_string(), 2),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
    let rpn = ShuntingParser::parse_str("sum(i , 0, gcd(24, 8))");
    let expect = vec![
        MathToken::Variable("i".to_string()),
        MathToken::Number(0.0),
        MathToken::Number(24.0),
        MathToken::Number(8.0),
        MathToken::Function("gcd".to_string(), 2),
        MathToken::Function("sum".to_string(), 3),
    ];
    assert_eq!(rpn, Ok(RPNExpr(expect)));
}
