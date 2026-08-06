# Language status

This page is the compact source of truth for what users can rely on. Update it
when a feature changes state, and link to the relevant implementation or
proposal.

| Area | Status | Notes |
| --- | --- | --- |
| Lexer and parser | Implemented | Includes expressions, blocks, functions, calls, arrays, ranges, and import syntax |
| Interpreter | Implemented for core | Dependencies initialize once with persistent shared module environments |
| Type checker | Implemented for core | Imports and qualified member paths are checked through module interfaces |
| REPL | Implemented for core | Persistent typed session with multiline entries; imports require file execution |
| Built-ins | Small core | `echo`, `range`, and `push` |
| Modules | Implemented initial version | Relative and `std::` imports, aliases, groups, globs, visibility, caching, and cycle rejection |
| Standard library | Minimal | File-backed `std::` resolution exists; the bundled module set is intentionally small |
| Named arguments | Implemented | Required named parameters remain planned |
| Overloads | Implemented | Resolution uses parameter names and compatible types |
| Structs, enums, interfaces, generics | Proposal | No runtime representation yet |
| Macros and compile-time features | Proposal | Design notes only |
| Effects, streams, resource management | Proposal | See the design pages |

Code examples in the language and reference sections should be runnable unless
they are explicitly marked as a proposal.
