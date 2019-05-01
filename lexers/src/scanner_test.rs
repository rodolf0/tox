use crate::scanner::Scanner;

#[test]
fn extremes() {
    let mut s = Scanner::new("just a test buffer@".chars());
    assert_eq!(s.prev(), None);
    assert_eq!(s.peek_prev(), None);
    assert_eq!(s.next(), Some('j'));
    assert_eq!(s.prev(), None);
    while s.next() != Some('@') {}
    assert_eq!(s.current(), Some('@'));
    assert_eq!(s.peek_prev(), Some('r'));
    assert_eq!(s.prev(), Some('r'));
    assert_eq!(s.prev(), Some('e'));
    assert_eq!(s.next(), Some('r'));
    assert_eq!(s.next(), Some('@'));
    assert_eq!(s.next(), None);
}

#[test]
fn extract() {
    let mut s = Scanner::new("just a test buffer@".chars());
    assert_eq!(s.extract(), Vec::new());
    for _ in 0..4 {
        assert!(s.next().is_some());
    }
    assert_eq!(s.extract().into_iter().collect::<String>(), "just");
    assert_eq!(s.peek(), Some(' '));
    assert_eq!(s.prev(), None);
    assert_eq!(s.next(), Some(' '));
    for _ in 0..6 {
        assert!(s.next().is_some());
    }
    assert_eq!(s.extract_string(), " a test");
    assert_eq!(s.next(), Some(' '));
    assert_eq!(s.peek_prev(), None);
    for _ in 0..7 {
        assert!(s.next().is_some());
    }
    assert_eq!(s.extract_string(), " buffer@");
    s.next();
    assert_eq!(s.extract(), Vec::new());
}

#[test]
fn accept() {
    static WHITE: &[char] = &[' ', '\n', '\r', '\t'];
    let mut s = Scanner::new("heey  you!".chars());
    assert!(!s.skip_all(WHITE));
    assert_eq!(s.prev(), None);
    assert_eq!(s.accept_any(&['h', 'e']), Some('h'));
    assert_eq!(s.current(), Some('h'));
    assert_eq!(s.accept_any(&['h', 'e']), Some('e'));
    assert_eq!(s.current(), Some('e'));
    assert_eq!(s.accept_any(&['h', 'y', 'e']), Some('e'));
    assert_eq!(s.accept_any(&['e']), None);
    assert_eq!(s.accept_any(&['h', 'e', 'y']), Some('y'));
    assert!(s.skip_all(WHITE));
    assert!(!s.skip_all(WHITE));
    assert_eq!(s.current(), Some(' '));
    assert_eq!(s.peek(), Some('y'));
    assert_eq!(s.next(), Some('y'));
    assert_eq!(s.next(), Some('o'));
}

#[test]
fn skips() {
    let mut s = Scanner::new("heey  you!".chars());
    assert_eq!(s.accept_any(&['h']), Some('h'));
    assert!(s.skip_all(&['h', 'e', 'y']));
    assert!(!s.skip_all(&['h', 'e', 'y']));
    assert_eq!(s.current(), Some('y'));
    assert!(s.until_any(&['!']));
    assert!(!s.until_any(&['!']));
    assert_eq!(s.accept_any(&['!']), Some('!'));
    assert_eq!(s.next(), None);
    assert_eq!(s.current(), None);
}
