use crate::scanner::Scanner;

pub fn quoted<'a, I: Iterator<Item = char>>(
    s: &'a mut Scanner<I>,
    start: &str,
    end: &str,
    escape: Option<char>,
) -> Option<&'a [char]> {
    let cp = s.checkpoint();
    if s.accept_seq(start.chars()).is_some() {
        // consume all escaped content until end delimiter.
        while let Some(c) = s.peek() {
            if Some(*c) == escape {
                s.advance(); // consume escape char
            } else if s.accept_seq(end.chars()).is_some() {
                return Some(s.view_from(cp));
            }
            s.advance();
        }
    }
    s.restore(cp);
    None
}

pub fn quoted_no_delims<'a, I: Iterator<Item = char>>(
    s: &'a mut Scanner<I>,
    start: &str,
    end: &str,
    escape: Option<char>,
) -> Option<&'a [char]> {
    let extract = quoted(s, start, end, escape)?;
    Some(&extract[start.len()..extract.len() - end.len()])
}

// scan numbers like -?[0-9]+(\.[0-9]+)?([eE][+-][0-9]+)?
// pub fn number<'a, I: Iterator<Item = char>>(s: &'a mut Scanner<I>) -> Option<&'a [char]> {
//     let backtrack = s.checkpoint();
//
//     let integer_part = s.accept_while(char::is_ascii_digit);
//
//     // require integer part
//     if !s.ignore(|c: &char| c.is_ascii_digit()) {
//         s.restore(backtrack);
//         return None;
//     }
//     // check for fractional part, else it's just an integer
//     let backtrack = s.checkpoint();
//     if s.accept('.').is_some() && !s.ignore(|c: &char| c.is_ascii_digit()) {
//         s.restore(backtrack);
//         return Some(s.extract_string()); // integer
//     }
//     // check for exponent part
//     let backtrack = s.checkpoint();
//     if s.accept(&['e', 'E']).is_some() {
//         s.accept(&['+', '-']); // exponent sign is optional
//         if !s.ignore(|c: &char| c.is_ascii_digit()) {
//             s.restore(backtrack);
//             return Some(s.extract_string()); //float
//         }
//     }
//     s.accept('i'); // accept imaginary numbers
//     Some(s.extract_string())
// }
