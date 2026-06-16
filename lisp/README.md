# lisp

A minimal Lisp interpreter inspired by [Norvig's lispy](http://norvig.com/lispy.html). 
Supports expressions, special forms, closures, and a basic REPL.

## Quick Start

```rust
use lisp::LispContext;

let result = LispContext::eval_str("(* pi (* 10 10))").unwrap();
println!("{}", result.to_string());
```

## Parsing and Evaluating

```rust
use lisp::{parse, LispContext};
use std::rc::Rc;

let expr = parse("(define r 10)").unwrap();
let cx = Rc::new(LispContext::new());
LispContext::eval(&expr, &cx).unwrap();

let expr = parse("(* pi (* r r))").unwrap();
let result = LispContext::eval(&expr, &cx).unwrap();
println!("{}", result.to_string());
```

## Special Forms

| Form | Description |
|------|-------------|
| `quote` | Return expression without evaluating |
| `if` | Conditional: `(if test consequent alternative)` |
| `define` | Bind a symbol in the current scope |
| `set!` | Update an existing binding |
| `lambda` | Create an anonymous function |

## Builtins

| Function | Description |
|----------|-------------|
| `+`, `-`, `*`, `/`, `%` | Arithmetic (fold over arguments) |
| `<`, `<=`, `>`, `>=`, `=`, `!=` | Comparison |
| `first` | First element of a list |
| `tail` | Rest of a list |
| `cons` | Prepend element to list |
| `list` | Create a list from arguments |
| `length` | Length of string or list |
| `number?`, `list?`, `symbol?`, `procedure?`, `null?` | Type predicates |
| `begin` | Evaluate sequence, return last value |

## The REPL

The crate ships with a simple REPL:

```
$ cargo run -p toxtools --bin lisp
~> (define r 10)
#t
~> (* pi (* r r))
314.1592653589793
~> (define square (lambda (x) (* x x)))
#t
~> (square 5)
25
```

## References

- [lispy](http://norvig.com/lispy.html) — Norvig's original
- [lispy2](http://norvig.com/lispy2.html) — extended version
