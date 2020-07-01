use lexers::{MathToken, MathTokenizer};

#[derive(PartialEq, Debug)]
pub enum Assoc {
    Left,
    Right,
    None,
}

pub fn precedence(mt: &MathToken) -> (usize, Assoc) {
    // You can play with the relation between exponentiation an unary - by
    // a. switching order in which the lexer tokenizes, if it tries
    // operators first then '-' will never be the negative part of number,
    // else if numbers are tried before operators, - can only be unary
    // for non-numeric tokens (eg: -(3)).
    // b. changing the precedence of '-' respect to '^'
    // If '-' has lower precedence then 2^-3 will fail to evaluate if the
    // '-' isn't part of the number because ^ will only find 1 operator
    match *mt {
        MathToken::OParen => (1, Assoc::Left), // keep at bottom
        MathToken::BOp(ref o) if o == "+" => (2, Assoc::Left),
        MathToken::BOp(ref o) if o == "-" => (2, Assoc::Left),
        MathToken::BOp(ref o) if o == "*" => (3, Assoc::Left),
        MathToken::BOp(ref o) if o == "/" => (3, Assoc::Left),
        MathToken::BOp(ref o) if o == "%" => (3, Assoc::Left),
        MathToken::UOp(ref o) if o == "-" => (5, Assoc::Right), // unary minus
        MathToken::BOp(ref o) if o == "^" => (5, Assoc::Right),
        MathToken::UOp(ref o) if o == "!" => (6, Assoc::Left), // factorial
        MathToken::Function(_, _) => (7, Assoc::Left),
        _ => (99, Assoc::None),
    }
}

#[derive(PartialEq, Debug)]
pub struct RPNExpr(pub Vec<MathToken>);

pub struct ShuntingParser;

impl ShuntingParser {
    pub fn parse_str(expr: &str) -> Result<RPNExpr, String> {
        Self::parse(&mut MathTokenizer::new(expr.chars()))
    }

    pub fn parse(lex: &mut impl Iterator<Item = MathToken>) -> Result<RPNExpr, String> {
        let mut out = Vec::new();
        let mut stack = Vec::new();
        let mut arity = Vec::<usize>::new();

        while let Some(token) = lex.next() {
            match token {
                MathToken::Number(_) => out.push(token),
                MathToken::Variable(_) => out.push(token),
                MathToken::OParen => stack.push(token),
                MathToken::Function(_, _) => {
                    stack.push(token);
                    arity.push(1);
                }
                MathToken::Comma | MathToken::CParen => {
                    while !stack.is_empty() && stack.last() != Some(&MathToken::OParen) {
                        out.push(stack.pop().unwrap());
                    }
                    if stack.is_empty() {
                        return Err(format!("Missing Opening Paren"));
                    }
                    // end of grouping: check if this is a function call
                    if token == MathToken::CParen {
                        stack.pop(); // peel matching OParen
                        match stack.pop() {
                            Some(MathToken::Function(func, _)) => {
                                out.push(MathToken::Function(func, arity.pop().unwrap()))
                            }
                            Some(other) => stack.push(other),
                            None => (),
                        }
                    } else if let Some(a) = arity.last_mut() {
                        *a += 1;
                    } // Comma
                }
                MathToken::UOp(_) | MathToken::BOp(_) => {
                    let (prec_rhs, assoc_rhs) = precedence(&token);
                    while !stack.is_empty() {
                        let (prec_lhs, _) = precedence(stack.last().unwrap());
                        if prec_lhs < prec_rhs {
                            break;
                        } else if prec_lhs > prec_rhs {
                            out.push(stack.pop().unwrap());
                        } else {
                            match assoc_rhs {
                                Assoc::Left => out.push(stack.pop().unwrap()),
                                Assoc::None => return Err(format!("No Associativity")),
                                Assoc::Right => break,
                            }
                        }
                    }
                    stack.push(token);
                }
                MathToken::Unknown(lexeme) => return Err(format!("Bad token: {}", lexeme)),
            }
        }
        while let Some(top) = stack.pop() {
            match top {
                MathToken::OParen => return Err(format!("Missing Closing Paren")),
                token => out.push(token),
            }
        }
        Ok(RPNExpr(out))
    }
}
