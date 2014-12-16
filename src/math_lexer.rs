use std::io;
use matchers;

#[deriving(PartialEq, Show, Clone)]
pub enum LexComp {
    Unknown,
    Number,
    Variable,
    Function,
    OParen,
    CParen,
    Comma,
    Plus,
    Minus,
    Times,
    Divide,
    Modulo,
    Power,
    UMinus,
    Factorial,
}


#[deriving(PartialEq, Show, Clone)]
pub struct MathToken {
    pub lexeme: String,
    pub lexcomp: LexComp
}

pub struct MathLexer<R: io::Reader> {
    m: matchers::Matcher<R>,
    buf: Vec<MathToken>,
    pos: int
}


// check if a '-' minus is unary based on the preceding token
fn makes_unary_minus(prev: &MathToken) -> bool {
    match prev.lexcomp {
        LexComp::Number | LexComp::Variable | LexComp::CParen => false,
        _ => true
    }
}


impl MathLexer<io::MemReader> {
    // Build a MathLexer reading from a string
    pub fn from_str(e: &str) -> MathLexer<io::MemReader> {
        let b = io::MemReader::new(e.as_bytes().to_vec());
        MathLexer::new(b)
    }
}


impl<R: io::Reader> MathLexer<R> {
    // Build a MathLexer
    pub fn new(r: R) -> MathLexer<R> {
        MathLexer{
            m: matchers::Matcher::new(r),
            buf: Vec::new(),
            pos: -1
        }
    }

    // read a token from the underlying scanner, classify it and add it to our buffer
    fn read_token(&mut self) -> Option<MathToken> {
        self.m.ignore_ws();
        // try variables / function names
        if let Some(name) = self.m.match_id() {
            self.m.ignore_ws();
            if self.m.peek() == Some('(') {
                return Some(MathToken{lexeme: name, lexcomp: LexComp::Function})
            } else {
                return Some(MathToken{lexeme: name, lexcomp: LexComp::Variable})
            }
        }
        // try operators
        if let Some(op) = self.m.accept("+-*/%^!(),") {
            match op {
                '+' => return Some(MathToken{lexeme: String::from_str("+"), lexcomp: LexComp::Plus}),
                '-' => {
                    let prevpos = (self.pos - 1) as uint;
                    if self.pos < 1 || makes_unary_minus(&self.buf[prevpos]) {
                        return Some(MathToken{lexeme: String::from_str("-"), lexcomp: LexComp::UMinus});
                    }
                    return Some(MathToken{lexeme: String::from_str("-"), lexcomp: LexComp::Minus});
                },
                '*' => return Some(MathToken{lexeme: String::from_str("*"), lexcomp: LexComp::Times}),
                '/' => return Some(MathToken{lexeme: String::from_str("/"), lexcomp: LexComp::Divide}),
                '%' => return Some(MathToken{lexeme: String::from_str("%"), lexcomp: LexComp::Modulo}),
                '^' => return Some(MathToken{lexeme: String::from_str("^"), lexcomp: LexComp::Power}),
                '!' => return Some(MathToken{lexeme: String::from_str("!"), lexcomp: LexComp::Factorial}),
                '(' => return Some(MathToken{lexeme: String::from_str("("), lexcomp: LexComp::OParen}),
                ')' => return Some(MathToken{lexeme: String::from_str(")"), lexcomp: LexComp::CParen}),
                ',' => return Some(MathToken{lexeme: String::from_str(","), lexcomp: LexComp::Comma}),
                _ => return Some(MathToken{lexeme: String::from_char(1, op), lexcomp: LexComp::Unknown})
            }
        }
        // try exotic integers
        if let Some(exint) = self.m.match_exint() {
            return Some(MathToken{lexeme: exint, lexcomp: LexComp::Number});
        }
        // try numbers
        if let Some(number) = self.m.match_number() {
            return Some(MathToken{lexeme: number, lexcomp: LexComp::Number});
        }
        // unkown lex-component
        if self.m.peek().is_some() {
            assert!(self.m.until_ws());
            return Some(MathToken{lexeme: self.m.extract(), lexcomp: LexComp::Unknown});
        }
        // if didn't match even the unknown arm we must be at EOF
        assert!(self.m.eof());
        None
    }

    // get the next token
    pub fn next(&mut self) -> Option<MathToken> {
        self.pos += 1;
        let pos = self.pos as uint;
        // reached end of buffer, fetch more tokens
        if pos >= self.buf.len() {
            match self.read_token() {
                None => {
                    self.pos = self.buf.len() as int;
                    return None;
                },
                Some(tok) => {
                    self.buf.push(tok);
                }
            }
        }
        self.curr()
    }

    // get the token the lexer is on
    pub fn curr(&self) -> Option<MathToken> {
        if self.pos < 0 {
            return None;
        }
        let pos = self.pos as uint;
        if pos >= self.buf.len() {
            return None;
        }
        Some(self.buf[pos].clone())
    }
}



#[cfg(test)]
mod test {
    use super::{MathLexer, LexComp, MathToken};

    #[test]
    fn test1() {
        let mut ml = MathLexer::from_str("3+4*2/-(1-5)^2^3");
        let expect = [
            ("3", LexComp::Number),
            ("+", LexComp::Plus),
            ("4", LexComp::Number),
            ("*", LexComp::Times),
            ("2", LexComp::Number),
            ("/", LexComp::Divide),
            ("-", LexComp::UMinus),
            ("(", LexComp::OParen),
            ("1", LexComp::Number),
            ("-", LexComp::Minus),
            ("5", LexComp::Number),
            (")", LexComp::CParen),
            ("^", LexComp::Power),
            ("2", LexComp::Number),
            ("^", LexComp::Power),
            ("3", LexComp::Number),
        ];
        for &(lexeme, lexcomp) in expect.iter() {
            let MathToken{lexeme: lx, lexcomp: lc} = ml.next().unwrap();
            assert_eq!(lx, lexeme);
            assert_eq!(lc, lexcomp);
        }
        assert_eq!(ml.next(), None);
    }

    #[test]
    fn test2() {
        let mut ml = MathLexer::from_str("3.4e-2 * sin(x)/(7! % -4) * max(2, x)");
        let expect = [
            ("3.4e-2", LexComp::Number),
            ("*", LexComp::Times),
            ("sin", LexComp::Function),
            ("(", LexComp::OParen),
            ("x", LexComp::Variable),
            (")", LexComp::CParen),
            ("/", LexComp::Divide),
            ("(", LexComp::OParen),
            ("7", LexComp::Number),
            ("!", LexComp::Factorial),
            ("%", LexComp::Modulo),
            ("-", LexComp::UMinus),
            ("4", LexComp::Number),
            (")", LexComp::CParen),
            ("*", LexComp::Times),
            ("max", LexComp::Function),
            ("(", LexComp::OParen),
            ("2", LexComp::Number),
            (",", LexComp::Comma),
            ("x", LexComp::Variable),
            (")", LexComp::CParen),
        ];
        for &(lexeme, lexcomp) in expect.iter() {
            let MathToken{lexeme: lx, lexcomp: lc} = ml.next().unwrap();
            assert_eq!(lx, lexeme);
            assert_eq!(lc, lexcomp);
        }
        assert_eq!(ml.next(), None);
    }

    #[test]
    fn test3() {
        let mut ml = MathLexer::from_str("sqrt(-(1i-x^2) / (1 + x^2))");
        let expect = [
            ("sqrt", LexComp::Function),
            ("(", LexComp::OParen),
            ("-", LexComp::UMinus),
            ("(", LexComp::OParen),
            ("1i", LexComp::Number),
            ("-", LexComp::Minus),
            ("x", LexComp::Variable),
            ("^", LexComp::Power),
            ("2", LexComp::Number),
            (")", LexComp::CParen),
            ("/", LexComp::Divide),
            ("(", LexComp::OParen),
            ("1", LexComp::Number),
            ("+", LexComp::Plus),
            ("x", LexComp::Variable),
            ("^", LexComp::Power),
            ("2", LexComp::Number),
            (")", LexComp::CParen),
            (")", LexComp::CParen),
        ];
        for &(lexeme, lexcomp) in expect.iter() {
            let MathToken{lexeme: lx, lexcomp: lc} = ml.next().unwrap();
            assert_eq!(lx, lexeme);
            assert_eq!(lc, lexcomp);
        }
        assert_eq!(ml.next(), None);
    }

    #[test]
    fn test4() {
        let mut ml = MathLexer::from_str("x---y");
        let expect = [
            ("x", LexComp::Variable),
            ("-", LexComp::Minus),
            ("-", LexComp::UMinus),
            ("-", LexComp::UMinus),
            ("y", LexComp::Variable),
        ];
        for &(lexeme, lexcomp) in expect.iter() {
            let MathToken{lexeme: lx, lexcomp: lc} = ml.next().unwrap();
            assert_eq!(lx, lexeme);
            assert_eq!(lc, lexcomp);
        }
        assert_eq!(ml.next(), None);
    }

    #[test]
    fn test5() {
        let mut ml = MathLexer::from_str("max(0, 1, 3)");
        let expect = [
            ("max", LexComp::Function),
            ("(", LexComp::OParen),
            ("0", LexComp::Number),
            (",", LexComp::Comma),
            ("1", LexComp::Number),
            (",", LexComp::Comma),
            ("3", LexComp::Number),
            (")", LexComp::CParen),
        ];
        for &(lexeme, lexcomp) in expect.iter() {
            let MathToken{lexeme: lx, lexcomp: lc} = ml.next().unwrap();
            assert_eq!(lx, lexeme);
            assert_eq!(lc, lexcomp);
        }
        assert_eq!(ml.next(), None);
    }
}
