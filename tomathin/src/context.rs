use crate::parser::Expr;

use std::collections::HashMap;

pub struct Context {
    c: HashMap<String, Expr>,
}

impl Context {
    pub fn new() -> Self {
        Self { c: HashMap::new() }
    }

    pub fn set(&mut self, sym: String, expr: Expr) {
        self.c.insert(sym, expr);
    }

    pub fn get(&self, sym: &str) -> Option<Expr> {
        self.c.get(sym).cloned()
    }
}
