use scanner::{Scanner, Nexter};
use std::ops::{Deref, DerefMut};

#[derive(PartialEq, Debug)]
pub enum Assoc {
    Left,
    Right,
    None
}

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    Unknown(String),
    Number(f64),
    Variable(String),
    Function(String, usize),
    Op(String, usize),
    OParen,
    CParen,
    Comma,
}

impl Token {
    pub fn precedence(&self) -> (usize, Assoc) {
        match *self {
            Token::Op(ref o, 2) if o == "+" => (2, Assoc::Left),
            Token::Op(ref o, 2) if o == "-" => (2, Assoc::Left),
            Token::Op(ref o, 2) if o == "*" => (3, Assoc::Left),
            Token::Op(ref o, 2) if o == "/" => (3, Assoc::Left),
            Token::Op(ref o, 2) if o == "%" => (3, Assoc::Left),
            Token::Op(ref o, 1) if o == "-" => (4, Assoc::Right), // unary minus
            Token::Op(ref o, 2) if o == "^" => (5, Assoc::Right),
            Token::Op(ref o, 1) if o == "!" => (6, Assoc::Left),  // factorial
            Token::Function(_, _) |
            Token::OParen                   => (1, Assoc::Left),  // grouping/fn
            _                               => (99, Assoc::None)
        }
    }

    pub fn is_op(&self, opstr: &str, arity: usize) -> bool {
        if let Token::Op(ref op, ar) = *self {
            return op == opstr && arity == ar;
        }
        false
    }
}

struct Tokenizer {
    src: Scanner<char>,
    prev: Option<Token>
}

pub struct Lexer {
    output: Scanner<Token>,
}

impl Deref for Lexer {
    type Target = Scanner<Token>;
    fn deref<'a>(&'a self) -> &'a Scanner<Token> { &self.output }
}

impl DerefMut for Lexer {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Scanner<Token> { &mut self.output }
}

impl Lexer {
    pub fn from_str(source: &str) -> Lexer {
        let tokenizer = Box::new(
            Tokenizer{src: Scanner::from_str(source), prev: None});
        Lexer{output: Scanner::new(tokenizer)}
    }
}

impl Nexter<Token> for Tokenizer {
    fn get_item(&mut self) -> Option<Token> {
        self.src.ignore_ws();
        let token = self.match_varfunc().
            or_else(|| self.match_operator()).
            or_else(|| self.match_number()).
            or_else(|| if self.src.next().is_some() {
                Some(Token::Unknown(self.src.extract_string()))
            } else {
                None
            });
        self.prev = token.clone();
        token
    }
}

impl Tokenizer {
    fn match_varfunc(&mut self) -> Option<Token> {
        let alfa = concat!("abcdefghijklmnopqrstuvwxyz",
                           "ABCDEFGHIJKLMNOPQRSTUVWXYZ_");
        let alnum = concat!("0123456789",
                            "abcdefghijklmnopqrstuvwxyz",
                            "ABCDEFGHIJKLMNOPQRSTUVWXYZ_");
        if self.src.accept_chars(alfa).is_some() {
            self.src.skip_chars(alnum);
            if self.src.peek() == Some('(') {
                return Some(Token::Function(self.src.extract_string(), 0));
            }
            return Some(Token::Variable(self.src.extract_string()));
        }
        None
    }

    fn match_number(&mut self) -> Option<Token> {
        use std::str::FromStr;
        if let Some(num) = self._match_number() {
            return match f64::from_str(&num) {
                Ok(fnum) => Some(Token::Number(fnum)),
                Err(_) => Some(Token::Unknown(num)),
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

    fn match_operator(&mut self) -> Option<Token> {
        let token = match self.src.accept_chars("+-*/%^!(),=") {
            Some('(') => Token::OParen,
            Some(')') => Token::CParen,
            Some(',') => Token::Comma,
            Some('!') => Token::Op('!'.to_string(), 1),
            Some('-') if Self::_makes_unary(&self.prev) => Token::Op('-'.to_string(), 1),
            Some(bop) => Token::Op(bop.to_string(), 2),
            None => return None
        };
        Some(token)
    }

    // when would a minus be unary? we need to know the prev token
    fn _makes_unary(prev: &Option<Token>) -> bool {
        match *prev {
            Some(Token::Number(_)) => false,
            Some(Token::Variable(_)) => false,
            Some(Token::CParen) => false,
            _ => true
        }
    }
}
