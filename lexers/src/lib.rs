#![deny(warnings)]

mod scanner;
mod helpers;
mod delim_tokenizer;
mod ebnf_tokenizer;
mod lisp_tokenizer;
mod math_tokenizer;

pub use crate::scanner::Scanner;
pub use crate::math_tokenizer::{MathTokenizer, MathToken};
pub use crate::delim_tokenizer::DelimTokenizer;
pub use crate::lisp_tokenizer::{LispTokenizer, LispToken};
pub use crate::ebnf_tokenizer::EbnfTokenizer;

pub use crate::helpers::scan_identifier;
pub use crate::helpers::scan_math_op;
pub use crate::helpers::scan_number;
pub use crate::helpers::scan_quoted_string;
pub use crate::helpers::scan_xob_integers;

#[cfg(test)]
mod scanner_test;
