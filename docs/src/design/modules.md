# Modules (proposal / partial)

The lexer and AST already represent module paths and relative imports. The
parser accepts forms such as:

```fob
import "relative/path"
import std::math::{abs}
```

The type checker currently leaves import checking unimplemented and the
interpreter reports `NotImplemented` for imports. This page therefore records
the intended direction, not a usable module guide.

Before implementation, settle:

- how module roots are found relative to the entry file;
- whether `std::...` is a built-in namespace or a package search;
- visibility and the meaning of `pub`;
- cycles, duplicate imports, and initialization order;
- diagnostics for missing modules and missing exported names.
