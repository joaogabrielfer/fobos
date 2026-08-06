# Fobos

Fobos is an experimental, interpreted programming language implemented in
Rust. The repository contains the lexer, parser, type checker, interpreter,
CLI, examples, and fixture-based tests.

The user documentation now lives in the [Fobos wiki](docs/src/index.md). Start
with the [getting started guide](docs/src/getting-started/install-and-run.md),
then use the [language reference](docs/src/language/overview.md). The
[language status page](docs/src/reference/language-status.md) distinguishes
implemented behavior from proposals.

## Quickstart

Run a source file:

```console
cargo run -- run examples/fib.fob
```

Inspect tokens or the parsed AST:

```console
cargo run -- tokens examples/fib.fob
cargo run -- ast examples/fib.fob
```

Run the REPL with no subcommand. It is currently experimental.

## Development

```console
cargo test
cargo run -- generate-expected
```

See [testing](docs/src/contributors/testing.md) for the fixture workflow and
[architecture](docs/src/contributors/architecture.md) for the implementation
boundaries.
