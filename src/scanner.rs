pub trait Nexter<T> {
    fn next(&mut self) -> Option<T>;
}

pub struct Scanner<T: Clone> {
    src: Option<Box<Nexter<T>>>,
    buf: Vec<T>,
    pos: isize,
}

impl<T: Clone> Scanner<T> {
    pub fn new(source: Box<Nexter<T>>) -> Scanner<T> {
        Scanner{src: Some(source), buf: Vec::new(), pos: -1}
    }

    pub fn pos(&self) -> isize { self.pos }

    pub fn curr(&self) -> Option<T> {
        let pos = self.pos as usize;
        if self.pos < 0 || pos >= self.buf.len() {
            return None;
        }
        Some(self.buf[pos].clone())
    }

    // get more tokens and fill the buffer
    fn check_buffer(&mut self) -> bool {
        if ((self.pos + 1) as usize) < self.buf.len() {
            return true;
        }
        if let Some(ref mut nexter) = self.src {
            if let Some(tok) = nexter.next() {
                self.buf.push(tok);
                return true;
            }
        }
        false
    }

    pub fn next(&mut self) -> Option<T> {
        if !self.check_buffer() { return None; }
        self.pos += 1;
        self.curr()
    }

    pub fn prev(&mut self) -> Option<T> {
        if self.pos >= 0 { self.pos -= 1; }
        self.curr()
    }

    pub fn peek(&mut self) -> Option<T> {
        let backtrack = self.pos;
        let peeked = self.next();
        self.pos = backtrack;
        peeked
    }

    pub fn view(&self) -> &[T] {
        let n = self.pos as usize + 1;
        &self.buf[..n]
    }

    pub fn ignore(&mut self) {
        if self.pos >= 0 {
            let n = self.pos as usize + 1;
            self.buf = self.buf[n..].to_vec();
        }
        self.pos = -1;
    }

    pub fn extract(&mut self) -> Vec<T> {
        let tokens = self.view().to_vec();
        self.ignore();
        tokens
    }
}

//impl<T: Clone> Scanner<T> {
    //// Advance the scanner only if the next char is in the 'any' set
    //// self.curr() will return the matched char if accept matched any
    //pub fn accept(&mut self, any: &[T]) -> Option<T> {
        //if let Some(next) = self.peek() {
            //if let Some(idx) = any.find(next) {
                //return Some(any[idx]);
            //}
        //}
        //None
    //}
//}


impl Scanner<char> {
    pub fn from_str(source: &str) -> Scanner<char> {
        Scanner{src: None, buf: source.chars().collect(), pos: -1}
    }

    pub fn extract_string(&mut self) -> String {
        let tokens = self.view().iter().cloned().collect::<String>();
        self.ignore();
        tokens
    }
}
