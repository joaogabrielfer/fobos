# Fobos

Fobos is an experimental, interpreted language with expression-oriented
blocks, explicit `yield` and `return`, first-class functions, named arguments,
overloads, file modules, and a small built-in runtime. The implementation is a
Rust crate, with executable examples in `examples/` and behavioral fixtures in
`fixtures/`.

This wiki has two audiences:

- **Language users** can start with [install and run](getting-started/install-and-run.md)
  and then follow the [language guide](language/overview.md).
- **Contributors** can read the [architecture](contributors/architecture.md)
  and [testing workflow](contributors/testing.md).

Fobos is not yet a stable language. Pages describe support explicitly. A page
marked as a proposal records intended design and should not be used as a
working-language reference.

## Current shape

Source is read by the lexer, parsed into an AST, checked, and evaluated by the
interpreter. The CLI exposes execution plus token and AST inspection. The
current runtime provides `echo`, `range`, and `push`.

File execution resolves and canonicalizes imports, type-checks dependency
interfaces, and initializes each module once with a persistent shared
environment. See [modules and imports](language/modules.md). The REPL is useful
for experiments, but it does not run the file-module pipeline and is not yet a
stable interface.
