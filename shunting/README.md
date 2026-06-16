# shunting

A library for evaluating math expressions via the shunting-yard algorithm. 
Handles operators, functions, variables, and random variables.

## Quick Start

```rust
use shunting::{ShuntingParser, MathContext};

let input = "sin(0.2)^2 + cos(0.2)^2";
let expr = ShuntingParser::parse_str(input).unwrap();
let result = MathContext::new().eval(&expr).unwrap();
println!("{} = {}", expr, result);
```

## MathContext

`MathContext` keeps state across multiple evaluations. Built-in variables: `pi`, `e`.

```rust
let ctx = MathContext::new();

// Set a variable
let expr = ShuntingParser::parse_str("a = sin(0.2)^2 + cos(0.2)^2").unwrap();
ctx.eval(&expr).unwrap();
let result = ctx.eval(&ShuntingParser::parse_str("a").unwrap()).unwrap();
assert_eq!(result, 1.0);
```

## Random Variables

Use `compile` to build expressions with random variables:

```rust
let expr = ShuntingParser::parse_str("normal(0, 1) + 5").unwrap();
let rv = MathContext::new().compile(&expr).unwrap();
println!("{}", rv.eval());
```

### Histogram

Sample a random variable and build a histogram:

```rust
let expr = ShuntingParser::parse_str("normal(0, 1)").unwrap();
let rv = MathContext::new().compile(&expr).unwrap();
let hist = rv.histogram::<10>(1000);
println!("{:?}", hist);
```

## MathOp

The `MathOp` enum represents a value in the expression system:

- `MathOp::Number(f64)` — a constant value
- `MathOp::RandVar` — a random variable (e.g. `normal(0, 1)`)
- `MathOp::Dynamic` — a lazily-evaluated expression (e.g. `a + b` where `a` or `b` is a variable)

## Supported Functions

| Function | Description |
|----------|-------------|
| `sin`, `cos` | Trigonometric |
| `abs` | Absolute value |
| `log` | Base-10 log |
| `ln` | Natural log |
| `atan2` | Arc tangent of two arguments |
| `max`, `min` | Min/max of all arguments |
| `nCr` | Combinations |
| `nPr` | Permutations |
| `nMCr` | Multicombinations |
| `nMPr` | n^r (ordered with replacement) |
| `rand` | Random value in [0, arg) |
| `normal(μ, σ)` | Normal distribution |
| `uniform(a, b)` | Uniform distribution |
| `lognormal(μ, σ)` | Log-normal distribution |

## Operators

Binary: `+`, `-`, `*`, `/`, `%`, `^` (or `**`)  
Unary: `-` (negation), `!` (factorial via gamma)

## The `tox` Binary

The crate ships with a math REPL:

```
$ tox
>> 4!
24
>> a = sin(0.2)^2 + cos(0.2)^2
>> a
1
>> (-3)!
NaN
>> (84 % (5/2)) !
1.32934
>> pi * 2.1^2 / cbrt(-(6+3))
-6.660512
```
