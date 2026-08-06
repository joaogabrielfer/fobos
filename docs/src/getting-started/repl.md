# REPL

Run the binary without a subcommand:

```console
cargo run
```

The prompt is `>>`. Expressions are parsed and evaluated one line at a time.
Press `Ctrl-C` twice to exit, or send EOF.

The REPL currently uses a synthetic `repl` path and has fewer guarantees than
file execution. For reproducible programs and diagnostics, prefer a `.fob`
file with `cargo run -- run <path>`.
