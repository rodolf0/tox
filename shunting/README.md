# Documentation

A library for evaluating math expressions.

## Using the library

```rust
fn main() {
  let input = "sin(0.2)^2 + cos(0.2)^2";
  let expr = ShuntingParser::parse_str(input).unwrap();
  let result = MathContext::new().eval(&expr).unwrap();
  println!("{} = {}", expr, result);
}
```

## A MathContext

`MathContext` allows keeping context across multiple invocations to parse and evaluate. You can do this via the `setvar` method.


## The tool in the crate

The crate also ship with the `tox` binary with a math repl.
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
