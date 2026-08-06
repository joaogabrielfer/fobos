# Operators

| Operator | Meaning |
| --- | --- |
| `+`, `-`, `*`, `/` | Arithmetic |
| `==`, `!=` | Equality |
| `<`, `<=`, `>`, `>=` | Ordering |
| `<>` | Combine values for output/string-like composition |
| `..=`, `..<` | Inclusive and exclusive ranges |
| `!`, unary `-` | Boolean negation and numeric negation |
| `.` | Piped/function call syntax |
| `::` | Qualified module path separator |

Binary precedence runs from ranges and comparisons through combination,
addition/subtraction, and multiplication/division. Parentheses can always be
used to make grouping explicit.

`::` forms a qualified path such as `math::add`; it does not participate in
binary precedence. `.` remains function-pipe syntax and is never module member
access.

Word-form boolean operators, bitwise operators, operator sections, and a plain
`..` range are proposals, not part of the current reference language.
