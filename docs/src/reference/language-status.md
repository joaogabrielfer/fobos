# Language status

This page is the compact source of truth for what users can rely on. Update it
when a feature changes state, and link to the relevant implementation or
proposal.

| Area | Status | Notes |
| --- | --- | --- |
| Lexer and parser | Implemented | Includes expressions, blocks, functions, calls, arrays, ranges, and import syntax |
| Interpreter | Implemented for core | Imports return `NotImplemented` at runtime |
| Type checker | Implemented for core | Import checking is still unimplemented |
| REPL | Experimental | Useful for exploration; file execution is the stable path |
| Built-ins | Small core | `echo`, `range`, and `push` |
| Modules and standard library | Proposal / partial | Syntax is represented, loading is not |
| Named arguments | Implemented | Required named parameters remain planned |
| Overloads | Implemented | Resolution uses parameter names and compatible types |
| Structs, enums, interfaces, generics | Proposal | No runtime representation yet |
| Macros and compile-time features | Proposal | Design notes only |
| Effects, streams, resource management | Proposal | See the design pages |

Code examples in the language and reference sections should be runnable unless
they are explicitly marked as a proposal.
