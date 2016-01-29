use helpers;
use scanner::Scanner;

#[test]
fn test_scan_number() {
    let tests = vec![
        "987", "-543", "435i",
        "41.98", "-83.5", "-54.3i",
        "28e3", "54E+2", "54e-33", "43e0i", "3E8i",
        "-38e3", "-53e+5", "-65E-4", "-32E-4i", "-33e+2i",
        "85.365e3", "54.234E+2", "54.849e-33", "1.4e+2i", "3.14e-5i",
        "-38.657e3", "53.845e+5", "65.987E-4", "-4.4e+2i", "-6.14e-5i",
    ];
    for t in tests.iter() {
        let mut s = Scanner::from_str(t);
        assert_eq!(Some(t.to_string()), helpers::scan_number(&mut s));
    }
}

#[test]
fn test_scan_math_ops() {
    let tests = vec![
        "<", "<=", "==", ">=", ">", "(", ")", ",",
        "*", "**", "^", "!", "+", "-", "/", "%",
    ];
    for t in tests.iter() {
        let mut s = Scanner::from_str(t);
        assert_eq!(Some(t.to_string()), helpers::scan_math_op(&mut s));
    }
}

#[test]
fn test_scan_identifiers() {
    let tests = vec!["id1", "func", "anyword", "_00", "bla23"];
    for t in tests.iter() {
        let mut s = Scanner::from_str(t);
        assert_eq!(Some(t.to_string()), helpers::scan_identifier(&mut s));
    }
}

#[test]
fn test_scan_string() {
    let tests = vec![
        r"'this is a test'",
        r"'another test \' with an escaped quote'",
    ];
    for t in tests.iter() {
        let mut s = Scanner::from_str(t);
        assert_eq!(Some(t.to_string()), helpers::scan_quoted_string(&mut s, '\''));
    }
}
