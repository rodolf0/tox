use scanner;

pub type Matcher = scanner::Scanner<char>;

impl Matcher {
    pub fn match_id(&mut self) -> Option<String> {
        let alfa = concat!("abcdefghijklmnopqrstuvwxyz",
                           "ABCDEFGHIJKLMNOPQRSTUVWXYZ_");
        let alnum = concat!("0123456789",
                            "abcdefghijklmnopqrstuvwxyz",
                            "ABCDEFGHIJKLMNOPQRSTUVWXYZ_");
        if self.accept_chars(alfa).is_some() {
            self.skip_chars(alnum);
            return Some(self.extract_string());
        }
        None
    }

    pub fn match_exotic_int(&mut self) -> Option<String> {
        let backtrack = self.pos();
        if self.accept_chars("0").is_some() {
            if self.accept_chars("xXoObB").is_some() {
                let digits = match self.curr().unwrap() {
                    'x' | 'X' => "0123456789aAbBcCdDeEfF",
                    'o' | 'O' => "01234567",
                    'b' | 'B' => "01",
                    _ => unreachable!()
                };
                if self.skip_chars(digits) {
                    return Some(self.extract_string());
                }
            }
            self.set_pos(backtrack); // was not an ex-int
        }
        None
    }

    pub fn match_number(&mut self) -> Option<String> {
        let backtrack = self.pos();
        let digits = "0123456789";
        // optional sign
        self.accept_chars("+-");
        // require integer part
        if !self.skip_chars(digits) {
            self.set_pos(backtrack);
            return None;
        }
        // check for fractional part, else it's just an integer
        let backtrack = self.pos();
        if self.accept_chars(".").is_some() && !self.skip_chars(digits) {
            self.set_pos(backtrack);
            return Some(self.extract_string()); // integer
        }
        // check for exponent part
        let backtrack = self.pos();
        if self.accept_chars("eE").is_some() { // can't parse exponents for bases-16
            self.accept_chars("+-"); // exponent sign is optional
            if !self.skip_chars(digits) {
                self.set_pos(backtrack);
                return Some(self.extract_string()); // number without exponent
            }
        }
        self.accept_chars("i"); // accept imaginary numbers
        Some(self.extract_string())
    }
}
