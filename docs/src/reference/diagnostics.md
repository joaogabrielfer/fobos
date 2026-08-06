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

Use `tokens` and `ast` to isolate whether a failure occurs during lexing or
parsing. Use `--disable-checker` only to distinguish type-checker failures from
interpreter behavior; it does not make unsupported runtime features work.

Some fixture snapshots currently embed absolute checkout paths. This makes
the snapshot tests non-portable across clones and is tracked as a testing
follow-up.

Module diagnostics include the dependency's original file and span. For a
cycle or a failure several imports deep, the loader adds each importing module
as context and reports the complete import chain.
