# Types and compile-time features (proposal)

The current checker supports primitive, tuple, array, range, function, `Any`,
and temporary type-variable representations. The following are design goals,
not supported syntax:

- nominal wrapper types and transparent aliases;
- structs, constructors, tags, enums, and algebraic data types;
- pattern matching and custom patterns;
- generics and interfaces;
- macros, syntax types, templates, interpolation, and hygienic expansion.

When these are implemented, their documentation should be split into a user
reference and a contributor design page. In particular, generic syntax should
not be documented as working merely because an example demonstrates the
desired form.
