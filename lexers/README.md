# lexers

Tools for tokenizing and scanning text.

## Scanner

The core building block. Wrap any iterator and look ahead, match patterns, and backtrack.

```rust
use lexers::{Scan, Scanner};

let mut s = "3 + 42".chars().scanner();

// Peek ahead without consuming
assert_eq!(s.peek(), Some(&'3'));

// Match digits
assert_eq!(s.accept_while(|c: &char| c.is_ascii_digit()), Some(&['3'][..]));

// Match a literal sequence
assert_eq!(s.accept_seq(" + ".chars()), Some(&[' ', '+', ' '][..]));

// Backtrack via checkpoints
let cp = s.checkpoint();
assert_eq!(s.advance(), Some(&'4'));
s.restore(cp); // back to before '4'

// Consume matched content from the buffer
assert_eq!(s.lift().collect::<String>(), "3 + ");
```

## Extractors

Functions that extract specific patterns from a `Scanner`:

```rust
use lexers::{Scanner, number, quoted, identifier};

let mut s = Scanner::new("x = 3.14e-2".chars());
assert_eq!(identifier(&mut s).map(|c| c.iter().collect::<String>()), Some("x".to_string()));

let mut s = Scanner::new(r#""hello""#.chars());
assert_eq!(quoted(&mut s, "\"", "\"", Some('\\')).map(|c| c.iter().collect::<String>()),
           Some(r#""hello""#.to_string()));

let mut s = Scanner::new("3.14e-2".chars());
assert_eq!(number(&mut s).map(|c| c.iter().collect::<String>()), Some("3.14e-2".to_string()));
```

Available extractors: `number`, `math_op`, `integer`, `identifier`, `unit`, `quoted`, `quoted_no_delims`.

## StringTokenizer

A configurable tokenizer that splits input into `String` tokens.

```rust
use lexers::StringTokenizer;

let tokens: Vec<_> = StringTokenizer::from("hello 123 , world")
    .split_by(lexers::number, false)         // extract numbers
    .split_on([",", "."], false)           // extract punctuation
    .collect();
// → ["hello", "123", ",", "world"]

// Discard whitespace and comments
let tokens: Vec<_> = StringTokenizer::from("code /* comment */ more")
    .split_by(|s| lexers::quoted_no_delims(s, "/*", "*/", None), true)
    .collect();
// → ["code", "more"]
```

## TypedTokenizer

Like `StringTokenizer`, but maps tokens to a custom type.

```rust
use lexers::{TypedTokenizer, identifier, Scanner};

#[derive(Debug, PartialEq)]
enum Token { Word(String), Number(String), Punct(String) }

let tokens: Vec<_> = TypedTokenizer::new("hello 123 , world".chars(), |chars| {
    Some(Token::Word(chars.iter().collect()))
})
.split_by(lexers::number, |chars| Some(Token::Number(chars.iter().collect())))
.split_on([",", "."], |chars| Some(Token::Punct(chars.iter().collect())))
.collect();

assert_eq!(tokens, vec![
    Token::Word("hello".to_string()),
    Token::Number("123".to_string()),
    Token::Punct(",".to_string()),
    Token::Word("world".to_string()),
]);
```

## Pre-built Tokenizers

Ready-made tokenizers for common formats:

**EbnfTokenizer** — tokenizes EBNF grammar definitions.

```rust
let grammar = r#"
    expr   := expr ('+'|'-') term | term ;
    term   := term ('*'|'/') factor | factor ;
"#;
let mut tok = lexers::EbnfTokenizer::new(grammar.chars());
```

**LispTokenizer** — tokenizes Lisp-like expressions.

```rust
let mut tok = lexers::LispTokenizer::from("(+ 3 4 5)");
for token in tok {
    println!("{:?}", token);
}
```

**MathTokenizer** — tokenizes math expressions, handling unary/binary operators and SI quantities.

```rust
let mut tok = lexers::MathTokenizer::from("3.4e-2 * sin(x) / (7! % -4)");
for token in tok {
    println!("{:?}", token);
}
```

## Writing a Custom Tokenizer

Combine `Scanner`, extractors, and the checkpoint/restore mechanism to build your own tokenizer.

```rust
use lexers::{Scan, Scanner, number, identifier, math_op};

fn my_tokenizer(input: &str) -> impl Iterator<Item = String> + use<'_> {
    let mut s = input.chars().scanner();
    std::iter::from_fn(move || {
        s.accept_while(|c: &char| c.is_whitespace()); // skip whitespace
        let _ = s.lift(); // discard matched whitespace
        if let Some(tok) = math_op(&mut s) {
            Some(tok.iter().collect())
        } else if let Some(tok) = number(&mut s) {
            Some(tok.iter().collect())
        } else if let Some(tok) = identifier(&mut s) {
            Some(tok.iter().collect())
        } else {
            s.advance().map(|c| c.to_string())
        }
    })
}

let tokens: Vec<_> = my_tokenizer("3 + foo * 2").collect();
// → ["3", "+", "foo", "*", "2"]
```
