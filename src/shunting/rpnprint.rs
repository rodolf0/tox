use shunting::{RPNExpr, Token, Assoc};
use std::fmt;

#[derive(Debug, Clone)]
enum AST<'a> {
    Leaf(&'a Token),
    Node(&'a Token, Vec<AST<'a>>),
}

impl RPNExpr {
    fn build_ast<'a>(&'a self) -> AST<'a> {
        let mut ops = Vec::new();
        for token in self.iter() {
            match *token {
                Token::Number(_) |
                Token::Variable(_) => ops.push(AST::Leaf(token)),
                Token::Function(_, arity) |
                Token::Op(_, arity) => {
                    let n = ops.len() - arity;
                    let node = AST::Node(token, ops.iter().skip(n).cloned().collect());
                    ops.truncate(n);
                    ops.push(node);
                },
                _ => unreachable!()
            }
        }
        ops.pop().unwrap()
    }
}

impl fmt::Display for RPNExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

// (3 + 4) / (2 * (5 + 3))
// 3 4 + 2 5 3 + * /

        fn printer(root: &AST) -> (String, (usize, Assoc)) {
            match root {
                &AST::Leaf(ref token) => {
                    match *token {
                        &Token::Number(ref x)   => (format!("{}", x), token.precedence()),
                        &Token::Variable(ref x) => (format!("{}", x), token.precedence()),
                        _ => unreachable!()
                    }
                },
                &AST::Node(ref token, ref args) => {
                    match *token {
                        &Token::Op(ref op, arity) if arity == 1 => {
                            let subtree = printer(&args[0]);
                            let (prec, assoc) = token.precedence();
                            // TODO: not entirely correct
                            if prec > (subtree.1).0 {
                                (format!("{}({})", op, subtree.0), (prec, assoc))
                            } else {
                                (format!("{}{}", op, subtree.0), (prec, assoc))
                            }
                        },
                        &Token::Op(ref op, arity) if arity == 2 => {
                            let (lhs, rhs) = (printer(&args[0]), printer(&args[1]));
                            let (prec, assoc) = token.precedence();

                            let lh = if prec > (lhs.1).0 ||
                                        (prec == (lhs.1).0 && assoc != Assoc::Left) {
                                format!("({})", lhs.0)
                            } else {
                                format!("{}", lhs.0)
                            };
                            let rh = if prec > (rhs.1).0 ||
                                        (prec == (rhs.1).0 && assoc != Assoc::Right) {
                                format!("({})", rhs.0)
                            } else {
                                format!("{}", rhs.0)
                            };
                            // TODO: figure out how '2+(3+4)' not print parens
                            (format!("{} {} {}", lh, op, rh), (prec, assoc))

                        },
                        _ => unreachable!()
                    }
                },
                //&AST::Node(&Token::Function(ref f, _), ref a) =>
                /*
                    let tok = match root {&AST::Node(tok, _) => tok, _ => unreachable!()};
                    let (prec, assoc) = tok.precedence();
                    let expr = a.iter()
                        .map(|leaf| printer(leaf, prec))
                        .collect::<Vec<String>>()
                        .connect(&format!(" {} ", f));

                    if prec < cur_prec ||
                       prec == cur_prec && assoc == Assoc::Right {
                        format!("({})", expr)
                    } else {
                        format!("{}", expr)
                    }
                */
            }
        }

        let x = printer(&self.build_ast());
        write!(f, "{}", x.0)
    }
}
