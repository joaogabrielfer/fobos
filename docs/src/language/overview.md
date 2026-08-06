# Language overview

Fobos is expression-oriented. A program is a sequence of statements, and
blocks can themselves be expressions. Statements include bindings, function
declarations, assignments, calls, `yield`, and `return`.

```fob
let greeting := "hello"
echo(greeting <> " Fobos")
```

The language uses indentation for readability but block structure is delimited
by keywords and `end`; indentation is not the parser's delimiter. A newline
after a block opener makes the block explicit. A same-line body has an
implicit end.

The implemented core currently includes:

- immutable and mutable bindings;
- integers, floats, booleans, strings, unit, tuples, arrays, and ranges;
- functions, lambdas, closures, piping with `.`, and overloads;
- `if`, `while`, and `for` expressions;
- explicit `yield` and `return` flow;
- a small type checker and the `echo`, `range`, and `push` built-ins.

See the focused pages in this section for syntax and examples. Future syntax
belongs under [Design notes](../design/roadmap.md), not this reference.
