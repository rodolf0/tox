extern crate num;
use shunting::ShuntingParser;
use shunting::MathContext;

macro_rules! fuzzy_eq {
    ($lhs:expr, $rhs:expr) => { num::abs($lhs - $rhs) < 1.0e-10 }
}

#[test]
fn test_eval1() {
    let expr = ShuntingParser::parse_str("3+4*2/-(1-5)^2^3").unwrap();
    fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 2.99987792969);
}

#[test]
fn test_eval2() {
    let expr = ShuntingParser::parse_str("3.4e-2 * sin(pi/3)/(540 % -4) * max(2, -7)").unwrap();
    fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), -0.000286429);
}

#[test]
fn test_eval3() {
    let expr = ShuntingParser::parse_str("(-(1-9^2) / (1 + 6^2))^0.5").unwrap();
    fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 1.470429244187615496759);
}

#[test]
fn test_eval4() {
    let expr = ShuntingParser::parse_str("sin(0.345)^2 + cos(0.345)^2").unwrap();
    fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 1.0);
}

#[test]
fn test_eval5() {
    let expr = ShuntingParser::parse_str("sin(e)/cos(e)").unwrap();
    fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), -0.4505495340698074);
}

#[test]
fn test_eval6() {
    let expr = ShuntingParser::parse_str("(3+4)*3").unwrap();
    fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 21.0);
}

#[test]
fn test_eval7() {
    let expr = ShuntingParser::parse_str("(3+4)*3").unwrap();
    fuzzy_eq!(MathContext::new().eval(&expr).unwrap(), 21.0);
}
