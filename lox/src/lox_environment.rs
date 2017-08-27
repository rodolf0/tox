#![deny(warnings)]

use std::collections::HashMap;
use lox_interpreter::V;
use std::cell::RefCell;
use std::rc::Rc;


pub struct Environment {
    values: HashMap<String, V>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment{values: HashMap::new(), enclosing: enclosing}
    }

    pub fn define(&mut self, name: String, val: V) {
        self.values.insert(name, val);
    }

    pub fn get(&self, name: &str) -> Result<V, String> {
        if self.values.contains_key(name) {
            return Ok(self.values.get(name).unwrap().clone());
        } else if let Some(ref enc) = self.enclosing {
            return enc.borrow().get(name);
        }
        Err(format!("undefined variable '{}'", name))
    }

    pub fn assign(&mut self, name: String, val: V) -> Result<V, String> {
        if self.values.contains_key(&name) {
            self.values.insert(name, val.clone());
            return Ok(val)
        } else if let Some(ref mut enc) = self.enclosing {
            return enc.borrow_mut().assign(name, val);
        }
        Err(format!("undefined variable '{}'", name))
    }
}
