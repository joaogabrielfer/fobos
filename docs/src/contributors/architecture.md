# Implementation architecture

Fobos currently follows a direct source-to-interpreter pipeline:

```text
source text -> lexer -> tokens -> parser -> AST -> type checker -> interpreter
```

The modules map to that pipeline:

- `src/source.rs` defines source positions and spans.
- `src/lexer.rs` tokenizes source and reports lexical errors.
- `src/parser.rs` builds the structures in `src/ast.rs`.
- `src/typechecker/` tracks environments and checks expressions/statements.
- `src/interpreter/` evaluates the AST, manages lexical environments, values,
  control-flow signals, and built-ins.
- `src/diagnostic.rs` and `src/errors.rs` render source-aware failures.
- `src/main.rs` owns the Clap CLI and the REPL.
- `src/dump.rs` regenerates fixture expectations.

The AST already contains import nodes, but import behavior crosses all three
semantic layers and is intentionally unfinished. New language features should
be documented with their lexer/parser, type-checker, interpreter, and fixture
impact rather than added to a single checklist.
