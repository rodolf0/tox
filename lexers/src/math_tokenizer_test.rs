use math_tokenizer::{MathToken, MathTokenizer};

#[test]
fn test1() {
    let mut lx = MathTokenizer::from_str("3+4*2/-(1-5)^2^3");
    let expect = [
        MathToken::Number(3.0),
        MathToken::Op(format!("+"), 2),
        MathToken::Number(4.0),
        MathToken::Op(format!("*"), 2),
        MathToken::Number(2.0),
        MathToken::Op(format!("/"), 2),
        MathToken::Op(format!("-"), 1),
        MathToken::OParen,
        MathToken::Number(1.0),
        MathToken::Op(format!("-"), 2),
        MathToken::Number(5.0),
        MathToken::CParen,
        MathToken::Op(format!("^"), 2),
        MathToken::Number(2.0),
        MathToken::Op(format!("^"), 2),
        MathToken::Number(3.0),
    ];
    for exp_token in expect.iter() {
        let token = lx.next().unwrap();
        assert_eq!(*exp_token, token);
    }
    assert_eq!(lx.next(), None);
}

#[test]
fn test2() {
    let mut lx = MathTokenizer::from_str("3.4e-2 * sin(x)/(7! % -4) * max(2, x)");
    let expect = [
        MathToken::Number(3.4e-2),
        MathToken::Op(format!("*"), 2),
        MathToken::Function(format!("sin"), 0),
        MathToken::OParen,
        MathToken::Variable(format!("x")),
        MathToken::CParen,
        MathToken::Op(format!("/"), 2),
        MathToken::OParen,
        MathToken::Number(7.0),
        MathToken::Op(format!("!"), 1),
        MathToken::Op(format!("%"), 2),
        MathToken::Op(format!("-"), 1),
        MathToken::Number(4.0),
        MathToken::CParen,
        MathToken::Op(format!("*"), 2),
        MathToken::Function(format!("max"), 0),
        MathToken::OParen,
        MathToken::Number(2.0),
        MathToken::Comma,
        MathToken::Variable(format!("x")),
        MathToken::CParen,
    ];
    for exp_token in expect.iter() {
        let token = lx.next().unwrap();
        assert_eq!(*exp_token, token);
    }
    assert_eq!(lx.next(), None);
}

#[test]
fn test3() {
    let mut lx = MathTokenizer::from_str("sqrt(-(1-x^2) / (1 + x^2))");
    let expect = [
        MathToken::Function(format!("sqrt"), 0),
        MathToken::OParen,
        MathToken::Op(format!("-"), 1),
        MathToken::OParen,
        MathToken::Number(1.0),
        MathToken::Op(format!("-"), 2),
        MathToken::Variable(format!("x")),
        MathToken::Op(format!("^"), 2),
        MathToken::Number(2.0),
        MathToken::CParen,
        MathToken::Op(format!("/"), 2),
        MathToken::OParen,
        MathToken::Number(1.0),
        MathToken::Op(format!("+"), 2),
        MathToken::Variable(format!("x")),
        MathToken::Op(format!("^"), 2),
        MathToken::Number(2.0),
        MathToken::CParen,
        MathToken::CParen,
    ];
    for exp_token in expect.iter() {
        let token = lx.next().unwrap();
        assert_eq!(*exp_token, token);
    }
    assert_eq!(lx.next(), None);
}

#[test]
fn test4() {
    let mut lx = MathTokenizer::from_str("x---y");
    let expect = [
        MathToken::Variable(format!("x")),
        MathToken::Op(format!("-"), 2),
        MathToken::Op(format!("-"), 1),
        MathToken::Op(format!("-"), 1),
        MathToken::Variable(format!("y")),
    ];
    for exp_token in expect.iter() {
        let token = lx.next().unwrap();
        assert_eq!(*exp_token, token);
    }
    assert_eq!(lx.next(), None);
}

#[test]
fn test5() {
    let mut lx = MathTokenizer::from_str("max(0, 1, 3)");
    let expect = [
        MathToken::Function(format!("max"), 0),
        MathToken::OParen,
        MathToken::Number(0.0),
        MathToken::Comma,
        MathToken::Number(1.0),
        MathToken::Comma,
        MathToken::Number(3.0),
        MathToken::CParen,
    ];
    for exp_token in expect.iter() {
        let token = lx.next().unwrap();
        assert_eq!(*exp_token, token);
    }
    assert_eq!(lx.next(), None);
}
