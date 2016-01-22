use scanner::{Nexter, Scanner};

#[derive(PartialEq, Debug)]
pub enum TokenAssoc {
    Left,
    Right,
    None
}

#[derive(Clone, PartialEq, Debug)]
pub enum MathToken {
    Unknown(String),
    Number(f64),
    Variable(String),
    Function(String, usize),
    Op(String, usize),
    OParen,
    CParen,
    Comma,
}

impl MathToken {
    pub fn precedence(&self) -> (usize, TokenAssoc) {
        // You can play with the relation between exponentiation an unary - by
        // a. switching order in which the lexer tokenizes, if it tries
        // operators first then '-' will never be the negative part of number,
        // else if numbers are tried before operators, - can only be unary
        // for non-numeric tokens (eg: -(3)).
        // b. changing the precedence of '-' respect to '^'
        // If '-' has lower precedence then 2^-3 will fail to evaluate if the
        // '-' isn't part of the number because ^ will only find 1 operator
        match *self {
            MathToken::OParen                   => (1, TokenAssoc::Left), // keep at bottom
            MathToken::Op(ref o, 2) if o == "+" => (2, TokenAssoc::Left),
            MathToken::Op(ref o, 2) if o == "-" => (2, TokenAssoc::Left),
            MathToken::Op(ref o, 2) if o == "*" => (3, TokenAssoc::Left),
            MathToken::Op(ref o, 2) if o == "/" => (3, TokenAssoc::Left),
            MathToken::Op(ref o, 2) if o == "%" => (3, TokenAssoc::Left),
            MathToken::Op(ref o, 1) if o == "-" => (5, TokenAssoc::Right), // unary minus
            MathToken::Op(ref o, 2) if o == "^" => (5, TokenAssoc::Right),
            MathToken::Op(ref o, 1) if o == "!" => (6, TokenAssoc::Left),  // factorial
            // Func: could keep at bottom like OParen but '(' is already split
            MathToken::Function(_, _)           => (7, TokenAssoc::Left),
            _                                   => (99, TokenAssoc::None)
        }
    }

    pub fn is_op(&self, opstr: &str, arity: usize) -> bool {
        if let MathToken::Op(ref op, ar) = *self {
            return op == opstr && arity == ar;
        }
        false
    }
}

pub struct MathTokenizer {
    src: Scanner<char>,
    prev: Option<MathToken>
}

impl MathTokenizer {
    pub fn from_str(source: &str) -> Scanner<MathToken> {
        Scanner::new(Box::new(
            MathTokenizer{src: Scanner::from_str(source), prev: None}
        ))
    }
}

impl Nexter<MathToken> for MathTokenizer {
    fn get_item(&mut self) -> Option<MathToken> {
        self.src.ignore_ws();
        let token = self.match_operator().
            or_else(|| self.match_varfunc()).
            or_else(|| self.match_number()).
            or_else(|| if self.src.next().is_some() {
                Some(MathToken::Unknown(self.src.extract_string()))
            } else {
                None
            });
        self.prev = token.clone();
        token
    }
}

impl MathTokenizer {
    fn match_varfunc(&mut self) -> Option<MathToken> {
        let alfa = concat!("abcdefghijklmnopqrstuvwxyz",
                           "ABCDEFGHIJKLMNOPQRSTUVWXYZ_");
        let alnum = concat!("0123456789",
                            "abcdefghijklmnopqrstuvwxyz",
                            "ABCDEFGHIJKLMNOPQRSTUVWXYZ_");
        if self.src.accept_chars(alfa).is_some() {
            self.src.skip_chars(alnum);
            if self.src.peek() == Some('(') {
                return Some(MathToken::Function(self.src.extract_string(), 0));
            }
            return Some(MathToken::Variable(self.src.extract_string()));
        }
        None
    }

    fn match_number(&mut self) -> Option<MathToken> {
        use std::str::FromStr;
        if let Some(num) = self._match_number() {
            return match f64::from_str(&num) {
                Ok(fnum) => Some(MathToken::Number(fnum)),
                Err(_) => Some(MathToken::Unknown(num)),
            }
        }
        None
    }

    fn _match_numeric(&mut self) -> Option<String> {
        let backtrack = self.src.pos();
        if self.src.accept_chars("0").is_some() {
            if self.src.accept_chars("xob").is_some() {
                let digits = match self.src.curr().unwrap() {
                    'x' => "0123456789ABCDEF",
                    'o' => "01234567",
                    'b' => "01",
                    _ => unreachable!()
                };
                if self.src.skip_chars(digits) {
                    return Some(self.src.extract_string());
                }
            }
            self.src.set_pos(backtrack); // was not an ex-int
        }
        None
    }

    fn _match_number(&mut self) -> Option<String> {
        let backtrack = self.src.pos();
        let digits = "0123456789";
        // optional sign
        self.src.accept_chars("+-");
        // require integer part
        if !self.src.skip_chars(digits) {
            self.src.set_pos(backtrack);
            return None;
        }
        // check for fractional part, else it's just an integer
        let backtrack = self.src.pos();
        if self.src.accept_chars(".").is_some() && !self.src.skip_chars(digits) {
            self.src.set_pos(backtrack);
            return Some(self.src.extract_string()); // integer
        }
        // check for exponent part
        let backtrack = self.src.pos();
        if self.src.accept_chars("e").is_some() {
            self.src.accept_chars("+-"); // exponent sign is optional
            if !self.src.skip_chars(digits) {
                self.src.set_pos(backtrack);
                return Some(self.src.extract_string()); //float
            }
        }
        self.src.accept_chars("i"); // accept imaginary numbers
        Some(self.src.extract_string())
    }

    fn match_operator(&mut self) -> Option<MathToken> {
        let token = match self.src.accept_chars("+-*/%^!(),=") {
            Some('(') => MathToken::OParen,
            Some(')') => MathToken::CParen,
            Some(',') => MathToken::Comma,
            Some('!') => MathToken::Op('!'.to_string(), 1),
            Some('-') if Self::_makes_unary(&self.prev) => MathToken::Op('-'.to_string(), 1),
            Some(bop) => MathToken::Op(bop.to_string(), 2),
            None => return None
        };
        self.src.ignore();
        Some(token)
    }

    // when would a minus be unary? we need to know the prev token
    fn _makes_unary(prev: &Option<MathToken>) -> bool {
        match *prev {
            Some(MathToken::Number(_)) => false,
            Some(MathToken::Variable(_)) => false,
            Some(MathToken::CParen) => false,
            _ => true
        }
    }
}
