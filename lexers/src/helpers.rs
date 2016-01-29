use scanner::Scanner;

/*
 * The caller of these function is expected to setup the scanner for a
 * clear start, ie: call scanner.ignore() to start fresh
 */

// scan numbers like -?[0-9]+(\.[0-9]+)?([eE][+-][0-9]+)?
pub fn scan_number(scanner: &mut Scanner<char>) -> Option<String> {
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

pub fn scan_math_op(scanner: &mut Scanner<char>) -> Option<String> {
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
pub fn scan_xob_integers(scanner: &mut Scanner<char>) -> Option<String> {
    let backtrack = scanner.pos();
    if scanner.accept_any_char("0").is_some() {
        if match scanner.accept_any_char("xob") {
            Some('x') => scanner.skip_all_chars("0123456789ABCDEFabcdef"),
            Some('o') => scanner.skip_all_chars("01234567"),
            Some('b') => scanner.skip_all_chars("01"),
            _ => false,
        } {
            return Some(scanner.extract_string());
        }
    }
    scanner.set_pos(backtrack);
    None
}

// scan a quoted string like "this is \"an\" example"
pub fn scan_quoted_string(scanner: &mut Scanner<char>, q: char) -> Option<String> {
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
pub fn scan_identifier(scanner: &mut Scanner<char>) -> Option<String> {
    let alfa = concat!("abcdefghijklmnopqrstuvwxyz",
                       "ABCDEFGHIJKLMNOPQRSTUVWXYZ_");
    let alnum = concat!("0123456789",
                        "abcdefghijklmnopqrstuvwxyz",
                        "ABCDEFGHIJKLMNOPQRSTUVWXYZ_");
    if scanner.accept_any_char(alfa).is_none() { return None; }
    scanner.skip_all_chars(alnum);
    Some(scanner.extract_string())
}
