# Types and compile-time features

The current checker supports primitive, tuple, array, range, function, `Any`,
and temporary type-variable representations. Typed top-level `const`
declarations are evaluated at compile time. They are intentionally a small,
value-only layer: calls and other runtime constructs are rejected, while
references to local or imported public constants are supported.

The following remain design goals, not supported syntax:

- nominal wrapper types and transparent aliases;
- structs, constructors, tags, enums, and algebraic data types;
- pattern matching and custom patterns;
- generics and interfaces;
- type-level constant parameters, such as fixed-size arrays;
- macros, syntax types, templates, interpolation, and hygienic expansion.

When these are implemented, their documentation should be split into a user
reference and a contributor design page. In particular, generic syntax should
not be documented as working merely because an example demonstrates the
desired form.
