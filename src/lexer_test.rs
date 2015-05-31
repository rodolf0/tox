use lexer::{Lexer, Token};

#[test]
fn test1() {
    let mut lx = Lexer::from_str("3+4*2/-(1-5)^2^3");
    let expect = [
        Token::Number(3.0),
        Token::Op("+".to_string(), 2),
        Token::Number(4.0),
        Token::Op("*".to_string(), 2),
        Token::Number(2.0),
        Token::Op("/".to_string(), 2),
        Token::Op("-".to_string(), 1),
        Token::OParen,
        Token::Number(1.0),
        Token::Op("-".to_string(), 2),
        Token::Number(5.0),
        Token::CParen,
        Token::Op("^".to_string(), 2),
        Token::Number(2.0),
        Token::Op("^".to_string(), 2),
        Token::Number(3.0),
    ];
    for exp_token in expect.iter() {
        let token = lx.next().unwrap();
        assert_eq!(*exp_token, token);
    }
    assert_eq!(lx.next(), None);
}

#[test]
fn test2() {
    let mut lx = Lexer::from_str("3.4e-2 * sin(x)/(7! % -4) * max(2, x)");
    let expect = [
        Token::Number(3.4e-2),
        Token::Op("*".to_string(), 2),
        Token::Op("sin".to_string(), 0),
        Token::OParen,
        Token::Variable("x".to_string()),
        Token::CParen,
        Token::Op("/".to_string(), 2),
        Token::OParen,
        Token::Number(7.0),
        Token::Op("!".to_string(), 1),
        Token::Op("%".to_string(), 2),
        Token::Op("-".to_string(), 1),
        Token::Number(4.0),
        Token::CParen,
        Token::Op("*".to_string(), 2),
        Token::Op("max".to_string(), 0),
        Token::OParen,
        Token::Number(2.0),
        Token::Comma,
        Token::Variable("x".to_string()),
        Token::CParen,
    ];
    for exp_token in expect.iter() {
        let token = lx.next().unwrap();
        assert_eq!(*exp_token, token);
    }
    assert_eq!(lx.next(), None);
}

#[test]
fn test3() {
    let mut lx = Lexer::from_str("sqrt(-(1-x^2) / (1 + x^2))");
    let expect = [
        Token::Op("sqrt".to_string(), 0),
        Token::OParen,
        Token::Op("-".to_string(), 1),
        Token::OParen,
        Token::Number(1.0),
        Token::Op("-".to_string(), 2),
        Token::Variable("x".to_string()),
        Token::Op("^".to_string(), 2),
        Token::Number(2.0),
        Token::CParen,
        Token::Op("/".to_string(), 2),
        Token::OParen,
        Token::Number(1.0),
        Token::Op("+".to_string(), 2),
        Token::Variable("x".to_string()),
        Token::Op("^".to_string(), 2),
        Token::Number(2.0),
        Token::CParen,
        Token::CParen,
    ];
    for exp_token in expect.iter() {
        let token = lx.next().unwrap();
        assert_eq!(*exp_token, token);
    }
    assert_eq!(lx.next(), None);
}

#[test]
fn test4() {
    let mut lx = Lexer::from_str("x---y");
    let expect = [
        Token::Variable("x".to_string()),
        Token::Op("-".to_string(), 2),
        Token::Op("-".to_string(), 1),
        Token::Op("-".to_string(), 1),
        Token::Variable("y".to_string()),
    ];
    for exp_token in expect.iter() {
        let token = lx.next().unwrap();
        assert_eq!(*exp_token, token);
    }
    assert_eq!(lx.next(), None);
}

#[test]
fn test5() {
    let mut lx = Lexer::from_str("max(0, 1, 3)");
    let expect = [
        Token::Op("max".to_string(), 0),
        Token::OParen,
        Token::Number(0.0),
        Token::Comma,
        Token::Number(1.0),
        Token::Comma,
        Token::Number(3.0),
        Token::CParen,
    ];
    for exp_token in expect.iter() {
        let token = lx.next().unwrap();
        assert_eq!(*exp_token, token);
    }
    assert_eq!(lx.next(), None);
}
