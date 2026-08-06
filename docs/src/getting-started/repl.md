# REPL

Start the REPL explicitly, or omit the subcommand:

```console
cargo run -- repl
```

The REPL retains declarations and functions between entries, prints a
non-`()` expression result, and accepts multiline blocks with a continuation
prompt. It type-checks entries by default; use `repl --no-check` to bypass that
step.

Use `:help` to list `:clear`, `:reset`, and `:quit` (or `:q`). `Ctrl-C` clears
an unfinished entry; pressing it twice at an empty prompt exits. Command
history is stored in the platform state directory.

Diagnostics render against the in-memory REPL source as `<repl>`. File imports
are intentionally unavailable in the REPL because imports require a canonical
module file and the dependency pipeline. Use `cargo run -- run <path>` for a
program that imports modules.
