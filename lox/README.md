# lox

A Rust implementation of the Lox programming language from the book [Crafting Interpreters](https://craftinginterpreters.com/). 
A dynamically typed scripting language with a familiar C-style syntax.

## The `lox` Binary

The crate ships with a command-line interpreter that can run scripts or act as a REPL:

```
# Run a script
$ cargo run -p lox -- script.lox

# Start the REPL
$ cargo run -p lox
~> print 1 + 2;
3
```

## Language Features

### Variables

```lox
var x = 10;
var name = "hello";
```

### Functions

```lox
fun add(a, b) {
  return a + b;
}
print add(3, 4);
```

### Closures

```lox
fun makeCounter() {
  var count = 0;
  fun counter() {
    count = count + 1;
    return count;
  }
  return counter;
}
var c = makeCounter();
print c(); // 1
print c(); // 2
```

### Control Flow

```lox
if (x > 0) {
  print "positive";
} else {
  print "not positive";
}

while (x < 10) {
  x = x + 1;
}

for (var i = 0; i < 5; i = i + 1) {
  print i;
}
```

### Classes

```lox
class Cake {
  init(flavor) {
    this.flavor = flavor;
  }
  taste() {
    print "The " + this.flavor + " cake is delicious!";
  }
}
var c = Cake("chocolate");
c.taste();
```

## Built-in Functions

| Function | Description |
|----------|-------------|
| `clock()` | Returns current Unix time in nanoseconds |

## Operators

Arithmetic: `+`, `-`, `*`, `/`  
Comparison: `<`, `<=`, `>`, `>=`, `==`, `!=`  
Logical: `and`, `or`, `!`  
Assignment: `=`

## Implementation Notes

The interpreter is built with a recursive descent parser, static variable resolver, and tree-walk evaluator. The scanner uses the `lexers` crate for tokenization.
