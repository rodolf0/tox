#![deny(warnings)]

mod tokenizer;

mod context;
pub use context::Context;
mod expr;
pub use expr::evaluate;
mod parser;
pub use parser::parser;

fn gamma(x: f64) -> f64 {
    #[link(name = "m")]
    unsafe extern "C" {
        fn tgamma(x: f64) -> f64;
    }
    unsafe { tgamma(x) }
}

mod findroot;
pub use findroot::{
    bisection, explore_domain, find_root_vec, find_roots, gauss_seidel, newton_raphson, nsolve,
    regula_falsi,
};

mod matrix;
pub use matrix::{dot_product, gram_schmidt_orthonorm, outer_product, qr_decompose};

#[cfg(test)]
mod tests;
