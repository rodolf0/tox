#![deny(warnings)]

use crate::scanner::Scanner;

static WHITE: &[char] = &[' ', '\n', '\r', '\t'];
static DIGITS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
static HEXDIGITS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'A', 'B', 'C',
    'D', 'E', 'F',
];
static ALPHA: &[char] = &[
    '_', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
    'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
static ALNUM: &[char] = &[
    '_', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
    'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A',
    'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V', 'W', 'X', 'Y', 'Z',
];

impl<I: Iterator<Item = char>> Scanner<I> {
    pub fn extract_string(&mut self) -> String {
        self.extract().into_iter().collect()
    }

    pub fn scan_whitespace(&mut self) -> Option<String> {
        self.skip_all(WHITE);
        Some(self.extract_string())
    }

    // scan numbers like -?[0-9]+(\.[0-9]+)?([eE][+-][0-9]+)?
    pub fn scan_number(&mut self) -> Option<String> {
        let backtrack = self.buffer_pos();
        // optional sign
        self.accept_any(&['+', '-']);
        // require integer part
        if !self.skip_all(DIGITS) {
            self.set_buffer_pos(backtrack);
            return None;
        }
        // check for fractional part, else it's just an integer
        let backtrack = self.buffer_pos();
        if self.accept(&'.').is_some() && !self.skip_all(DIGITS) {
            self.set_buffer_pos(backtrack);
            return Some(self.extract_string()); // integer
        }
        // check for exponent part
        let backtrack = self.buffer_pos();
        if self.accept_any(&['e', 'E']).is_some() {
            self.accept_any(&['+', '-']); // exponent sign is optional
            if !self.skip_all(DIGITS) {
                self.set_buffer_pos(backtrack);
                return Some(self.extract_string()); //float
            }
        }
        self.accept(&'i'); // accept imaginary numbers
        Some(self.extract_string())
    }

    pub fn scan_math_op(&mut self) -> Option<String> {
        const OPS: &[char] = &['+', '-', '*', '/', '%', '^', '!', '(', ')', ','];
        if self.accept_any(&['>', '=', '<']).is_some() {
            // accept '<', '>', '=', '<=', '>=', '=='
            self.accept(&'=');
            Some(self.extract_string())
        } else if self.accept(&':').is_some() && self.accept(&'=').is_some() {
            // accept ':='. Set delayed to avoid immediate eval of rhs.
            Some(self.extract_string())
        } else if self.accept(&'*').is_some() {
            // accept '*', '**'
            self.accept(&'*');
            Some(self.extract_string())
        } else if self.accept_any(OPS).is_some() {
            Some(self.extract_string())
        } else {
            None
        }
    }

    // scan integers like 0x34 0b10101 0o657
    pub fn scan_integer(&mut self) -> Option<String> {
        let backtrack = self.buffer_pos();
        if self.accept(&'0').is_some()
            && match self.accept_any(&['x', 'o', 'b']) {
                Some('x') => self.skip_all(HEXDIGITS),
                Some('o') => self.skip_all(&HEXDIGITS[..8]),
                Some('b') => self.skip_all(&HEXDIGITS[..2]),
                _ => false,
            }
        {
            return Some(self.extract_string());
        }
        self.set_buffer_pos(backtrack);
        None
    }

    // scan a quoted string like "this is \"an\" example"
    pub fn scan_quoted_string(&mut self, q: char) -> Option<String> {
        let backtrack = self.buffer_pos();
        self.accept(&q)?;
        while let Some(n) = self.next() {
            if n == '\\' {
                self.next();
                continue;
            }
            if n == q {
                return Some(self.extract_string());
            }
        }
        self.set_buffer_pos(backtrack);
        None
    }

    // scan [a-zA-Z_][a-zA-Z0-9_]+
    pub fn scan_identifier(&mut self) -> Option<String> {
        self.accept_any(ALPHA)?;
        self.skip_all(ALNUM);
        Some(self.extract_string())
    }

    // scan an optional prefix (unit multiplier) and unit
    pub fn scan_unit(&mut self) -> Option<(String, String)> {
        static PFX: &[&str] = &[
            "da", "h", "k", "M", "G", "T", "P", "E", "Z", "Y",
            "y", "z", "a", "f", "p", "n", "µ", "m", "c", "d",
            "", // no multiplier prefix, raw unit
        ];
        // NOTE: longest prefix first for longest match (ie: 'da')
        assert_eq!(PFX[0], "da");
        static BARE_UNITS: &[&str] = &[
            "kat", "mol", "rad",
            "Bq", "cd", "Gy", "Hz", "lm", "lx", "Pa", "sr", "Sv", "Wb",
            "A", "°C", "C", "F", "g", "H", "J", "K", "m", "N", "s", "S",
            "T", "V", "W", "Ω",
        ];
        assert_eq!(BARE_UNITS[0].len(), 3);
        for prefix in PFX {
            let pfx_backtrack = self.buffer_pos();
            if self.accept_all(prefix.chars()) {
                for unit in BARE_UNITS {
                    if self.accept_all(unit.chars()) {
                        self.extract_string(); // ignore
                        return Some((prefix.to_string(), unit.to_string()))
                    }
                }
            }
            self.set_buffer_pos(pfx_backtrack);
        }
        None
    }
}
