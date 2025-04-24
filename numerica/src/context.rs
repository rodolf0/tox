use crate::expr::Expr;

use std::collections::HashMap;

pub struct Context {
    bindings: HashMap<String, Expr>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn extend(&self) -> Self {
        Self {
            bindings: self.bindings.clone(),
        }
    }

    pub fn set(&mut self, sym: String, expr: Expr) {
        self.bindings.insert(sym, expr);
    }

    pub fn get(&self, sym: &str) -> Option<Expr> {
        self.bindings.get(sym).cloned()
    }
}
