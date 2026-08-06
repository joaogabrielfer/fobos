# Diagnostics

Lexer, parser, type-checker, and runtime errors carry a source path and span.
When the source can be read, Fobos prints the line, column, excerpt, and a
caret-style location:

```text
type error: mismatched types, expected Int but got String
  --> examples/main.fob:3:5:
   |
 3 | echo("wrong" + 1)
   |     ^
```

Use `debug tokens` and `debug ast` to isolate whether a failure occurs during lexing or
parsing. Use `--no-check` (or its `--disable-checker` alias) only to distinguish type-checker failures from
interpreter behavior; it does not make unsupported runtime features work.

Fixture comparisons normalize embedded source paths, so snapshots remain
portable across checkouts.

Module diagnostics include the dependency's original file and span. For a
cycle or a failure several imports deep, the loader adds each importing module
as context and reports the complete import chain.
