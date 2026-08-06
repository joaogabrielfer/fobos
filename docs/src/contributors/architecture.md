# Implementation architecture

For a file entry point, Fobos follows a module-aware source-to-interpreter
pipeline:

```text
entry file -> compiler session -> dependency graph -> checked modules
          -> runtime module initialization -> interpreter
```

The modules map to that pipeline:

- `src/source.rs` defines source positions and spans.
- `src/lexer.rs` tokenizes source and reports lexical errors.
- `src/parser.rs` builds the structures in `src/ast.rs`.
- `src/module.rs` resolves canonical module identities, loads and caches the
  dependency graph, exposes compile-time interfaces, rejects cycles, and owns
  runtime module instances.
- `src/typechecker/` tracks environments and checks expressions/statements.
- `src/interpreter/` evaluates the AST, manages lexical environments, values,
  control-flow signals, and built-ins.
- `src/diagnostic.rs` and `src/errors.rs` render source-aware failures.
- `src/main.rs` owns the Clap CLI; `src/repl.rs` owns the stateful interactive session.
- `src/dump.rs` regenerates fixture expectations.

The type checker stores module namespaces and imported members as explicit
symbol kinds. It resolves qualified paths against `ModuleInterface`, never a
runtime environment. At runtime, imported members remain aliases into a
module's persistent `EnvRef`; module frames are not copied into importers.

When the interpreter temporarily switches to a module environment, it restores
the previous environment and source path before propagating an error. New
language features should still document their lexer/parser, type-checker,
interpreter, module, and fixture impact rather than being added to a single
checklist.
