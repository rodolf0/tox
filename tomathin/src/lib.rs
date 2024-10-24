mod expr;
pub use expr::{evaluate, Expr};
mod parser;
mod tokenizer;
pub use parser::parser;

mod findroot;
pub use findroot::find_root;

#[cfg(test)]
mod tests;
