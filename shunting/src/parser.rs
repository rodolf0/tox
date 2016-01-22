use lexers::{Scanner, MathTokenizer, MathToken, TokenAssoc};
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
    expr: Vec<MathToken>
}

impl Deref for RPNExpr {
    type Target = Vec<MathToken>;
    fn deref<'a>(&'a self) -> &'a Vec<MathToken> { &self.expr }
}

pub struct ShuntingParser;

impl ShuntingParser {
    pub fn parse_str(expr: &str) -> Result<RPNExpr, ParseError> {
        Self::parse(&mut MathTokenizer::from_str(expr))
    }

    pub fn parse(lex: &mut Scanner<MathToken>) -> Result<RPNExpr, ParseError> {
        let mut out = Vec::new();
        let mut stack = Vec::new();
        let mut arity = Vec::<usize>::new();

        while let Some(token) = lex.next() {
            match token {
                MathToken::Number(_)             => out.push(token),
                MathToken::Variable(_)           => out.push(token),
                MathToken::OParen                => stack.push(token),
                MathToken::Function(_, _)        => {
                    stack.push(token);
                    arity.push(1); // keep track of number of arguments
                },
                MathToken::Comma | MathToken::CParen => {
                    while !stack.is_empty() && stack.last() != Some(&MathToken::OParen) {
                        out.push(stack.pop().unwrap());
                    }
                    if stack.is_empty() {
                        return Err(ParseError::MissingOParen);
                    }
                    // end of grouping: check if this is a function call
                    if token == MathToken::CParen {
                        stack.pop(); // peel matching OParen
                        match stack.pop() {
                            Some(MathToken::Function(func, _)) =>
                                out.push(MathToken::Function(func, arity.pop().unwrap())),
                            Some(other) => stack.push(other),
                            None => ()
                        }
                    } else if let Some(a) = arity.last_mut() { *a += 1; } // Comma
                },
                MathToken::Op(_, _) => {
                    let (prec_rhs, assoc_rhs) = token.precedence();
                    while !stack.is_empty() {
                        let (prec_lhs, _) = stack.last().unwrap().precedence();
                        if prec_lhs < prec_rhs {
                            break;
                        } else if prec_lhs > prec_rhs {
                            out.push(stack.pop().unwrap());
                        } else {
                            match assoc_rhs {
                                TokenAssoc::Left  => out.push(stack.pop().unwrap()),
                                TokenAssoc::None  => return Err(ParseError::NonAssoc),
                                TokenAssoc::Right => break
                            }
                        }
                    }
                    stack.push(token);
                },
                MathToken::Unknown(lexeme) => return Err(ParseError::BadToken(lexeme))
            }
        }
        while let Some(top) = stack.pop() {
            match top {
                MathToken::OParen => return Err(ParseError::MissingCParen),
                token             => out.push(token),
            }
        }
        Ok(RPNExpr{expr: out})
    }

}
