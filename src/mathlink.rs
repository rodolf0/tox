#![feature(std_misc)]
use std::dynamic_lib::DynamicLibrary;
use std::mem;

// Use dynamic linker to get hold of math library functions
pub fn link_fn(fname: &str) -> Result<fn(f64) -> f64, String> {
    match DynamicLibrary::open(None) { // open self
        Err(e) => return Err(e),
        Ok(lib) => {
            let func = unsafe {
                // a very generic pointer: '*mut u8'
                match lib.symbol(fname) {
                    Err(e) => return Err(e),
                    Ok(f) => mem::transmute::<*mut u8, fn(f64) -> f64>(f)
                }
            };
            return Ok(func);
        }
    }
}
