use std::io;
use scanner;

// Unwrap a ResultScanner's Ok value, or Panic if Error
macro_rules! uop(
    ($e: expr) => ($e.ok().unwrap())
)

pub type Matcher<R> = scanner::Scanner<R>;


impl<R: io::Reader> Matcher<R> {

    pub fn new(r: R) -> Matcher<R> {
        scanner::Scanner::new(r)
    }

    // Match an alfanumeric id which can't start with a number
    pub fn match_id(&mut self) -> Option<String> {
        let alfa = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
        let alnum = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
        if uop!(self.accept(alfa)) {
            assert!(self.skip(alnum).is_ok());
            return Some(self.extract());
        }
        None
    }

    // Match decimal numbers and base 16/8/2 integers
    pub fn match_number(&mut self) -> Option<String> {
        let mut digits = "0123456789";
        let mut base10 = true;
        let backtrack = self.pos;

        // optional sign
        assert!(self.accept("+-").is_ok());
        // check for other bases
        if uop!(self.accept("0")) {
            base10 = false;
            if uop!(self.accept("xX")) {
                digits = "0123456789aAbBcCdDeEfF";
            } else if uop!(self.accept("oO")) {
                digits = "01234567";
            } else if uop!(self.accept("bB")) {
                digits = "01";
            } else { // base 10 number starting with 0
                base10 = true;
                assert!(self.prev().is_ok());
            }
        }
        // require integer part
        if ! uop!(self.skip(digits)) {
            self.pos = backtrack;
            return None;
        }
        if ! base10 { // fraction/exponent only for base10
            return Some(self.extract());
        }
        // check for fractional part
        let backtrack = self.pos;
        if uop!(self.accept(".")) {
            // require fractional digits
            if ! uop!(self.skip(digits)) {
                self.pos = backtrack;
                return Some(self.extract()); // found an integer
            }
        }
        // check for exponent part
        let backtrack = self.pos;
        if uop!(self.accept("eE")) { // can't parse exponents for bases-16
            assert!(self.accept("+-").is_ok()); // optional exponent sign
            // require exponent digits
            if ! uop!(self.skip(digits)) {
                self.pos = backtrack;
                return Some(self.extract()); // found a number without exponent
            }
        }
        assert!(self.accept("i").is_ok()); // accept imaginary numbers
        Some(self.extract())
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
