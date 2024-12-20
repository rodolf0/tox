#![deny(warnings)]

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
pub use findroot::{
    bisection, explore_domain, find_root, find_root_vec, gauss_seidel, nsolve, regula_falsi,
};

mod matrix;
pub use matrix::{dot_product, gram_schmidt_orthonorm, outer_product, qr_decompose};

#[cfg(test)]
mod tests;
