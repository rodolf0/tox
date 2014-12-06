extern crate tox;

fn main() {
    use std::io;
    let b = io::MemReader::new(b"just a test buffer".to_vec());
    let mut s = tox::scanner::Scanner::new(b);

    loop {
        match s.next() {
            Err(e) => {
                println!("{}", e);
                break;
            },
            Ok(c) => print!("{}", c)
        }
    }
}
