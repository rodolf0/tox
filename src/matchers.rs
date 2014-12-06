use std::io;
use scanner;

pub type Matcher<R> = scanner::Scanner<R>;
pub type MatcherResult<T> = Result<T, scanner::ScannerErr>;


impl<R: io::Reader> Matcher<R> {

    pub fn new(r: R) -> Matcher<R> {
        scanner::Scanner::new(r)
    }

    pub fn match_id(&mut self) -> MatcherResult<bool> {
        let alfa = "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let alnum = "0123456789_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        if try!(self.accept(alfa)) {
            assert!(self.skip(alnum).is_ok());
            return Ok(true);
        }
        Ok(false)
    }
}
