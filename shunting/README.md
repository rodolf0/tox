# shunting
A shunting-yard lib to parse math expressions.
It supports prefix, infix and postfix operators, unary minus has the correct precedence,
you get whatever the standard math library has (dlopen self and tap into functions).

Try it out with the **tox** binary.

```
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

### shunting references
* http://en.wikipedia.org/wiki/Operator-precedence_grammar
* http://en.wikipedia.org/wiki/Operator-precedence_parser
* http://en.wikipedia.org/wiki/Shunting-yard_algorithm
* http://wcipeg.com/wiki/Shunting_yard_algorithm
* http://en.wikipedia.org/wiki/Operator_associativity
* http://www.haskell.org/pipermail/haskell-prime/2010-July/003229.html
* http://en.wikipedia.org/wiki/Algebraic_expression
* http://h14s.p5r.org/2014/10/shiftreduce-expression-parsing-by-douglas-gregor.html
