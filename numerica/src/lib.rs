#![deny(warnings)]

mod tokenizer;

mod context;
pub use context::Context;
mod expr;
pub use expr::{Expr, evaluate, is_stochastic};
mod parser;
pub use parser::{expr_tree, parser};

// TODO: clean this up
mod findroot;
pub use findroot::{
    bisection, explore_domain, find_root_vec, find_roots, gauss_seidel, newton_raphson, nsolve,
    regula_falsi,
};

mod matrix;
pub use matrix::{dot_product, gram_schmidt_orthonorm, outer_product, qr_decompose};

#[cfg(test)]
mod tests;
