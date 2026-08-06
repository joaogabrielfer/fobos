# CLI reference

The binary is invoked through Cargo during development: `cargo run -- <command>`.

| Command | Purpose |
| --- | --- |
| *(none)* or `repl` | Start the REPL |
| `run <path>` | Compile the module graph, type-check it, and interpret the entry `.fob` file |
| `run <path> --no-check` | Interpret without type checking |
| `debug tokens <path>` | Print tokens, including source spans |
| `debug tokens <path> --kinds` | Print only token kinds |
| `debug ast <path>` | Print the parsed AST |
| `debug generate-expected` | Regenerate outputs under `fixtures/expected/` |

Errors are rendered with the source path, line, column, and a source excerpt
when the span can be resolved.

`--no-check` (also available as `--disable-checker`) runs a single parsed
function-only entry file directly. It does not provide module loading or
compile-time constant evaluation, so use the normal `run` command for any
program with imports or `const` declarations.
