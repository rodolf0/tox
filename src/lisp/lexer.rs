use scanner::{Scanner, Nexter};
use std::str::FromStr;
use std::ops;

#[derive(Clone, PartialEq, Debug)]
pub enum Token {
    OParen, CParen,
    //Quote(String), QuasiQuote, UnQuote, UnQSplice,
    True, False,
    Symbol(String),
    Number(f64),
    String(String),
}

struct Tokenizer {
    src: Scanner<char>,
}

impl Nexter<Token> for Tokenizer {
    fn get_item(&mut self) -> Option<Token> {
        self.src.ignore_ws();
        let token = match self.src.next() {
            Some('(')  => Token::OParen,
            Some(')')  => Token::CParen,

            // TODO parse quoted expr
            //Some('\'') => Token::Quote,
            //Some('`')  => Token::QuasiQuote,
            //Some(',')  => match self.src.peek() {
                //Some('@') => { self.src.next(); Token::UnQSplice },
                //_ => Token::UnQuote,
            //},

            Some('"')  => {
                self.src.until_chars("\"");
                if self.src.next() != Some('"') { // consume closing quote
                    self.src.ignore();
                    return None; // drop partial string, parse as unexpected EOF
                } else {
                    let token = self.src.extract();
                    Token::String(token.iter()
                                  .take(token.len() - 2)
                                  .skip(1).cloned().collect())
                }
            },
            Some(_) => {
                self.src.until_chars(" \n\r\t)");
                let token = self.src.extract_string();
                match &token[..] {
                    "#t" => Token::True,
                    "#f" => Token::False,
                    num  => match f64::from_str(num) {
                        Ok(n) => Token::Number(n),
                        Err(_)  => Token::Symbol(token.clone())
                    }
                }
            },
            None => return None
        };
        self.src.ignore();
        Some(token)
    }
}

pub struct Lexer {
    output: Scanner<Token>,
}

impl ops::Deref for Lexer {
    type Target = Scanner<Token>;
    fn deref<'a>(&'a self) -> &'a Scanner<Token> { &self.output }
}

impl ops::DerefMut for Lexer {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Scanner<Token> { &mut self.output }
}

impl Lexer {
    pub fn from_str(source: &str) -> Lexer {
        let tokenizer = Box::new(Tokenizer{src: Scanner::from_str(source)});
        Lexer{output: Scanner::new(tokenizer)}
    }
}
