# AGENTS.md

## Project overview

Fobos is a small Lua-inspired programming language implemented in Rust. It currently uses a tree-walking interpreter and includes:

* Lexer and parser
* AST
* Static type checker
* Runtime interpreter
* Persistent lexical environments using `Rc<RefCell<EnvFrame>>`
* Closures
* Function overloads
* Named call arguments
* Builtin functions
* Arrays, tuples, ranges, loops, and higher-order functions
* Explicit `Result`-style error handling

The long-term goal may include bytecode compilation, macros, generators, richer types, and a Rust-like module/package system, but changes should remain incremental.

## Language syntax

Generics use angle brackets:

```fob
fun <T> identity(value: T): T =
    return value
end

type Parser<T>: alias = ...
```

Bindings:

```fob
let value := expression
var value := expression

let value: Int = expression
```

Functions:

```fob
fun inferred_return() :=
    return 10
end

fun explicit_return(): Int =
    return 10
end
```

An omitted return type with `=` means `Unit`. `:=` means inferred return type.

As in:

```fob
fun unit_return() =
    echo("no return function")
end
```

Blocks use `do`/`end` or another valid block opener. Fobos does not use braces for ordinary blocks.

Returns are explicit. Do not introduce implicit tail returns.

## Important language semantics

Dot syntax is pipe/function-call sugar, not method or module access:

```fob
value.foo(arg)
```

means:

```fob
foo(value, arg)
```

Module and type paths use `::`:

```fob
math::add(1, 2)
Result::Ok(value)
```

Boolean predicate functions conventionally end in `?`, such as:

```fob
empty?()
```

`?` is part of the identifier and is not an error-propagation operator.

Named arguments are call metadata, not runtime values. Use structures such as:

```rust
CallArgument<Expr>
CallArgument<Type>
CallArgument<Value>
```

Do not add `Value::NamedArg`.

Named arguments must be normalized into parameter declaration order before overload matching or runtime binding.

## Runtime environments

Environment references are persistent and shared:

```rust
type EnvRef = Rc<RefCell<EnvFrame>>;
```

Closures and modules must capture the original `EnvRef`. Do not copy complete environment frames to simulate closure or module capture.

Whenever interpreter code temporarily replaces `self.env`, restore it even if evaluation fails:

```rust
let previous_env = self.env.clone();
self.env = temporary_env;

let result = self.eval_something();

self.env = previous_env;

let value = result?;
```

Do not use `?` before restoring the previous environment.

## Modules

The initial module system should support:

```fob
import "foo.fob"
import "foo.fob" as foo

import std::math
import std::math as m
import std::math::add
import std::math::add as plus
import std::math::{add, sub}
import std::math::*
```

Rules:

* One `.fob` file is one module.
* Relative paths resolve from the importing file and are canonicalized.
* A module is parsed, type-checked, and initialized once.
* Circular imports are rejected initially.
* Imports are top-level declarations.
* `import "foo.fob"` imports public members into the current module scope.
* `import "foo.fob" as foo` binds a module namespace.
* `as` renames the resolved entity.
* Imported overloads remain one overload set.
* Unrelated imported overload sets must not merge automatically.
* Name collisions are hard errors.
* Imported names are not automatically re-exported.
* Module environments are persistent and shared, not copied.

Internally, resolved imports should reduce to module bindings or member bindings.

## Builtins

Regular builtins receive evaluated arguments:

```rust
Vec<CallArgument<Value>>
```

Raw builtins receive unevaluated expressions:

```rust
Vec<CallArgument<Expr>>
```

For example, `push` is raw because its target must remain an assignable expression.

Builtins should use the same named-argument normalization and overload-selection logic as user functions.

Never evaluate raw arguments merely to produce an error message, because doing so may cause side effects.

## Type system direction

Current and planned type syntax uses `<...>`:

```fob
Arr<Int>
Result<Int, Error>
Parser<Token>
```

Do not use square brackets for generic arguments. Square brackets are reserved for possible future array-related type syntax.

Interfaces are structural. Fobos has no `impl` declaration syntax. A type satisfies an interface when the required functions or operations can be resolved structurally.

Do not implement speculative features unless required by the current task. Notably, these remain future work unless already present in the repository:

* `named`-only parameters
* `var` parameters in function signatures
* Higher-kinded types
* General laziness
* `use` continuation syntax
* Full macro hygiene
* Recursive module imports
* Rust-style package manifests

## Error handling

Preserve source spans and file paths in lexer, parser, type, module, and runtime errors.

When reporting errors from imported modules, preserve the original diagnostic and add import-chain context instead of replacing it with a generic “failed to import” message.

Avoid storing complete runtime `Value`s in errors when that would make errors non-`Send` or non-`Sync`. Store rendered, owned descriptions where appropriate.

## Development guidelines

Before changing architecture, inspect existing representations and reuse established naming and error conventions.

Prefer:

* Small focused changes
* Shared normalization/resolution helpers
* Explicit compiler phases
* Exhaustive matches
* Tests for regressions and invalid programs
* Clear separation between AST, checked representation, and runtime values

Avoid:

* Duplicating argument matching between user functions and builtins
* Copying module or closure environments
* Introducing syntax inconsistent with current Fobos conventions
* Treating `.` as method or namespace access
* Silently changing language semantics while fixing implementation bugs

After changes, run:

```sh
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

Running `cargo test` will run the fixtures and compare them with the expected result. After a feature is implemented and verified working correctly, you may run `cargo run -- generate-expected` with the user aproval to regenerate the fixtures.

Fix warnings introduced by the change unless an existing project convention explicitly allows them.
