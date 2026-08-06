# Built-ins

Built-ins are loaded into every interpreter and type-checker environment.

| Built-in | Signatures | Behavior |
| --- | --- | --- |
| `echo` | `echo(value: Any): Unit` | Writes one value followed by a newline |
| `range` | `range(end)`, `range(start, end)`, `range(start, end, step)` | Creates an integer range; `step` cannot be zero |
| `push` | `push(target: Arr<Any>, value: Any): Unit` | Mutates a mutable array; method-style `array.push(value)` is supported |

`push` takes a place rather than an evaluated value for its first argument, so
the target must be a mutable binding or assignable indexed location. Calling it
on an immutable binding is a runtime error.

Standard modules are distinct from built-ins and are loaded from the bundled
`std/` tree through the normal module interface. That tree is currently
minimal; `Result`, `Option`, and I/O modules are not available yet.
