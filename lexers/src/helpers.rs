#![deny(warnings)]

use crate::scanner::Scanner;

/*
 * The caller of these function is expected to setup the scanner for a
 * clear start, ie: call scanner.ignore() to start fresh
 */

// scan numbers like -?[0-9]+(\.[0-9]+)?([eE][+-][0-9]+)?
pub fn scan_number<I: Iterator<Item=char>>(scanner: &mut Scanner<I>) -> Option<String> {
    let backtrack = scanner.pos();
    let digits = "0123456789";
    // optional sign
    scanner.accept_any_char("+-");
    // require integer part
    if !scanner.skip_all_chars(digits) {
        scanner.set_pos(backtrack);
        return None;
    }
    // check for fractional part, else it's just an integer
    let backtrack = scanner.pos();
    if scanner.accept_any_char(".").is_some() && !scanner.skip_all_chars(digits) {
        scanner.set_pos(backtrack);
        return Some(scanner.extract_string()); // integer
    }
    // check for exponent part
    let backtrack = scanner.pos();
    if scanner.accept_any_char("Ee").is_some() {
        scanner.accept_any_char("+-"); // exponent sign is optional
        if !scanner.skip_all_chars(digits) {
            scanner.set_pos(backtrack);
            return Some(scanner.extract_string()); //float
        }
    }
    scanner.accept_any_char("i"); // accept imaginary numbers
    Some(scanner.extract_string())
}

pub fn scan_math_op<I: Iterator<Item=char>>(scanner: &mut Scanner<I>) -> Option<String> {
    if scanner.accept_any_char(">=<").is_some() {
        // accept '<', '>', '=', '<=', '>=', '=='
        scanner.accept_any_char("=");
        Some(scanner.extract_string())
    } else if scanner.accept_any_char("*").is_some() {
        // accept '*', '**'
        scanner.accept_any_char("*");
        Some(scanner.extract_string())
    } else if scanner.accept_any_char("+-*/%^!(),").is_some() {
        Some(scanner.extract_string())
    } else {
        None
    }
}

// scan integers like 0x34 0b10101 0o657
pub fn scan_xob_integers<I: Iterator<Item=char>>(scanner: &mut Scanner<I>) -> Option<String> {
    let backtrack = scanner.pos();
    if scanner.accept_any_char("0").is_some() &&
        match scanner.accept_any_char("xob") {
            Some('x') => scanner.skip_all_chars("0123456789ABCDEFabcdef"),
            Some('o') => scanner.skip_all_chars("01234567"),
            Some('b') => scanner.skip_all_chars("01"),
            _ => false,
        } {
        return Some(scanner.extract_string());
    }
    scanner.set_pos(backtrack);
    None
}

// scan a quoted string like "this is \"an\" example"
pub fn scan_quoted_string<I: Iterator<Item=char>>(scanner: &mut Scanner<I>, q: char) -> Option<String> {
    let backtrack = scanner.pos();
    if ! scanner.accept_char(q) { return None; }
    while let Some(n) = scanner.next() {
        if n == '\\' { scanner.next(); continue; }
        if n == q { return Some(scanner.extract_string()); }
    }
    scanner.set_pos(backtrack);
    None
}

// scan [a-zA-Z_][a-zA-Z0-9_]+
pub fn scan_identifier<I: Iterator<Item=char>>(scanner: &mut Scanner<I>) -> Option<String> {
    let alfa = concat!("abcdefghijklmnopqrstuvwxyz",
                       "ABCDEFGHIJKLMNOPQRSTUVWXYZ_");
    let alnum = concat!("0123456789",
                        "abcdefghijklmnopqrstuvwxyz",
                        "ABCDEFGHIJKLMNOPQRSTUVWXYZ_");
    scanner.accept_any_char(alfa)?;
    scanner.skip_all_chars(alnum);
    Some(scanner.extract_string())
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

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
            let mut s = Scanner::new(t.chars());
            assert_eq!(Some(t.to_string()), scan_number(&mut s));
        }
    }

    #[test]
    fn test_scan_math_ops() {
        let tests = vec![
            "<", "<=", "==", ">=", ">", "(", ")", ",",
            "*", "**", "^", "!", "+", "-", "/", "%",
        ];
        for t in tests.iter() {
            let mut s = Scanner::new(t.chars());
            assert_eq!(Some(t.to_string()), scan_math_op(&mut s));
        }
    }

    #[test]
    fn test_scan_identifiers() {
        let tests = vec!["id1", "func", "anyword", "_00", "bla23"];
        for t in tests.iter() {
            let mut s = Scanner::new(t.chars());
            assert_eq!(Some(t.to_string()), scan_identifier(&mut s));
        }
    }

    #[test]
    fn test_scan_string() {
        let tests = vec![
            r"'this is a test'",
            r"'another test \' with an escaped quote'",
        ];
        for t in tests.iter() {
            let mut s = Scanner::new(t.chars());
            assert_eq!(Some(t.to_string()), scan_quoted_string(&mut s, '\''));
        }
    }
}
