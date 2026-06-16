# tox

A Rust workspace of parsing and expression evaluation crates.

```rust
// Quick example: parse and evaluate a math expression
use shunting::{ShuntingParser, MathContext};

let expr = ShuntingParser::parse_str("sin(0.2)^2 + cos(0.2)^2").unwrap();
let result = MathContext::new().eval(&expr).unwrap();
println!("{}", result);
```

## Crates

| Crate | Description |
|-------|-------------|
| [lexers](lexers/README.md) | Reusable tokenizers — numbers, identifiers, math, strings, lisp, quoted text |
| [earlgrey](earlgrey/README.md) | Earley CFG parser — ambiguous grammars, EBNF, custom ASTs, all parse trees |
| [kronos](kronos/README.md) | Time sequence computation — "the 3rd Monday of the month", intervals, shifts |
| [fluxcap](fluxcap/README.md) | Natural language time parsing — "next Tuesday at 3pm", "3 weeks ago" |
| [shunting](shunting/README.md) | Math expression evaluator — operators, functions, variables, random variables |
| [lisp](lisp/README.md) | Minimal Lisp interpreter — closures, special forms, REPL |
| [lox](lox/README.md) | C-style scripting language — variables, functions, closures, classes |

## Quick Examples

**Parse natural language time expressions:**

```rust
use fluxcap::TimeMachine;

let tm = TimeMachine::new();
let results = tm.eval("next Tuesday", None).unwrap();
for r in results {
    println!("{:?}", r);
}
```

**Build a grammar and parse custom syntax:**

```rust
use earlgrey::{ParserBuilder, EbnfParser};

let grammar = EbnfParser::new().parse(r#"
    expr := 'x' '+' 'y';
"#).unwrap();

let parser = ParserBuilder::new(grammar, "expr")
    .terminal("x", |_| Some(1))
    .terminal("y", |_| Some(2))
    .build()
    .unwrap();

for tree in parser.parse_all(&mut vec!["x", "+", "y"].into_iter()) {
    println!("{:?}", tree);
}
```

**Compute time sequences:**

```rust
use kronos::{TimeSeqSpec, TimeSpan};
use time::macros::datetime;

let reftime = datetime!(2024-06-01 12:00 UTC);
let mondays = TimeSeqSpec::weekday(1); // Monday
for day in mondays.future(reftime).take(3) {
    println!("{:?}", day);
}
```

## Running a Crate's REPL or Binary

Most crates ship with a runnable example or binary:

```bash
cargo run -p lisp      # Lisp REPL
cargo run -p lox       # Lox interpreter
cargo run -p shunting  # Math expression REPL
cargo run -p fluxcap   # Time expression parser
```

See each crate's README for detailed documentation.
