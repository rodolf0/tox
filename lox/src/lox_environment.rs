#![deny(warnings)]

use std::collections::HashMap;
use lox_interpreter::V;


pub struct Environment {
    values: HashMap<String, V>,
}

impl Environment {
    pub fn new() -> Self {
        Environment{values: HashMap::new()}
    }

    pub fn define(&mut self, name: String, val: V) {
        self.values.insert(name, val);
    }

    pub fn get(&self, name: &str) -> Result<V, String> {
        self.values.get(name)
            .ok_or(format!("undefined variable '{}'", name))
            .map(|v| v.clone())
    }

    pub fn assign(&mut self, name: String, val: V) -> Result<V, String> {
        if self.values.contains_key(&name) {
            self.values.insert(name, val.clone());
            return Ok(val)
        }
        Err(format!("undefined variable '{}'", name))
    }
}
