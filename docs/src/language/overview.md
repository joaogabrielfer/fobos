# Language overview

Fobos is expression-oriented inside function bodies. A file itself is a module:
its top level contains only `import`, `const`, and named `fun` declarations.
The root file must declare one `fun main(): ()` entry point, which is executed
after its imports, constants, and functions have been installed.

```fob
const GREETING: String = "hello"

fun main(): () =
    echo(GREETING <> " Fobos")
end
```

The language uses indentation for readability but block structure is delimited
by keywords and `end`; indentation is not the parser's delimiter. A newline
after a block opener makes the block explicit. A same-line body has an
implicit end.

The implemented core currently includes:

- immutable and mutable local bindings, plus typed top-level constants;
- integers, floats, booleans, strings, unit, tuples, arrays, and ranges;
- functions, lambdas, closures, piping with `.`, and overloads;
- relative and standard module imports with public functions and constants;
- `if`, `while`, and `for` expressions;
- explicit `yield` and `return` flow;
- a small type checker and the `echo`, `range`, and `push` built-ins.

See the focused pages, including [modules and imports](modules.md), for syntax
and examples. Future syntax
belongs under [Design notes](../design/roadmap.md), not this reference.
