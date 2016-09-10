# a collection of toy parsers and libraries written in rust

## lexers
Random tokenizers for math expressions, splitting text, parsing lisp-like stuff, etc.

## shunting
A shunting-yard lib to parse math expressions.
It supports prefix, infix and postfix operators, unary minus has the correct precedence,
you get whatever the standard math library has (dlopen self and tap into functions).

## earlgrey
A library for parsing using the *earley* algorithm.
You can extract all parse trees when the grammar is ambiguous.

## lisp
A lisp-like interpreter following norvig's lispy notes.

## kronos
A library (WIP) to work with time expressions.
