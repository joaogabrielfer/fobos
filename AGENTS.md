# AGENTS.md

## Project and documentation

Fobos is a Lua-inspired language implemented in Rust with a lexer, parser,
AST, type checker, tree-walking interpreter, closures, overloads, named
arguments, built-ins, collections, and file modules.

Use the mdBook documentation as the detailed source of truth:

- `docs/src/language/` documents implemented language behavior.
- `docs/src/reference/language-status.md` tracks implementation status.
- `docs/src/design/` contains architecture, limitations, and future work.
- `docs/src/contributors/` documents repository structure and testing.

Keep documentation aligned when behavior or architecture changes. Do not copy
large reference sections back into this file.

## Language guardrails

- Generic arguments use `<...>`, never square brackets.
- `let value := expression` and `var value := expression` infer binding types.
- An omitted function return type with `=` means `Unit`; `:=` infers it.
- Ordinary blocks use their valid opener plus `end`, not braces.
- Returns are explicit. Do not introduce implicit tail returns.
- `value.foo(arg)` is pipe sugar for `foo(value, arg)`.
- `::` qualifies module and type paths. `.` is never namespace access.
- A trailing `?` belongs to predicate identifiers; it is not error propagation.
- Interfaces are structural; do not introduce `impl` declaration syntax.
- Named arguments are call metadata represented with `CallArgument<Expr>`,
  `CallArgument<Type>`, or `CallArgument<Value>`. Never add `Value::NamedArg`.
- Normalize named arguments into declaration order before overload selection or
  runtime binding.

## Environments and modules

Environment references are persistent and shared:

```rust
type EnvRef = Rc<RefCell<EnvFrame>>;
```

Closures and modules capture the original `EnvRef`; never copy an environment
frame to simulate capture. When temporarily replacing interpreter environment
or source-path context, restore it before applying `?` or propagating errors.

The initial module system is implemented in `src/module.rs`. See
`docs/src/language/modules.md` for syntax and `docs/src/design/modules.md` for
architecture. Preserve these invariants:

- One canonical `.fob` file has one `ModuleId`, interface, and runtime instance.
- Relative imports resolve from the importer and use canonical paths.
- Imports appear at the beginning of a module and never inside blocks.
- Dependencies compile and initialize once, dependency-first.
- Circular imports are rejected with the complete import chain.
- The checker resolves qualified members through `ModuleInterface`, not runtime
  environments.
- Runtime module and member bindings retain their origin and shared `EnvRef`.
- Only `pub` top-level declarations are exported; imports are not re-exported.
- Name collisions are hard errors. Unrelated overload sets never merge.
- Importing a function preserves its complete overload set.
- Direct assignment through `module::member` is rejected.
- Standard modules are currently file-backed under `std/`.

`--disable-checker` evaluates one function-only entry file directly. It does
not provide the module compilation pipeline or compile-time constant
evaluation.

## Built-ins and errors

Regular built-ins receive evaluated `Vec<CallArgument<Value>>`; raw built-ins
receive unevaluated `Vec<CallArgument<Expr>>`. `push` is raw because its target
must remain assignable. Use the same argument normalization and overload logic
as user functions. Never evaluate raw arguments merely to format an error.

Preserve source spans and file paths across lexer, parser, type, module, and
runtime errors. Dependency failures must retain the original diagnostic and
add import-chain context. Prefer owned rendered descriptions over complete
runtime `Value`s when error values must remain `Send` or `Sync`.

## Scope and development

Keep changes incremental and reuse existing representations, naming, and error
conventions. Prefer shared helpers, explicit phases, exhaustive matches, and
focused valid and invalid regression tests. Keep AST, checked data, and runtime
values separate.

Do not implement speculative features without a task requiring them. Current
future work is tracked in `docs/src/design/roadmap.md`; it includes package
manifests, recursive imports, re-exports, richer types, macros, and effects.

After code changes, run:

```sh
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

Module dependency fixtures belong under `fixtures/modules/`; top-level fixture
programs and generated snapshots belong under `fixtures/` and
`fixtures/expected/`. Run `cargo run -- debug generate-expected` only after the user
approves regenerating intentional snapshot changes.

After documentation changes, run:

```sh
mdbook test docs
mdbook build docs
```

Fix warnings introduced by the change.
