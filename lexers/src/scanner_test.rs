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
    assert_eq!(s.accept_any_char("he"), Some('h'));
    assert_eq!(s.curr(), Some('h'));
    assert_eq!(s.accept_any_char("he"), Some('e'));
    assert_eq!(s.curr(), Some('e'));
    assert_eq!(s.accept_any_char("hye"), Some('e'));
    assert_eq!(s.accept_any_char("e"), None);
    assert_eq!(s.accept_any_char("hey"), Some('y'));
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
    assert_eq!(s.accept_any_char("h"), Some('h'));
    assert!(s.skip_all_chars("hey"));
    assert!(!s.skip_all_chars("hey"));
    assert_eq!(s.curr(), Some('y'));
    assert!(s.until_any_char("!"));
    assert!(!s.until_any_char("!"));
    assert_eq!(s.accept_any_char("!"), Some('!'));
    assert_eq!(s.next(), None);
    assert_eq!(s.curr(), None);
}
