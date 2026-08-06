# Testing and fixtures

The test suite discovers `.fob` programs under `fixtures/` and compares lexer,
parser, and interpreter output with corresponding files under
`fixtures/expected/`:

```console
cargo test
```

Regenerate expectations after an intentional language-output change:

```console
cargo run -- generate-expected
```

Expectation files are grouped by phase:

- `.tokens` checks lexer output;
- `.ast` checks parser output;
- `.eval` checks interpreter output and errors.

When adding a feature, add the smallest fixture that proves its syntax and
semantics, then update the expected output in the same change. Keep source
fixtures as `.fob` files and avoid hand-editing generated output unless the
change is a deliberate snapshot update.

There is a current portability issue: diagnostic snapshots include absolute
paths from the checkout that generated them. Tests can fail after cloning to a
different directory even when behavior is unchanged. Normalize paths before
treating the fixture suite as a portable CI gate.
