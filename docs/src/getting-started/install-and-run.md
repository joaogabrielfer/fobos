# Install and run

## Prerequisites

Install a recent Rust toolchain with Cargo. The repository uses Rust edition
2024.

Clone the repository and run the test suite:

```console
cargo test
```

## Run a program

Fobos source files use the `.fob` extension:

```console
cargo run -- run examples/fib.fob
```

The `run` command performs lexing, parsing, type checking, and interpretation.
Type checking can be disabled for experiments:

```console
cargo run -- run examples/foo.fob --disable-checker
```

The repository's `examples/` directory is the best place to find complete
programs. Some examples are intentionally ahead of the implementation and may
be useful as design sketches; check [language status](../reference/language-status.md)
when an example does not run.

## Inspect a program

Use `tokens` to inspect the lexer output and `ast` to inspect the parser output:

```console
cargo run -- tokens examples/fib.fob
cargo run -- tokens examples/fib.fob --kinds
cargo run -- ast examples/fib.fob
```
