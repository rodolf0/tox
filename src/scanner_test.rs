#![cfg(test)]

use scanner::Scanner;

#[test]
fn test_extremes() {
    let mut s = Scanner::from_str("just a test buffer@");
    assert_eq!(s.prev(), None);
    assert_eq!(s.peek_prev(), None);
    assert_eq!(s.next(), Some('j'));
    assert_eq!(s.prev(), None);
    while s.next() != Some('@') {}
    assert_eq!(s.curr(), Some('@'));
    assert_eq!(s.peek_prev(), Some('r'));
    assert_eq!(s.prev(), Some('r'));
    assert_eq!(s.prev(), Some('e'));
    assert_eq!(s.next(), Some('r'));
    assert_eq!(s.next(), Some('@'));
    assert_eq!(s.next(), None);
}

#[test]
fn test_extract() {
    let mut s = Scanner::from_str("just a test buffer@");
    for _ in 0..4 { assert!(s.next().is_some()); }
    assert_eq!(s.extract().iter().cloned().collect::<String>(), "just");
    assert_eq!(s.peek(), Some(' '));
    assert_eq!(s.prev(), None);
    assert_eq!(s.next(), Some(' '));
    for _ in 0..6 { assert!(s.next().is_some()); }
    assert_eq!(s.extract_string(), " a test");
    assert_eq!(s.next(), Some(' '));
    assert_eq!(s.peek_prev(), None);
}

#[test]
fn test_accept() {
    let mut s = Scanner::from_str("heey  you!");
    assert!(!s.skip_ws());
    assert_eq!(s.prev(), None);
    assert_eq!(s.accept_chars("he"), Some('h'));
    assert_eq!(s.curr(), Some('h'));
    assert_eq!(s.accept_chars("he"), Some('e'));
    assert_eq!(s.curr(), Some('e'));
    assert_eq!(s.accept_chars("hye"), Some('e'));
    assert_eq!(s.accept_chars("e"), None);
    assert_eq!(s.accept_chars("hey"), Some('y'));
    assert!(s.skip_ws());
    assert!(!s.skip_ws());
    assert_eq!(s.curr(), Some(' '));
    assert_eq!(s.peek(), Some('y'));
    assert_eq!(s.next(), Some('y'));
    assert_eq!(s.next(), Some('o'));
}

#[test]
fn test_skips() {
    let mut s = Scanner::from_str("heey  you!");
    assert_eq!(s.accept_chars("h"), Some('h'));
    assert!(s.skip_chars("hey"));
    assert!(!s.skip_chars("hey"));
    assert_eq!(s.curr(), Some('y'));
    assert!(s.until_chars("!"));
    assert!(!s.until_chars("!"));
    assert_eq!(s.accept_chars("!"), Some('!'));
    assert_eq!(s.next(), None);
    assert_eq!(s.curr(), None);
}
