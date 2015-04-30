use lexer::{MathLexer, MathToken, LexComp};

#[derive(PartialEq, Debug)]
pub enum ParseError {
    MissingOParen,
    MissingCParen,
    MisplacedComma,
    NonAssociative,
    UnknownToken(String),
}

#[derive(PartialEq, Debug)]
enum Assoc {
    Left,
    Right,
    None
}

#[derive(PartialEq, Debug)]
pub struct Token {
    pub lxtoken: MathToken,
    pub arity: usize
}

impl Token {
    pub fn is(&self, lexcomp: &LexComp) -> bool {
        self.lxtoken.lexcomp == *lexcomp
    }
}

pub type RPNExpr = Vec<Token>;

pub struct MathParser;

impl MathParser {
    pub fn parse_str(expr: &str) -> Result<RPNExpr, ParseError> {
        let mut lx = MathLexer::lex_str(expr);
        Self::parse(&mut lx)
    }

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

    fn some(t: &Option<&Token>, lc: &LexComp) -> bool {
        t.is_some() && t.unwrap().is(lc)
    }

    fn some_not(t: &Option<&Token>, lc: &LexComp) -> bool {
        t.is_some() && !t.unwrap().is(lc)
    }

    fn none_ornot(t: &Option<&Token>, lc: &LexComp) -> bool {
        t.is_none() || !t.unwrap().is(lc)
    }

    // http://en.wikipedia.org/wiki/Shunting-yard_algorithm
    pub fn parse(lexer: &mut MathLexer) -> Result<RPNExpr, ParseError> {
        let mut out = Vec::new();
        let mut stack = Vec::new();
        let mut arity = Vec::<usize>::new();

        while let Some(lextok) = lexer.next() {
            match lextok.lexcomp {
                LexComp::Number |
                LexComp::Variable => out.push(Token{lxtoken: lextok, arity: 0}),
                LexComp::OParen => stack.push(Token{lxtoken: lextok, arity: 0}),
                LexComp::Comma => {
                    while Self::some_not(&stack.last(), &LexComp::OParen) {
                        out.push(stack.pop().unwrap());
                    }
                    if Self::none_ornot(&stack.last(), &LexComp::OParen) {
                        return Err(ParseError::MisplacedComma);
                    }
                    if let Some(a) = arity.last_mut() { *a += 1; }
                },
                LexComp::CParen => {
                    while Self::some_not(&stack.last(), &LexComp::OParen) {
                        out.push(stack.pop().unwrap());
                    }
                    if Self::none_ornot(&stack.pop().as_ref(), &LexComp::OParen) {
                        return Err(ParseError::MissingOParen);
                    }
                    if Self::some(&stack.last(), &LexComp::Function) {
                        stack.last_mut().unwrap().arity = arity.pop().unwrap();
                        out.push(stack.pop().unwrap());
                    }
                },
                LexComp::Function => {
                    stack.push(Token{lxtoken: lextok, arity: 0});
                    arity.push(1);
                },
                LexComp::Plus   |
                LexComp::Minus  |
                LexComp::Times  |
                LexComp::Divide |
                LexComp::Modulo |
                LexComp::UMinus |
                LexComp::Power  |
                LexComp::Factorial => {
                    let (prec_rhs, assoc_rhs) = Self::precedence(&lextok.lexcomp);
                    while stack.len() > 0 {
                        let (prec_lhs, _) = {
                            let top = stack.last().unwrap();
                            Self::precedence(&top.lxtoken.lexcomp)
                        };
                        if prec_rhs > prec_lhs {
                            break;
                        } else if prec_rhs < prec_lhs {
                            out.push(stack.pop().unwrap())
                        } else {
                            match assoc_rhs {
                                Assoc::Right => break,
                                Assoc::None => return Err(ParseError::NonAssociative),
                                Assoc::Left => out.push(stack.pop().unwrap())
                            }
                        }
                    }
                    stack.push(Token{lxtoken: lextok, arity: 0});
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
}
