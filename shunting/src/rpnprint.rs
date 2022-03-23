use crate::parser::RPNExpr;
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
        for token in &self.0 {
            match token {
                MathToken::Number(_) | MathToken::Variable(_) =>
                    ops.push(AST::Leaf(token)),
                MathToken::Function(_, arity) => {
                    let children = ops.split_off(ops.len() - arity);
                    ops.push(AST::Node(token, children));
                },
                MathToken::BOp(_) => {
                    let children = ops.split_off(ops.len() - 2);
                    ops.push(AST::Node(token, children));
                },
                MathToken::UOp(_) => {
                    let children = ops.split_off(ops.len() - 1);
                    ops.push(AST::Node(token, children));
                },
                _ => unreachable!(),
            }
        }
        ops.pop().unwrap()
    }
}

impl fmt::Display for RPNExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn print_helper(root: &AST, indent: &str, out: &mut String) {
            match root {
                AST::Leaf(tok) => *out += &format!("\u{2500}{:?}\n", tok),
                AST::Node(tok, children) => {
                    // Print current node
                    *out += &format!("\u{252c}{:?}\n", tok);
                    // Print its children
                    if let Some((last_node, rest)) = children.split_last() {
                        for mid_node in rest {
                            *out += &format!("{}\u{251c}", indent);
                            print_helper(mid_node, &format!("{}\u{2502}", indent), out);
                        }
                        *out += &format!("{}\u{2570}", indent);
                        print_helper(last_node, &format!("{} ", indent), out);
                    }
                }
            }
        }
        let mut output = String::new();
        print_helper(&self.build_ast(), "", &mut output);
        write!(f, "{}", output)
    }
}
