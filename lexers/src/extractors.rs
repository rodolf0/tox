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

// scan numbers like [0-9]+(\.[0-9]+)?([eE][+-][0-9]+)?
// NOTE: leading minus is left to the parser instead of lexers.
pub fn number<'a, I>(s: &'a mut Scanner<I>) -> Option<&'a [char]>
where
    I: Iterator<Item = char>,
{
    let cp = s.checkpoint();
    // if it doesn't start with digits it's not a number.
    s.accept_while(char::is_ascii_digit)?;
    // maybe accept a fractional part.
    let bt = s.checkpoint();
    if s.accept('.').is_some() && s.accept_while(char::is_ascii_digit).is_none() {
        s.restore(bt); // revert leading '.'
    }
    // maybe take exponentpart.
    let bt = s.checkpoint();
    if s.accept(['e', 'E']).is_some() {
        s.accept(['+', '-']); // optional exponent sign
        if s.accept_while(char::is_ascii_digit).is_none() {
            s.restore(bt); // wasn't an exponent
        }
    }
    // maybe take imaginary numbers
    s.accept('i');
    Some(s.view_from(cp))
}

pub fn math_op<'a, I>(s: &'a mut Scanner<I>) -> Option<&'a [char]>
where
    I: Iterator<Item = char>,
{
    let cp = s.checkpoint();
    if s.accept(['>', '=', '<']).is_some() {
        s.accept('=');
        Some(s.view_from(cp))
    } else if s.accept_seq(":=".chars()).is_some() {
        Some(s.view_from(cp))
    } else if s.accept('*').is_some() {
        s.accept('*');
        Some(s.view_from(cp))
    } else {
        static OPS: &[char] = &['+', '-', '/', '%', '^', '!', '(', ')', ','];
        if s.accept(OPS).is_some() {
            Some(s.view_from(cp))
        } else {
            None
        }
    }
}

pub fn integer<'a, I>(s: &'a mut Scanner<I>) -> Option<&'a [char]>
where
    I: Iterator<Item = char>,
{
    let cp = s.checkpoint();
    s.accept('0')?; // must start with 0, eg: 0xa 0b1 0o6
    match s.accept(['x', 'o', 'b']) {
        Some('x') => {
            s.accept_while(char::is_ascii_hexdigit);
            return Some(s.view_from(cp));
        }
        Some('o') => {
            s.accept_while(|c: &char| matches!(c, '0'..='7'));
            return Some(s.view_from(cp));
        }
        Some('b') => {
            s.accept_while(|c: &char| matches!(c, '0'..='1'));
            return Some(s.view_from(cp));
        }
        _ => {
            s.restore(cp);
            None
        }
    }
}

pub fn identifier<'a, I>(s: &'a mut Scanner<I>) -> Option<&'a [char]>
where
    I: Iterator<Item = char>,
{
    let cp = s.checkpoint();
    if s.accept(|c: &char| c.is_alphabetic() || *c == '_')
        .is_some()
    {
        s.accept_while(|c: &char| c.is_alphanumeric() || *c == '_');
        Some(s.view_from(cp))
    } else {
        None
    }
}

static UNIT_PFX: &[&str] = &[
    "da", "h", "k", "M", "G", "T", "P", "E", "Z", "Y", "y", "z", "a", "f", "p", "n", "µ", "m", "c",
    "d", "", // empty: no multiplier prefix, raw unit, KEEP longest to shortest order !
];
static BARE_UNITS: &[&str] = &[
    // KEEP longest prefix first !
    "kat", "mol", "rad", "Bq", "cd", "Gy", "Hz", "lm", "lx", "Pa", "sr", "Sv", "Wb", "A", "°C", "C",
    "F", "g", "H", "J", "K", "m", "N", "s", "S", "T", "V", "W", "Ω",
];

pub fn unit<I>(s: &mut Scanner<I>) -> Option<(&'static str, &'static str)>
where
    I: Iterator<Item = char>,
{
    let cp = s.checkpoint();
    for prefix in UNIT_PFX {
        if prefix.is_empty() || s.accept_seq(prefix.chars()).is_some() {
            for unit in BARE_UNITS {
                if s.accept_seq(unit.chars()).is_some() {
                    return Some((prefix, unit));
                }
            }
        }
        s.restore(cp);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_math_ops() {
        let tests = vec![
            "<", "<=", "=", "==", ">=", ">", "(", ")", ",", "*", "**", "^", "!", "+", "-", "/",
            "%", ":=",
        ];
        for t in tests.iter() {
            let s = &mut Scanner::new(t.chars());
            let result: Option<String> = math_op(s).map(|c| c.iter().collect());
            assert_eq!(Some(t.to_string()), result);
        }
        // Negative tests
        let s = &mut Scanner::new(":".chars());
        let result: Option<String> = math_op(s).map(|c| c.iter().collect());
        assert_eq!(result, None);
    }

    #[test]
    fn scan_integers() {
        let tests = vec!["0x34", "0b10101", "0o657"];
        for t in tests.iter() {
            let s = &mut Scanner::new(t.chars());
            let result: Option<String> = integer(s).map(|c| c.iter().collect());
            assert_eq!(Some(t.to_string()), result);
        }
        let s = &mut Scanner::new("0".chars());
        let result: Option<String> = integer(s).map(|c| c.iter().collect());
        assert_eq!(result, None);
    }

    #[test]
    fn scan_identifiers() {
        let tests = vec!["id1", "fu_nc", "anyword", "_00", "bla23"];
        for t in tests.iter() {
            let s = &mut Scanner::new(t.chars());
            let result: Option<String> = identifier(s).map(|c| c.iter().collect());
            assert_eq!(Some(t.to_string()), result);
        }
    }

    #[test]
    fn scan_units() {
        for prefix in UNIT_PFX {
            for unit_base in BARE_UNITS {
                let u = format!("{}{}", prefix, unit_base);
                let s = &mut Scanner::new(u.chars());
                assert_eq!(unit(s), Some((*prefix, *unit_base)));
            }
        }
    }

    #[test]
    fn scan_number() {
        let tests = vec![
            "987",
            "41.98",
            "54E+2",
            "435i",
            "28e3",
            "54e-33",
            "43e0i",
            "3E8i",
            "85.365e3",
            "54.234E+2",
            "54.849e-33",
            "1.4e+2i",
            "3.14e-5i",
            "53.845e+5",
            "65.987E-4",
        ];
        for t in tests {
            let s = &mut Scanner::new(t.chars());
            let n: Option<String> = number(s).map(|c| c.iter().collect());
            assert_eq!(n.as_deref(), Some(t));
        }
    }

    #[test]
    fn scan_quoted() {
        #[rustfmt::skip]
        let tests = vec![
            (r#""hello""#, "\"", "\"", Some('\\'), Some(r#""hello""#)),
            (r#""he\"llo""#, "\"", "\"", Some('\\'), Some(r#""he\"llo""#)),
            (r#"'single'"#, "'", "'", Some('\\'), Some(r#"'single'"#)),
            (r#"[[bracketed]]"#, "[[", "]]", None, Some(r#"[[bracketed]]"#)),
            (r#""unterminated"#, "\"", "\"", Some('\\'), None),
        ];
        for (t, start, end, esc, expected) in tests {
            let s = &mut Scanner::new(t.chars());
            let q: Option<String> = quoted(s, start, end, esc).map(|c| c.iter().collect());
            assert_eq!(q.as_deref(), expected);
        }
    }

    #[test]
    fn scan_quoted_no_delims() {
        #[rustfmt::skip]
        let tests = vec![
            (r#""hello""#, "\"", "\"", Some('\\'), Some(r#"hello"#)),
            (r#""he\"llo""#, "\"", "\"", Some('\\'), Some(r#"he\"llo"#)),
            (r#"'single'"#, "'", "'", Some('\\'), Some(r#"single"#)),
            (r#"[[bracketed]]"#, "[[", "]]", None, Some(r#"bracketed"#)),
            (r#""unterminated"#, "\"", "\"", Some('\\'), None),
        ];
        for (t, start, end, esc, expected) in tests {
            let s = &mut Scanner::new(t.chars());
            let q: Option<String> =
                quoted_no_delims(s, start, end, esc).map(|c| c.iter().collect());
            assert_eq!(q.as_deref(), expected);
        }
    }
}
