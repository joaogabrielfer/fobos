# Modules and imports

Every `.fob` file is a module. Imports must appear at the beginning of a file,
before declarations and executable statements. Imported modules are parsed,
type-checked, and initialized once per program.

## Relative modules

A relative path is resolved from the directory containing the importing file.
Without an alias, all public exports are brought into the current module:

```fob
import "math.fob"

echo(add(10, 20))
```

An alias keeps the exports behind a namespace:

```fob
import "math.fob" as math

echo(math::add(10, 20))
```

Relative paths must name a `.fob` file. Paths are canonicalized, so imports
such as `./math.fob` and `./utils/../math.fob` identify the same module.

## Standard modules

Standard modules use `std::` paths. A whole-module import binds the final path
segment as a namespace:

```fob
import std::foo

foo::bar()
```

The namespace can be renamed:

```fob
import std::foo as f

f::bar()
```

Individual members can be imported or renamed:

```fob
import std::foo::bar
import std::foo::baz as other

bar()
other()
```

Groups and globs import members directly into the current module:

```fob
import std::foo::{bar, baz as other}
import std::another::*
```

An alias renames the entity resolved by the path. It does not create an extra
namespace around a member. Consequently, `import std::foo::bar as run` is
called as `run()`, not `run::bar()`.

Groups and globs cannot themselves be aliased. Alias the module instead when a
namespace is wanted.

## Visibility

Only top-level declarations marked `pub` are exported:

```fob
var count: Int = 0

pub fun next(): Int =
    count = count + 1
    return count
end
```

Private declarations remain available to functions in the same module.
Imported names are not automatically re-exported.

Importing a function imports its complete overload set. Overload sets from
different modules are not merged automatically; importing two entities under
the same local name is a collision. Local top-level declarations and glob
imports follow the same hard-collision rule.

## Initialization and state

A canonical module has one persistent runtime environment. Functions capture
that exact environment, and every importer observes the same mutable state.
Dependencies initialize before their importers, and importing the same module
through multiple canonical-equivalent paths does not run it again.

Circular imports are rejected with the full import chain. Runtime and
type-checking failures retain the dependency's original file and source span,
with import context added around the diagnostic.

Module members can be read and called through `::`, but direct external
assignment is not supported:

```fob
config::value = 10 // error
```

Expose mutation through a public function instead.
