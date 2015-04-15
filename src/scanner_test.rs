#![cfg(test)]

use super::scanner::Scanner;

#[test]
fn test_extremes() {
    let mut s = Scanner::from_str("just a test buffer@");
    assert_eq!(s.prev(), None);
    assert_eq!(s.next(), Some('j'));
    assert_eq!(s.prev(), None);
    while s.next() != Some('@') {}
    assert_eq!(s.curr(), Some('@'));
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
}
