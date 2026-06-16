# earlgrey

An **Earley parser** for context-free grammars. Handles ambiguous grammars, 
extracts all parse trees, and supports EBNF-style grammar definitions.

## Quick Start — S-expressions

The easiest way to get started is `ParserBuilder::for_sexpr`, which builds a 
parser that returns s-expression trees:

```rust
use earlgrey::ParserBuilder;

let grammar = r#"
    expr   := expr ('+'|'-') term | term ;
    term   := term ('*'|'/') factor | factor ;
    factor := '-' factor | power ;
    power  := ufact '^' factor | ufact ;
    ufact  := ufact '!' | group ;
    group  := num | '(' expr ')' ;
"#;

let parser = earlgrey::ParserBuilder::for_sexpr(grammar, "expr")
    .terminal("num", |n| Some(earlgrey::Sexpr::Atom(n.to_string())))
    .build()?;

let tokens = lexers::StringTokenizer::from("3 + 4 * 2")
    .split_on(["+", "-", "*", "/", "^", "!", "(", ")"], false);

for tree in parser.parse_sexpr(tokens)? {
    println!("{}", tree.print());
}
```

## ParserBuilder — Custom AST

For full control, use `ParserBuilder` to map grammar rules to your own AST type:

```rust
#[derive(Clone, Debug)]
enum Expr {
    Num(f64),
    Neg(Box<Expr>),
    Fact(Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

let grammar = r#"
    expr   := expr ('+'|'-') term | term ;
    term   := term ('*'|'/') factor | factor ;
    factor := '-' factor | power ;
    power  := ufact '^' factor | ufact ;
    ufact  := ufact '!' | group ;
    group  := num | '(' expr ')' ;
"#;

let parser = earlgrey::ParserBuilder::new(grammar, "expr")
    .terminal("num", |n| n.parse::<f64>().ok().map(Expr::Num))
    .action2("expr -> expr + term", |l, r| Expr::Add(Box::new(l), Box::new(r)))
    .action2("expr -> expr - term", |l, r| Expr::Sub(Box::new(l), Box::new(r)))
    .action2("term -> term * factor", |l, r| Expr::Mul(Box::new(l), Box::new(r)))
    .action2("term -> term / factor", |l, r| Expr::Div(Box::new(l), Box::new(r)))
    .action2("power -> ufact ^ factor", |b, e| Expr::Pow(Box::new(b), Box::new(e)))
    .action1("factor -> - factor", |v| Expr::Neg(Box::new(v)))
    .action1("ufact -> ufact !", |v| Expr::Fact(Box::new(v)))
    .build()?;

let tokens = lexers::StringTokenizer::from("3 + 4 * 2")
    .split_on(["+", "-", "*", "/", "^", "!", "(", ")"], false);

let ast = parser.parse(tokens)?;
```

## EBNF Grammar Syntax

Grammars are written in a lightweight EBNF dialect:

| Construct | Syntax | Meaning |
|-----------|--------|---------|
| Alternation | `a \| b` | Match `a` or `b` |
| Grouping | `(a \| b)` | Group alternatives |
| Optional | `[a]` | Match `a` zero or one times |
| Repetition | `{a}` | Match `a` zero or more times |
| Tagging | `{a} @tag` | Name a repetition for easier action matching |

## Features

* `debug` — Enable verbose printing of internal parser state.

```toml
[dependencies]
earlgrey = { version = "0.5", features = ["debug"] }
```
