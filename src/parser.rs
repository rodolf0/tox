use lexer::{Lexer, Token, Assoc};
use std::ops::Deref;

#[derive(PartialEq, Debug)]
pub enum ParseError {
    MissingOParen,
    MissingCParen,
    NonAssoc,
    BadToken(String),
}

#[derive(PartialEq, Debug)]
pub struct RPNExpr {
    expr: Vec<Token>
}

impl Deref for RPNExpr {
    type Target = Vec<Token>;
    fn deref<'a>(&'a self) -> &'a Vec<Token> { &self.expr }
}

pub struct ShuntingParser;

impl ShuntingParser {
    pub fn parse_str(expr: &str) -> Result<RPNExpr, ParseError> {
        Self::parse(&mut Lexer::from_str(expr))
    }

    pub fn parse(lex: &mut Lexer) -> Result<RPNExpr, ParseError> {
        let mut out = Vec::new();
        let mut stack = Vec::new();
        let mut arity = Vec::<usize>::new();

        while let Some(token) = lex.next() {
            match token {
                Token::Number(_)             => out.push(token),
                Token::Variable(_)           => out.push(token),
                Token::OParen                => stack.push(token),
                Token::Function(_, _)        => {
                    stack.push(token);
                    arity.push(1); // keep track of number of arguments
                },
                Token::Comma | Token::CParen => {
                    while !stack.is_empty() && stack.last() != Some(&Token::OParen) {
                        out.push(stack.pop().unwrap());
                    }
                    if stack.is_empty() {
                        return Err(ParseError::MissingOParen);
                    }
                    // end of grouping: check if this is a function call
                    if token == Token::CParen {
                        stack.pop(); // peel matching OParen
                        match stack.pop() {
                            Some(Token::Function(func, _)) =>
                                out.push(Token::Function(func, arity.pop().unwrap())),
                            Some(other) => stack.push(other),
                            None => ()
                        }
                    } else if let Some(a) = arity.last_mut() { *a += 1; } // Comma
                },
                Token::Op(_, _) => {
                    let (prec_rhs, assoc_rhs) = token.precedence();
                    while !stack.is_empty() {
                        let (prec_lhs, _) = stack.last().unwrap().precedence();
                        if prec_rhs > prec_lhs {
                            break;
                        } else if prec_rhs < prec_lhs {
                            out.push(stack.pop().unwrap());
                        } else {
                            match assoc_rhs {
                                Assoc::Right => break,
                                Assoc::None => return Err(ParseError::NonAssoc),
                                Assoc::Left => out.push(stack.pop().unwrap())
                            }
                        }
                    }
                    stack.push(token);
                },
                Token::Unknown(lexeme) => return Err(ParseError::BadToken(lexeme))
            }
        }
        while let Some(top) = stack.pop() {
            match top {
                Token::OParen => return Err(ParseError::MissingCParen),
                token         => out.push(token),
            }
        }
        Ok(RPNExpr{expr: out})
    }

}
