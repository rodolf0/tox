use crate::parser::{precedence, Assoc, RPNExpr};
use lexers::MathToken;
use std::fmt;

#[derive(Debug, Clone)]
enum AST<'a> {
    Leaf(&'a MathToken),
    Node(&'a MathToken, Vec<AST<'a>>),
}

impl RPNExpr {
    fn build_ast(&self) -> AST {
        let mut ops = Vec::new();
        for token in self.0.iter() {
            match *token {
                MathToken::Number(_) | MathToken::Variable(_) => ops.push(AST::Leaf(token)),
                MathToken::Function(_, arity) => {
                    let n = ops.len() - arity;
                    let operands = ops.split_off(n);
                    ops.push(AST::Node(token, operands));
                }
                MathToken::BOp(_) => {
                    let n = ops.len() - 2;
                    let operands = ops.split_off(n);
                    ops.push(AST::Node(token, operands));
                }
                MathToken::UOp(_) => {
                    let n = ops.len() - 1;
                    let operands = ops.split_off(n);
                    ops.push(AST::Node(token, operands));
                }
                _ => unreachable!(),
            }
        }
        ops.pop().unwrap()
    }
}

impl fmt::Display for RPNExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn printer(root: &AST) -> (String, (usize, Assoc)) {
            match root {
                AST::Leaf(ref token) => match *token {
                    MathToken::Number(ref x) => (format!("{}", x), precedence(token)),
                    MathToken::Variable(ref x) => (format!("{}", x), precedence(token)),
                    _ => unreachable!(),
                },
                AST::Node(ref token, ref args) => {
                    match *token {
                        MathToken::UOp(ref op) => {
                            let subtree = printer(&args[0]);
                            let (prec, assoc) = precedence(token);
                            // TODO: distinguish perfix/postfix operators
                            if prec > (subtree.1).0 {
                                (format!("{}({})", op, subtree.0), (prec, assoc))
                            } else {
                                (format!("{}{}", op, subtree.0), (prec, assoc))
                            }
                        }
                        MathToken::BOp(ref op) => {
                            let (lhs, rhs) = (printer(&args[0]), printer(&args[1]));
                            let (prec, assoc) = precedence(token);

                            let lh = if prec > (lhs.1).0
                                || (prec == (lhs.1).0 && assoc != Assoc::Left)
                            {
                                format!("({})", lhs.0)
                            } else {
                                format!("{}", lhs.0)
                            };
                            let rh = if prec > (rhs.1).0
                                || (prec == (rhs.1).0 && assoc != Assoc::Right)
                            {
                                format!("({})", rhs.0)
                            } else {
                                format!("{}", rhs.0)
                            };
                            // NOTE: '2+(3+4)' will show parens to indicate that user
                            // explicitly put them there
                            (format!("{} {} {}", lh, op, rh), (prec, assoc))
                        }
                        MathToken::Function(ref func, _) => {
                            let expr = args
                                .iter()
                                .map(|leaf| printer(&leaf).0)
                                .collect::<Vec<String>>()
                                .join(", ");
                            (format!("{}({})", func, expr), precedence(token))
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }

        write!(f, "{}", printer(&self.build_ast()).0)
    }
}
