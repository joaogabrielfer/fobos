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
It also loads imported modules recursively and initializes dependencies before
the entry module. See [modules and imports](../language/modules.md).
Type checking can be disabled for experiments:

```console
cargo run -- run examples/foo.fob --disable-checker
```

The module compiler is part of the normal checked file pipeline. Programs that
use imports should not use `--disable-checker`.

The repository's `examples/` directory is the best place to find complete
programs. Some examples are intentionally ahead of the implementation and may
be useful as design sketches; check [language status](../reference/language-status.md)
when an example does not run.

## Inspect a program

Use `debug tokens` to inspect the lexer output and `debug ast` to inspect the parser output:

```console
cargo run -- debug tokens examples/fib.fob
cargo run -- debug tokens examples/fib.fob --kinds
cargo run -- debug ast examples/fib.fob
```
