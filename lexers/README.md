# Documentation

## Tokenizers
This crate provides multiple tokenizers built on top of `Scanner`.

- **EbnfTokenizer**: A tokenizing an EBNF grammar.
```rust
let grammar = r#"
    expr   := expr ('+'|'-') term | term ;
    term   := term ('*'|'/') factor | factor ;
    factor := '-' factor | power ;
    power  := ufact '^' factor | ufact ;
    ufact  := ufact '!' | group ;
    group  := num | '(' expr ')' ;
"#;
let mut tok = EbnfTokenizer::new(grammar.chars())
```
- **LispTokenizer**: for tokenizing lisp like input.
```rust
LispTokenizer::new("(+ 3 4 5)".chars());
```
- **MathTokenizer**: emits `MathToken` tokens.
```rust
MathTokenizer::new("3.4e-2 * sin(x)/(7! % -4)".chars());
```
- **DelimTokenizer**: emits tokens split by some delimiter.


## Scanner
`Scanner` is the building block for implementing tokenizers. You can build one from an Iterator and use it to extract tokens. Check the above mentioned tokenizers for examples.

### Example

```rust
// Define a Tokenizer
struct Tokenizer<I: Iterator<Item=char>>(lexers::Scanner<I>);

impl<I: Iterator<Item=char>> Iterator for Tokenizer<I> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.scan_whitespace();
        self.0.scan_math_op()
            .or_else(|| self.0.scan_number())
            .or_else(|| self.0.scan_identifier())
    }
}

fn tokenizer<I: Iterator<Item=char>>(input: I) -> Tokenizer<I> {
    Tokenizer(lexers::Scanner::new(input))
}

// Use it to tokenize a math expression
let mut lx = tokenizer("3+4*2/-(1-5)^2^3".chars());
let token = lex.next();
```

### Tips

- `scan_X` functions try to consume some text-object out of the scanner. For example numbers, identifiers, quoted strings, etc.

- `buffer_pos` and `set_buffer_pos` are used for back-tracking as long as the Scanner's buffer still has the data you need. That means you haven't consumed or discarded it.
