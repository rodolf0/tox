use std::io;
use std::cmp::Ordering;
use math_lexer::{MathLexer, MathToken, LexComp};

#[derive(Show, PartialEq)]
enum Assoc {
    Left,
    Right,
    None
}

// Get operator precedence and associativity
fn precedence(lc: &LexComp) -> (usize, Assoc) {
    match *lc {
        // need OParen/Function because they can be pushed onto the stack
        LexComp::OParen | LexComp::Function => (1, Assoc::Left),
        LexComp::Plus | LexComp::Minus => (2, Assoc::Left),
        LexComp::Times | LexComp::Divide | LexComp::Modulo => (3, Assoc::Left),
        LexComp::UMinus => (4, Assoc::Right),
        LexComp::Power => (5, Assoc::Right),
        LexComp::Factorial => (6, Assoc::Left),
        _ => (100, Assoc::None)
    }
}


// A parser token
#[derive(Show, PartialEq)]
struct Token {
    pub lxtok: MathToken,
    pub arity: usize // number of function parameters
}

// The type of a parsed expression turned into RPN
pub type RPNExpr = Vec<Token>;

// Errors that can arise while parsing
#[derive(Show, PartialEq)]
pub enum ParseError {
    MissingOParen,
    MissingCParen,
    MisplacedComma,
    NonAssociative,
    UnknownToken(String),
}

pub fn parse(expr: &str) -> Result<RPNExpr, ParseError> {
    let mut ml = MathLexer::from_str(expr);
    mlparse(&mut ml)
}

// Parse expression with shunting yard algorithm
// http://en.wikipedia.org/wiki/Shunting-yard_algorithm
pub fn mlparse<R: io::Reader>(ml: &mut MathLexer<R>) -> Result<RPNExpr, ParseError> {
    let mut out = Vec::new();
    let mut stack = Vec::new();
    let mut arity = Vec::new();

    'next_token: while let Some(mltok) = ml.next() {
        match mltok.lexcomp {
            LexComp::Number | LexComp::Variable => out.push(Token{lxtok: mltok, arity: 0}),

            LexComp::Function => {
                stack.push(Token{lxtok: mltok, arity: 0});
                arity.push(1);
            },

            // Start-of-grouping token
            LexComp::OParen => stack.push(Token{lxtok: mltok, arity: 0}),

            // function-argument/group-element separator
            LexComp::Comma => {
                // track n-arguments for function calls. If cannot unwrap => bad parens
                if let Some(a) = arity.last_mut() { *a += 1; }
                while let Some(top) = stack.pop() {
                    if top.lxtok.lexcomp != LexComp::OParen {
                        out.push(top);
                    } else {
                        stack.push(top);
                        continue 'next_token;
                    }
                }
                return Err(ParseError::MisplacedComma);
            },

            // End-of-grouping token
            LexComp::CParen => {
                while let Some(top) = stack.pop() {
                    if top.lxtok.lexcomp != LexComp::OParen {
                        out.push(top);
                    } else {
                        // found OParen, check if it's a function call
                        if let Some(mut func) = stack.pop() {
                            if func.lxtok.lexcomp == LexComp::Function {
                                // adjust the function arity based on seen arguments
                                func.arity = arity.pop().unwrap();
                                out.push(func);
                            } else {
                                stack.push(func); // nope! return token
                            }
                        }
                        continue 'next_token;
                    }
                }
                return Err(ParseError::MissingOParen);
            },

            // Operators
            LexComp::Plus | LexComp::Minus |
            LexComp::Times | LexComp::Divide | LexComp::Modulo |
            LexComp::UMinus | LexComp::Power | LexComp::Factorial => {
                let (buf_prec, buf_assoc) = precedence(&mltok.lexcomp);
                while let Some(top) = stack.pop() {
                    let (top_prec, _) = precedence(&top.lxtok.lexcomp);
                    match buf_prec.cmp(&top_prec) {
                        Ordering::Greater => { stack.push(top); break; }, // return top to stack
                        Ordering::Equal if buf_assoc == Assoc::Right => { stack.push(top); break; },
                        Ordering::Equal if buf_assoc == Assoc::None => return Err(ParseError::NonAssociative),
                        _ => out.push(top)
                    }
                }
                stack.push(Token{lxtok: mltok, arity: 2}); // only care about arity for Function
            },

            _ => return Err(ParseError::UnknownToken(mltok.lexeme))
        }
    }
    while let Some(top) = stack.pop() {
        if top.lxtok.lexcomp == LexComp::OParen {
            return Err(ParseError::MissingCParen);
        }
        out.push(top);
    }
    Ok(out)
}



#[cfg(test)]
mod test {
    use super::{Token, parse, ParseError};
    use math_lexer::{LexComp, MathToken};

    #[test]
    fn test_parse1() {
        let rpn = parse("3+4*2/-(1-5)^2^3").ok().unwrap();
        let expect = [
            ("3", LexComp::Number),
            ("4", LexComp::Number),
            ("2", LexComp::Number),
            ("*", LexComp::Times),
            ("1", LexComp::Number),
            ("5", LexComp::Number),
            ("-", LexComp::Minus),
            ("2", LexComp::Number),
            ("3", LexComp::Number),
            ("^", LexComp::Power),
            ("^", LexComp::Power),
            ("-", LexComp::UMinus),
            ("/", LexComp::Divide),
            ("+", LexComp::Plus),
        ];
        for (i, &(ref lexeme, ref lexcomp)) in expect.iter().enumerate() {
            let Token{lxtok: MathToken{lexeme: ref lx, lexcomp: ref lc }, arity: _} = rpn[i];
            assert_eq!(*lexcomp, *lc);
            assert_eq!(*lexeme, *lx);
        }
    }

    #[test]
    fn test_parse2() {
        let rpn = parse("3.4e-2 * sin(x)/(7! % -4) * max(2, x)").ok().unwrap();
        let expect = [
            ("3.4e-2", LexComp::Number),
            ("x", LexComp::Variable),
            ("sin", LexComp::Function),
            ("*", LexComp::Times),
            ("7", LexComp::Number),
            ("!", LexComp::Factorial),
            ("4", LexComp::Number),
            ("-", LexComp::UMinus),
            ("%", LexComp::Modulo),
            ("/", LexComp::Divide),
            ("2", LexComp::Number),
            ("x", LexComp::Variable),
            ("max", LexComp::Function),
            ("*", LexComp::Times),
        ];
        for (i, &(ref lexeme, ref lexcomp)) in expect.iter().enumerate() {
            let Token{lxtok: MathToken{lexeme: ref lx, lexcomp: ref lc }, arity: _} = rpn[i];
            assert_eq!(*lexcomp, *lc);
            assert_eq!(*lexeme, *lx);
        }
    }

    #[test]
    fn test_parse3() {
        let rpn = parse("sqrt(-(1i-x^2) / (1 + x^2))").ok().unwrap();
        let expect = [
            ("1i", LexComp::Number),
            ("x", LexComp::Variable),
            ("2", LexComp::Number),
            ("^", LexComp::Power),
            ("-", LexComp::Minus),
            ("-", LexComp::UMinus),
            ("1", LexComp::Number),
            ("x", LexComp::Variable),
            ("2", LexComp::Number),
            ("^", LexComp::Power),
            ("+", LexComp::Plus),
            ("/", LexComp::Divide),
            ("sqrt", LexComp::Function),
        ];
        for (i, &(ref lexeme, ref lexcomp)) in expect.iter().enumerate() {
            let Token{lxtok: MathToken{lexeme: ref lx, lexcomp: ref lc }, arity: _} = rpn[i];
            assert_eq!(*lexcomp, *lc);
            assert_eq!(*lexeme, *lx);
        }
    }

    #[test]
    fn bad_parse() {
        let rpn = parse("sqrt(-(1i-x^2) / (1 + x^2)");
        assert_eq!(rpn, Err(ParseError::MissingCParen));

        let rpn = parse("-(1i-x^2) / (1 + x^2))");
        assert_eq!(rpn, Err(ParseError::MissingOParen));

        let rpn = parse("max 4, 6, 4)");
        assert_eq!(rpn, Err(ParseError::MisplacedComma));
    }

    #[test]
    fn check_arity() {
        use std::collections::HashMap;
        let rpn = parse("sin(1)+(max(2, gamma(3.5), gcd(24, 8))+sum(i,0,10))");
        let mut rpn = rpn.ok().unwrap();
        let mut expect = HashMap::new();
        expect.insert("sin", 1);
        expect.insert("max", 3);
        expect.insert("gamma", 1);
        expect.insert("gcd", 2);
        expect.insert("sum", 3);

        while let Some(tok) = rpn.pop() {
            if tok.lxtok.lexcomp == LexComp::Function {
                let expected_arity = expect.get(tok.lxtok.lexeme.as_slice());
                assert_eq!(*expected_arity.unwrap(), tok.arity);
            }
        }
    }
}
