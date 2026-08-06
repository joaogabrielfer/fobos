# Testing and fixtures

The test suite discovers `.fob` programs under `fixtures/` and compares lexer,
parser, and interpreter output with corresponding files under
`fixtures/expected/`:

```console
cargo test
```

Regenerate expectations after an intentional language-output change:

```console
cargo run -- debug generate-expected
```

Expectation files are grouped by phase:

- `.tokens` checks lexer output;
- `.ast` checks parser output;
- `.eval` checks interpreter output and errors.

When adding a feature, add the smallest fixture that proves its syntax and
semantics, then update the expected output in the same change. Keep source
fixtures as `.fob` files and avoid hand-editing generated output unless the
change is a deliberate snapshot update.

Module fixtures keep dependency files under `fixtures/modules/` so the
top-level fixture scanner does not execute dependencies as entry programs.
Focused Rust tests additionally verify all supported import forms, shared
declarative constants across importers, private-export errors, collisions,
invalid top-level items, external assignment rejection, and cycle diagnostics.

Diagnostic snapshots normalize source paths to `$FIXTURE_PATH`, so fixture
output is portable across checkout locations. Keep generated snapshots in that
form rather than replacing the marker with a local absolute path.
