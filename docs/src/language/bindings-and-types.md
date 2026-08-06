# Bindings and types

Use `let` for an immutable binding and `var` for a mutable binding. `:=`
allows inference; an explicit annotation follows the name:

```fob
let answer := 42
var count: Int = 0
let label: String = "items"
```

Bindings can be assigned only when mutable:

```fob
count = count + 1
```

The implemented type vocabulary is `Int`, `Float`, `Bool`, `String`, `Unit`,
`Any`, `Range`, tuples such as `(Int, String)`, arrays such as `Arr<Int>`, and
function types such as `(Int, Int) -> Int`. Inferred annotations are represented
as `Any` while the type system is still evolving.

Tuple and array literals are available:

```fob
let pair := (10, "ten")
var values: Arr<Int> = [1, 2, 3]
echo(values[0])
```

The type checker reports mismatched values, invalid calls, invalid assignments,
and invalid indexing with source locations. Generic types, structs, enums,
interfaces, and pattern types are design work; see [types and compile-time
features](../design/types-and-compile-time.md).
