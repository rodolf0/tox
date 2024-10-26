mod tokenizer;

mod expr;
pub use expr::{evaluate, Expr};
mod parser;
pub use parser::parser;

mod findroot;
pub use findroot::find_root;

#[cfg(test)]
mod tests;
