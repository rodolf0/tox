pub fn scan_math_op(&mut self) -> Option<String> {
    const OPS: &[char] = &['+', '-', '*', '/', '%', '^', '!', '(', ')', ','];
    if self.accept(&['>', '=', '<']).is_some() {
        // accept '<', '>', '=', '<=', '>=', '=='
        self.accept('=');
        Some(self.extract_string())
    } else if self.accept(':').is_some() && self.accept('=').is_some() {
        // accept ':='. Set delayed to avoid immediate eval of rhs.
        Some(self.extract_string())
    } else if self.accept('*').is_some() {
        // accept '*', '**'
        self.accept('*');
        Some(self.extract_string())
    } else if self.accept(OPS).is_some() {
        Some(self.extract_string())
    } else {
        None
    }
}

// scan integers like 0x34 0b10101 0o657
pub fn scan_integer(&mut self) -> Option<String> {
    let backtrack = self.buffer_pos();
    if self.accept('0').is_some()
        && match self.accept(&['x', 'o', 'b']) {
            Some('x') => self.ignore(|c: &char| c.is_ascii_hexdigit()),
            Some('o') => self.ignore(|c: &char| matches!(c, '0'..='7')),
            Some('b') => self.ignore(|c: &char| matches!(c, '0'..='1')),
            _ => false,
        }
    {
        return Some(self.extract_string());
    }
    self.set_buffer_pos(backtrack);
    None
}

// scan [a-zA-Z_][a-zA-Z0-9_]+
pub fn scan_identifier(&mut self) -> Option<String> {
    self.accept(|c: &char| c.is_alphabetic() || *c == '_')?;
    self.ignore(|c: &char| c.is_alphanumeric() || *c == '_');
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
                    self.extract_string(); // commit token, reset buffer
                    return Some((prefix.to_string(), unit.to_string()))
                }
            }
        }
        self.set_buffer_pos(pfx_backtrack);
    }
    None
}
