#![deny(warnings)]

use crate::lox_environment::Environment;
use crate::lox_interpreter::{Callable, LoxInterpreter, V};
use std::rc::Rc;

pub struct Clock;

impl Callable for Clock {
    fn call(&self, _: &mut LoxInterpreter, _: &[V]) -> Result<V, String> {
        Ok(V::Num(
            (time::OffsetDateTime::now_utc() - time::OffsetDateTime::UNIX_EPOCH)
            .whole_nanoseconds() as f64))
    }
    fn arity(&self) -> usize { 0 }
    fn id(&self) -> String { "clock".to_string() }
}

pub fn native_fn_env() -> Environment {
    let mut environment = Environment::new(None);
    environment.define("clock", V::Callable(Rc::new(Clock)));
    environment
}
