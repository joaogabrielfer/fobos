# Roadmap and design notes

The former README contained a broad language roadmap. It is preserved here as
design material, separate from the working-language reference.

Near-term work should make the current implementation coherent before adding
large surface areas:

1. finish module loading and import checking;
2. make fixture snapshots portable across checkout paths;
3. stabilize the REPL and standardize the runnable examples;
4. define the type representation needed for generics and user-defined types;
5. add a standard library boundary before adding macros or effects.

Later proposals include required named parameters, boolean identifier suffixes,
word-form boolean operators, operator sections, spread/rest syntax, pattern
matching, structs, tags, enums, interfaces, generics, macros, templates,
resource management, C FFI, streams, and effect handlers.

Each proposal should document syntax, semantics, interaction with the type
checker/runtime, an acceptance example, and the point at which it becomes
implemented. Until then, proposal examples must not be mixed into the user
reference.
