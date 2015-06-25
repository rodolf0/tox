use lisp::{Lexer, Token, LispExpr};

pub struct Parser;

#[derive(PartialEq, Debug)]
pub enum ParseError {
    UnexpectedCParen,
    UnexpectedEOF,
    NotImplemented,
}

impl Parser {
    pub fn parse_str(expr: &str) -> Result<LispExpr, ParseError> {
        Self::parse(&mut Lexer::from_str(expr))
    }

    fn parse(lex: &mut Lexer) -> Result<LispExpr, ParseError> {
        match lex.next() {
            None                    => Err(ParseError::UnexpectedEOF),
            Some(Token::CParen)     => Err(ParseError::UnexpectedCParen),
            Some(Token::True)       => Ok(LispExpr::True),
            Some(Token::False)      => Ok(LispExpr::False),
            Some(Token::String(n))  => Ok(LispExpr::String(n)),
            Some(Token::Number(n))  => Ok(LispExpr::Number(n)),
            Some(Token::Symbol(s))  => Ok(LispExpr::Symbol(s)),
            Some(Token::OParen)     => {
                let mut list = Vec::new();
                while lex.peek() != Some(Token::CParen) { // even != None
                    match Parser::parse(lex) {
                        Err(err) => return Err(err),
                        Ok(expr) => list.push(expr),
                    }
                }
                lex.next(); // get over that CParen
                Ok(LispExpr::List(list))
            },
            //Some(Token::Quote)      => Err(ParseError::NotImplemented),
            //Some(Token::QuasiQuote) => Err(ParseError::NotImplemented),
            //Some(Token::UnQuote)    => Err(ParseError::NotImplemented),
            //Some(Token::UnQSplice)  => Err(ParseError::NotImplemented),
        }
    }
}
