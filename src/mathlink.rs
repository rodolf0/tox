use std::dynamic_lib::DynamicLibrary;
use std::mem;

// Use dynamic linker to get hold of math library functions (dlopen self)
pub fn link_fn(fname: &str) -> Result<fn(f64) -> f64, String> {
    match DynamicLibrary::open(None) {
        Ok(lib) => unsafe {
            match lib.symbol(fname) {
                Ok(f) => Ok(mem::transmute::<*mut u8, fn(f64) -> f64>(f)),
                Err(e) => Err(e)
            }
        },
        Err(e) => Err(e)
    }
}
