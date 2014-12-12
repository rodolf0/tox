use math_lexer::LexComp;
use shunting::RPNExpr;

// temporal workaround until gamma is reachable in the library
extern {
    fn tgamma(x: f64) -> f64;
}

// Evaluate a RPN expression
pub fn eval(rpn: &RPNExpr) -> Option<f64> {
    use std::num::Float;
    let mut stack = Vec::new();

    for tok in rpn.iter() {
        match tok.lxtok.lexcomp {
            LexComp::Number => {
                let s = tok.lxtok.lexeme.as_slice();
                let n = from_str::<f64>(s).unwrap();
                stack.push(n);
            },

            LexComp::Plus => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l + r);
            },

            LexComp::Minus => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l - r);
            },

            LexComp::Times => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l * r);
            },

            LexComp::Divide => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l / r);
            },

            LexComp::Modulo => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l.rem(&r));
            },

            LexComp::Power => {
                let (r, l) = (stack.pop().unwrap(), stack.pop().unwrap());
                stack.push(l.powf(r));
            },

            LexComp::UMinus => {
                let r = stack.pop().unwrap();
                stack.push(-r);
            },

            LexComp::Factorial => {
                let l = stack.pop().unwrap();
                stack.push(unsafe { tgamma(l) });
            },

            LexComp::Function => panic!("not-implemented"),

            LexComp::Variable | LexComp::Unknown | LexComp::OParen | LexComp::CParen | LexComp::Comma => ()
        }
    }
    stack.pop()
}
