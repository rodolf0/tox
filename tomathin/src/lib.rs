// #![deny(warnings)]

mod tokenizer;

mod context;
pub use context::Context;
mod expr;
pub use expr::{eval_with_ctx, evaluate};
mod parser;
pub use parser::parser;

fn gamma(x: f64) -> f64 {
    #[link(name = "m")]
    extern "C" {
        fn tgamma(x: f64) -> f64;
    }
    unsafe { tgamma(x) }
}

mod findroot;

mod matrix;

#[cfg(test)]
mod tests;
