extern crate lexers;
use self::lexers::{Scanner, scan_number, scan_identifier};


#[derive(Clone,Debug,PartialEq)]
pub enum TT {
    // single char tokens
    OPAREN, CPAREN, OBRACE, CBRACE, COMMA, DOT,
    MINUS, PLUS, SEMICOLON, SLASH, STAR, DOLLAR,
    BANG, ASSIGN, NE, EQ, GT, GE, LT, LE,
    // literals
    Id(String), Str(String), Num(f64),
    // keywords
    AND, CLASS, ELSE, FALSE, FUN, FOR, IF, NIL, OR,
    PRINT, RETURN, SUPER, THIS, TRUE, VAR, WHILE, EOF,
}

#[derive(Clone,Debug)]
pub struct Token {
    pub line: usize,
    pub token: TT,
    pub lexeme: String,
}

pub struct LoxScanner {
    src: Scanner<char>,
    line: usize,
    errors: bool,
}


impl LoxScanner {
    pub fn scanner(src: String) -> Scanner<Token> {
        Scanner::new(Box::new(
            LoxScanner{src: Scanner::from_str(&src), line: 1, errors: false}))
    }

    fn tokenize(&mut self, literal: TT) -> Option<Token> {
        let lexeme = match literal {
            TT::EOF => String::new(),
            _ => self.src.extract_string()
        };
        let literal = match literal {
            TT::Str(_) => TT::Str(lexeme.trim_matches('"').to_string()),
            other => other
        };
        Some(Token{line: self.line, token: literal, lexeme: lexeme})
    }

    fn error<T: AsRef<str>>(&mut self, err: T) {
        eprintln!("LoxScanner error: {}", err.as_ref());
        self.errors = true;
    }

    fn scan_restof_string(&mut self, q: char) -> bool {
        let backtrack = self.src.pos();
        let orig_line = self.line;
        while let Some(n) = self.src.next() {
            if n == '\n' { self.line += 1; }
            if n == '\\' { self.src.next(); continue; }
            if n == q { return true; }
        }
        self.src.set_pos(backtrack);
        self.line = orig_line;
        false
    }

    fn id_or_keyword(&mut self, keyword: String) -> Option<Token> {
        let key2 = keyword.clone();
        let tok = |literal: TT| -> Option<Token> {
            Some(Token{line: self.line, token: literal, lexeme: key2})
        };
        match keyword.as_ref() {
            "and" => tok(TT::AND),
            "class" => tok(TT::CLASS),
            "else" => tok(TT::ELSE),
            "false" => tok(TT::FALSE),
            "fun" => tok(TT::FUN),
            "for" => tok(TT::FOR),
            "if" => tok(TT::IF),
            "nil" => tok(TT::NIL),
            "or" => tok(TT::OR),
            "print" => tok(TT::PRINT),
            "return" => tok(TT::RETURN),
            "super" => tok(TT::SUPER),
            "this" => tok(TT::THIS),
            "true" => tok(TT::TRUE),
            "var" => tok(TT::VAR),
            "while" => tok(TT::WHILE),
            _ => Some(Token{line: self.line,
                      token: TT::Id(keyword.clone()), lexeme: keyword})
        }
    }

    fn scan_token(&mut self) -> Option<Token> {
        let token = match self.src.next() {
            Some('(') => self.tokenize(TT::OPAREN),
            Some(')') => self.tokenize(TT::CPAREN),
            Some('{') => self.tokenize(TT::OBRACE),
            Some('}') => self.tokenize(TT::CBRACE),
            Some(',') => self.tokenize(TT::COMMA),
            Some('.') => self.tokenize(TT::DOT),
            Some('-') => self.tokenize(TT::MINUS),
            Some('+') => self.tokenize(TT::PLUS),
            Some(';') => self.tokenize(TT::SEMICOLON),
            Some('*') => self.tokenize(TT::STAR),
            Some('$') => self.tokenize(TT::DOLLAR),
            Some('!') => match self.src.accept_char('=') {
                true => self.tokenize(TT::NE),
                false => self.tokenize(TT::BANG)
            },
            Some('=') => match self.src.accept_char('=') {
                true => self.tokenize(TT::EQ),
                false => self.tokenize(TT::ASSIGN)
            },
            Some('<') => match self.src.accept_char('=') {
                true => self.tokenize(TT::LE),
                false => self.tokenize(TT::LT)
            },
            Some('>') => match self.src.accept_char('=') {
                true => self.tokenize(TT::GE),
                false => self.tokenize(TT::GT)
            },
            Some('/') => match self.src.accept_char('/') {
                true => { self.src.until_any_char("\n"); None }, // skip comment
                false => self.tokenize(TT::SLASH),
            },
            Some(' ') | Some('\t') | Some('\r') => None,
            Some('\n') => { self.line += 1; None }, // track current line
            Some('"') => match self.scan_restof_string('"') {
                true => self.tokenize(TT::Str(String::new())),
                false => { self.error("unterminated string"); None }
            },
            Some(d) if d.is_digit(10) => {
                self.src.prev(); // hacky but works
                let num = scan_number(&mut self.src).unwrap();
                use std::str::FromStr;
                Some(Token{line: self.line,
                     token: TT::Num(f64::from_str(&num).unwrap()), lexeme: num})
            },
            Some(a) if a.is_alphabetic() => {
                self.src.prev(); // hacky but works
                let id = scan_identifier(&mut self.src).unwrap();
                self.id_or_keyword(id)
            },
            Some(c) => {
                let err = format!("bad char '{}' at line {}", c, self.line);
                self.error(err);
                None
            },
            None => self.tokenize(TT::EOF)
        };
        self.src.ignore(); // ignore what we didn't harvest
        token
    }
}

impl Iterator for LoxScanner {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        loop { // consume all white space and errors
            if let Some(token) = self.scan_token() {
                match token.token {
                    TT::EOF => return None,
                    _ => return Some(token)
                }
            }
        }
    }
}
