use lexer::{MathLexer, MathToken, LexComp};
use std::cmp::Ordering;

#[derive(PartialEq, Debug)]
enum Assoc {
    Left,
    Right,
    None
}

#[derive(PartialEq, Debug)]
pub struct Token {
    pub lxtoken: MathToken, // TODO: make priv
    pub arity: usize
}

impl Token {
    pub fn is(&self, lexcomp: &LexComp) -> bool {
        self.lxtoken.lexcomp == *lexcomp
    }
}

#[derive(PartialEq, Debug)]
pub enum ParseError {
    MissingOParen,
    MissingCParen,
    MisplacedComma,
    NonAssociative,
    UnknownToken(String),
}

pub type RPNExpr = Vec<Token>;

fn precedence(lc: &LexComp) -> (usize, Assoc) {
    match *lc {
        // need OParen/Function because they can be pushed onto the stack
        LexComp::OParen |
        LexComp::Function => (1, Assoc::Left),
        LexComp::Plus |
        LexComp::Minus => (2, Assoc::Left),
        LexComp::Times |
        LexComp::Divide |
        LexComp::Modulo => (3, Assoc::Left),
        LexComp::UMinus => (4, Assoc::Right),
        LexComp::Power => (5, Assoc::Right),
        LexComp::Factorial => (6, Assoc::Left),
        _ => (100, Assoc::None)
    }
}

pub fn parse(expr: &str) -> Result<RPNExpr, ParseError> {
    let mut lx = MathLexer::lex_str(expr);
    _parse(&mut lx)
}



// http://en.wikipedia.org/wiki/Shunting-yard_algorithm
pub fn _parse(lexer: &mut MathLexer) -> Result<RPNExpr, ParseError> {
    let mut out = Vec::new();
    let mut stack = Vec::new();
    let mut arity = Vec::new();

    while let Some(lextok) = lexer.next() {
        match lextok.lexcomp {
            LexComp::Number |
            LexComp::Variable => out.push(Token{lxtoken: lextok, arity: 0}),

            LexComp::Function => {
                stack.push(Token{lxtoken: lextok, arity: 0});
                arity.push(1);
            },

            // Start-of-grouping token
            LexComp::OParen => stack.push(Token{lxtoken: lextok, arity: 0}),

            // function-argument/group-element separator
            LexComp::Comma => {
                // track n-arguments for function calls. If cannot unwrap => bad parens

                while stack.last().is_some() &&
                    !stack.last().unwrap().is(&LexComp::OParen) {
                    out.push(stack.pop().unwrap());
                }

                if stack.len() == 0 {
                    return Err(ParseError::MisplacedComma);
                }

                if let Some(a) = arity.last_mut() { *a += 1; }

            },

            // End-of-grouping token
            LexComp::CParen => {

                //while stack.last().some_and(|&t| !t.is(&LexComp::OParen)) {
                while stack.last().is_some() &&
                    !stack.last().unwrap().is(&LexComp::OParen) {
                    out.push(stack.pop().unwrap());
                }

                if stack.len() == 0 {
                    return Err(ParseError::MissingOParen);
                } else {
                    stack.pop(); // remove paren
                    if stack.last().is_some() &&
                       stack.last().unwrap().is(&LexComp::Function) {
                        // adjust the function arity based on seen arguments
                        let mut func = stack.pop().unwrap();
                        func.arity = arity.pop().unwrap();
                        out.push(func);
                    }

                }
            },

            // Operators
            LexComp::Plus   |
            LexComp::Minus  |
            LexComp::Times  |
            LexComp::Divide |
            LexComp::Modulo |
            LexComp::UMinus |
            LexComp::Power  |
            LexComp::Factorial => {
                let (buf_prec, buf_assoc) = precedence(&lextok.lexcomp);
                while let Some(top) = stack.pop() {
                    let (top_prec, _) = precedence(&top.lxtoken.lexcomp);
                    match buf_prec.cmp(&top_prec) {
                        Ordering::Greater => { stack.push(top); break; }, // return top to stack
                        Ordering::Equal if buf_assoc == Assoc::Right => { stack.push(top); break; },
                        Ordering::Equal if buf_assoc == Assoc::None => return Err(ParseError::NonAssociative),
                        _ => out.push(top)
                    }
                }
                stack.push(Token{lxtoken: lextok, arity: 2}); // only care about arity for Function
            },

            _ => return Err(ParseError::UnknownToken(lextok.lexeme))
        }
    }
    while let Some(top) = stack.pop() {
        if top.is(&LexComp::OParen) {
            return Err(ParseError::MissingCParen);
        }
        out.push(top);
    }
    Ok(out)
}
