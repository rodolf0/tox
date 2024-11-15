// #![deny(warnings)]

mod tokenizer;

mod expr;
pub use expr::evaluate;
mod parser;
pub use parser::parser;

mod findroot;

#[cfg(test)]
mod tests;
