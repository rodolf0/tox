#![deny(warnings)]

use std::collections::HashMap;
use lox_interpreter::V;
use std::cell::RefCell;
use std::rc::Rc;


pub struct Environment {
    values: HashMap<String, V>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment{values: HashMap::new(), parent: parent}
    }

    fn ancestor(&self, depth: usize) -> Option<Rc<RefCell<Environment>>> {
        let mut ancestor = self.parent.clone();
        for _ in 1..depth {
            ancestor = match ancestor {
                Some(a) => a.borrow().parent.clone(),
                None => return None
            }
        }
        ancestor
    }

    pub fn define<S: Into<String>>(&mut self, name: S, val: V) {
        self.values.insert(name.into(), val);
    }

    pub fn get(&self, name: &str) -> Result<V, String> {
        if self.values.contains_key(name) {
            return Ok(self.values.get(name).unwrap().clone());
        } else if let Some(ref enc) = self.parent {
            return enc.borrow().get(name);
        }
        Err(format!("Environment get - undefined entity '{}'", name))
    }

    pub fn get_at(&self, depth: usize, name: &str) -> Result<V, String> {
        match depth > 0 {
            false => Ok(self.values.get(name).unwrap().clone()),
            true => match self.ancestor(depth) {
                None => panic!("Resolver Bug! wrong env depth {}", depth),
                Some(env) => match env.borrow().values.get(name) {
                    Some(value) => Ok(value.clone()),
                    _ => Err(format!(
                        "Environment get_at - undefined entity '{}' depth {}",
                        name, depth))
                }
            }
        }
    }

    pub fn assign(&mut self, name: String, val: V) -> Result<V, String> {
        if self.values.contains_key(&name) {
            self.values.insert(name, val.clone());
            return Ok(val)
        } else if let Some(ref mut enc) = self.parent {
            return enc.borrow_mut().assign(name, val);
        }
        Err(format!("Environment assign - undefined entity '{}'", name))
    }

    pub fn assign_at(&mut self, depth: usize,
                     name: String, val: V) -> Result<V, String> {
        match depth > 0 {
            false => if self.values.contains_key(&name) {
                self.values.insert(name, val.clone());
                return Ok(val);
            },
            true => match self.ancestor(depth) {
                None => panic!("Resolver Bug! wrong env depth {}", depth),
                Some(env) => if env.borrow().values.contains_key(&name) {
                    env.borrow_mut().values.insert(name, val.clone());
                    return Ok(val);
                }
            }
        }
        Err(format!("Environment assign_at - undefined entity '{}'", name))
    }
}
