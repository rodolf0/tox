use mathscanner::MathScanner;
use scanner::{Scanner, Nexter};

#[derive(Clone, PartialEq, Debug)]
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
    Assign,
}

#[derive(Clone, PartialEq, Debug)]
pub struct MathToken {
    pub lexeme: String,
    pub lexcomp: LexComp
}

impl MathToken {
    pub fn is(&self, lexcomp: &LexComp) -> bool {
        self.lexcomp == *lexcomp
    }
}

pub type MathLexer = Scanner<MathToken>;

struct TokenReader {
    src: MathScanner,
    prev: Option<MathToken>
}

impl MathLexer {
    pub fn lex_str(input: &str) -> MathLexer {
        Self::new(Box::new(
            TokenReader{src: MathScanner::from_str(input),
                        prev: None}))
    }
}

impl TokenReader {
    fn lex_varfunc(&mut self) -> Option<MathToken> {
        match self.src.scan_id() {
            Some(name) => if self.src.peek() == Some('(') {
                Some(MathToken{lexeme: name,
                               lexcomp: LexComp::Function})
            } else {
                Some(MathToken{lexeme: name,
                               lexcomp: LexComp::Variable})
            },
            _ => None
        }
    }

    // when would a minus be unary? we need to know the prev token
    fn makes_unary_minus(prev: &Option<MathToken>) -> bool {
        if let Some(ref mtok) = *prev {
            match mtok.lexcomp {
                LexComp::Number => false,
                LexComp::Variable => false,
                LexComp::CParen => false,
                _ => true
            }
        } else {
            true // if prev is None '-' is at begining of buffer
        }
    }

    fn lex_operator(&mut self) -> Option<MathToken> {
        let tok = match self.src.accept_chars("+-*/%^!(),=") {
            None => return None,
            Some('+') => MathToken{lexeme: "+".to_string(), lexcomp: LexComp::Plus},
            Some('-') => if TokenReader::makes_unary_minus(&self.prev) {
                MathToken{lexeme: "-".to_string(), lexcomp: LexComp::UMinus}
            } else {
                MathToken{lexeme: "-".to_string(), lexcomp: LexComp::Minus}
            },
            Some('*') => MathToken{lexeme: "*".to_string(), lexcomp: LexComp::Times},
            Some('/') => MathToken{lexeme: "/".to_string(), lexcomp: LexComp::Divide},
            Some('%') => MathToken{lexeme: "%".to_string(), lexcomp: LexComp::Modulo},
            Some('^') => MathToken{lexeme: "^".to_string(), lexcomp: LexComp::Power},
            Some('!') => MathToken{lexeme: "!".to_string(), lexcomp: LexComp::Factorial},
            Some('(') => MathToken{lexeme: "(".to_string(), lexcomp: LexComp::OParen},
            Some(')') => MathToken{lexeme: ")".to_string(), lexcomp: LexComp::CParen},
            Some(',') => MathToken{lexeme: ",".to_string(), lexcomp: LexComp::Comma},
            Some('=') => MathToken{lexeme: "=".to_string(), lexcomp: LexComp::Assign},
            _ => unreachable!()
        };
        Some(tok)
    }

    fn lex_number(&mut self) -> Option<MathToken> {
        if let Some(number) = self.src.scan_exotic_int() {
            Some(MathToken{lexeme: number,
                           lexcomp: LexComp::Number})
        } else if let Some(number) = self.src.scan_number() {
            Some(MathToken{lexeme: number,
                           lexcomp: LexComp::Number})
        } else {
            None
        }
    }
}

impl Nexter<MathToken> for TokenReader {
    fn get_item(&mut self) -> Option<MathToken> {
        self.src.ignore_ws();
        let mathtok = self.lex_varfunc().
            or_else(|| self.lex_operator()).
            or_else(|| self.lex_number()).
            or_else(|| if let Some(_) = self.src.next() {
                Some(MathToken{lexeme: self.src.extract_string(),
                               lexcomp: LexComp::Unknown})
            } else {
                None
            });
        self.prev = mathtok.clone();
        mathtok
    }
}
