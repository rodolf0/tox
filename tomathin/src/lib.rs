// #![deny(warnings)]

mod tokenizer;

mod context;
pub use context::Context;
mod expr;
pub use expr::{eval_with_ctx, evaluate};
mod parser;
pub use parser::parser;

mod findroot;

#[cfg(test)]
mod tests;
