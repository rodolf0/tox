use std::io;
use scanner;

pub type Matcher<R> = scanner::Scanner<R>;
pub type MatcherResult<T> = Result<T, scanner::ScannerErr>;


impl<R: io::Reader> Matcher<R> {

    pub fn new(r: R) -> Matcher<R> {
        scanner::Scanner::new(r)
    }

    // Match an alfanumeric id which can't start with a number
    pub fn match_id(&mut self) -> MatcherResult<bool> {
        let alfa = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
        let alnum = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
        if try!(self.accept(alfa)) {
            assert!(self.skip(alnum).is_ok());
            return Ok(true);
        }
        Ok(false)
    }

    // Match decimal numbers and base 16/8/2 integers
    pub fn match_number(&mut self) -> MatcherResult<bool> {
        let mut digits = "0123456789";
        let mut base10 = true;
        let backtrack = self.pos;

        // optional sign
        try!(self.accept("+-"));
        // check for other bases
        if try!(self.accept("0")) {
            base10 = false;
            if try!(self.accept("xX")) {
                digits = "0123456789aAbBcCdDeEfF";
            } else if try!(self.accept("oO")) {
                digits = "01234567";
            } else if try!(self.accept("bB")) {
                digits = "01";
            } else { // base 10 number starting with 0
                base10 = true;
                assert!(self.prev().is_ok());
            }
        }
        // require integer part
        if ! try!(self.skip(digits)) {
            self.pos = backtrack;
            return Ok(false);
        }
        if ! base10 { // fraction/exponent only for base10
            return Ok(true);
        }
        // check for fractional part
        let backtrack = self.pos;
        if try!(self.accept(".")) {
            // require fractional digits
            if ! try!(self.skip(digits)) {
                self.pos = backtrack;
                return Ok(true); // found an integer
            }
        }
        // check for exponent part
        let backtrack = self.pos;
        if try!(self.accept("eE")) { // can't parse exponents for bases-16
            try!(self.accept("+-")); // optional exponent sign
            // require exponent digits
            if ! try!(self.skip(digits)) {
                self.pos = backtrack;
                return Ok(true); // found a number without exponent
            }
        }
        try!(self.accept("i")); // accept imaginary numbers
        Ok(false)
    }
}



#[cfg(test)]
mod test {
    use std::io;

    #[test]
    fn test_ids() {
    }

    #[test]
    fn test_nums() {
        let nums = concat!(
            "0 10 -0 -7 -10 987654321 0.34 -2.14 2e3 -3e1 5.3E2 ",
            "2.65i 9i 8.2e-3i ",
            "1.234e2 -3.4523e+1 256E-2 354e-4 -3487.23e-1 0.001e+5 -9e-2 ",
            "0x0 0x10 -0x20 0xff 0xabcdEf ");
        let expect = [
            "0", "10", "-0", "-7", "-10", "987654321", "0.34", "-2.14", "2e3", "-3e1", "5.3E2",
            "2.65i", "9i", "8.2e-3i",
            "1.234e2", "-3.4523e+1", "256E-2", "354e-4", "-3487.23e-1", "0.001e+5", "-9e-2",
            "0x0", "0x10", "-0x20", "0xff", "0xabcdEf",
        ];

        let b = io::MemReader::new(nums.into_string().into_bytes());
        let mut m = super::Matcher::new(b);

        //loop {
            //assert!(m.skip_ws().is_ok());
        //}
    }
}
