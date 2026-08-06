# CLI reference

The binary is invoked through Cargo during development: `cargo run -- <command>`.

| Command | Purpose |
| --- | --- |
| *(none)* | Start the experimental REPL |
| `run <path>` | Compile the module graph, type-check it, and interpret the entry `.fob` file |
| `run <path> --disable-checker` | Interpret without type checking |
| `tokens <path>` | Print tokens, including source spans |
| `tokens <path> --kinds` | Print only token kinds |
| `ast <path>` | Print the parsed AST |
| `generate-expected` | Regenerate outputs under `fixtures/expected/` |

Errors are rendered with the source path, line, column, and a source excerpt
when the span can be resolved.

`--disable-checker` runs a single parsed file directly and therefore does not
provide module loading. Use the normal `run` command for programs with imports.
