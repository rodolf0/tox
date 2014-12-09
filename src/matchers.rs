use std::io;
use scanner;

pub type Matcher<R> = scanner::Scanner<R>;


impl<R: io::Reader> Matcher<R> {

    pub fn new(r: R) -> Matcher<R> {
        scanner::Scanner::new(r)
    }

    // Match an alfanumeric id which can't start with a number
    pub fn match_id(&mut self) -> Option<String> {
        let alfa = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
        let alnum = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
        if self.accept(alfa).is_some() {
            self.skip(alnum);
            return Some(self.extract());
        }
        None
    }

    // match exotic base integer
    pub fn match_exint(&mut self) -> Option<String> {
        let backtrack = self.pos;
        if self.accept("0").is_some() && self.accept("xXoObB").is_some() {
            let digits = match self.curr().unwrap() {
                'x' | 'X' => "0123456789aAbBcCdDeEfF",
                'o' | 'O' => "01234567",
                'b' | 'B' => "01",
                _ => "non-reachable!"
            };
            if self.skip(digits) {
                return Some(self.extract());
            }
            self.pos = backtrack; // was not an ex-int
        }
        None
    }

    // Match numbers with fractional part and exponent
    pub fn match_number(&mut self) -> Option<String> {
        let backtrack = self.pos;
        let digits = "0123456789";
        // optional sign
        self.accept("+-");
        // require integer part
        if !self.skip(digits) {
            self.pos = backtrack;
            return None;
        }
        // check for fractional part, else it's just an integer
        let backtrack = self.pos;
        if self.accept(".").is_some() && !self.skip(digits) {
            self.pos = backtrack;
            return Some(self.extract()); // integer
        }
        // check for exponent part
        let backtrack = self.pos;
        if self.accept("eE").is_some() { // can't parse exponents for bases-16
            self.accept("+-"); // exponent sign is optional
            if !self.skip(digits) {
                self.pos = backtrack;
                return Some(self.extract()); // number without exponent
            }
        }
        self.accept("i"); // accept imaginary numbers
        Some(self.extract())
    }
}



#[cfg(test)]
mod test {
    use std::io;

    #[test]
    fn test_nums() {
        let nums = concat!(
            "0 10 -0 -7 -10 987654321 0.34 -2.14 2e3 -3e1 5.3E2 ",
            "2.65i 9i 8.2e-3i ",
            "1.234e2 -3.4523e+1 256E-2 354e-4 -3487.23e-1 0.001e+5 -9e-2");
        let expect = [
            "0", "10", "-0", "-7", "-10", "987654321", "0.34", "-2.14", "2e3", "-3e1", "5.3E2",
            "2.65i", "9i", "8.2e-3i",
            "1.234e2", "-3.4523e+1", "256E-2", "354e-4", "-3487.23e-1", "0.001e+5", "-9e-2",
        ];

        let b = io::MemReader::new(nums.into_string().into_bytes());
        let mut m = super::Matcher::new(b);
        for exnum in expect.iter() {
            m.ignore_ws();
            let num = m.match_number();
            assert_eq!(num.unwrap().as_slice(), *exnum);
        }
        m.ignore_ws();
        assert!(m.eof());
    }

    #[test]
    fn test_exnums() {
        let nums = "0x0 0x10 0x20 0xff 0xabcdEf 0b0101";
        let expect = ["0x0", "0x10", "0x20", "0xff", "0xabcdEf", "0b0101"];
        let b = io::MemReader::new(nums.into_string().into_bytes());
        let mut m = super::Matcher::new(b);
        for exnum in expect.iter() {
            m.ignore_ws();
            let num = m.match_exint();
            assert_eq!(num.unwrap().as_slice(), *exnum);
        }
        m.ignore_ws();
        assert!(m.eof());
    }

    #[test]
    fn test_mixed() {
        let mixed = "0 0b10 _id -0 -7 word -10 987654321 0.34 test -2.14 2e3 -3e1 0x34 5.3E2";
        let expect = [("0", "number"), ("0b10", "exint"), ("_id", "id"), ("-0", "number"),
                      ("-7", "number"), ("word", "id"), ("-10", "number"), ("987654321", "number"),
                      ("0.34", "number"), ("test", "id"), ("-2.14", "number"), ("2e3", "number"),
                      ("-3e1", "number"), ("0x34", "exint"), ("5.3E2", "number")];
        let b = io::MemReader::new(mixed.into_string().into_bytes());
        let mut m = super::Matcher::new(b);

        for &(tok, typ) in expect.iter() {
            m.ignore_ws();
            match typ {
                "number" => assert_eq!(m.match_number().unwrap().as_slice(), tok),
                "exint" => assert_eq!(m.match_exint().unwrap().as_slice(), tok),
                "id" => assert_eq!(m.match_id().unwrap().as_slice(), tok),
                _ => fail!("non-reachable")
            }
        }
        assert!(m.eof());
    }
}
