#![deny(warnings)]

extern crate time;
use lox_interpreter::{V, Callable, LoxInterpreter};
use lox_environment::Environment;
use std::rc::Rc;


pub struct Clock;

impl Callable for Clock {
    fn call(&self, _: &mut LoxInterpreter, _: &Vec<V>) -> V {
        V::Num(time::precise_time_ns() as f64)
    }
    fn arity(&self) -> usize { 0 }
    fn id(&self) -> String { "clock".to_string() }
}

pub fn native_fn_env() -> Environment {
    let mut environment = Environment::new(None);
    environment.define("clock", V::Callable(Rc::new(Clock)));
    environment
}
