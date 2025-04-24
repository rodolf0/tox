#![deny(warnings)]

mod tokenizer;

mod context;
pub use context::Context;
mod expr;
pub use expr::evaluate;
mod parser;
pub use parser::parser;

mod findroot;
pub use findroot::{
    bisection, explore_domain, find_root_vec, find_roots, gauss_seidel, newton_raphson, nsolve,
    regula_falsi,
};

mod matrix;
pub use matrix::{dot_product, gram_schmidt_orthonorm, outer_product, qr_decompose};

#[cfg(test)]
mod tests;
