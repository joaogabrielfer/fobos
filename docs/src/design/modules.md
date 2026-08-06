# Module architecture and limits

The initial module system is implemented in `src/module.rs`. One canonical
`.fob` file corresponds to one `ModuleId`, one compile-time interface, and one
persistent runtime instance.

## Compilation

`CompilerSession` canonicalizes the entry path and recursively discovers
imports. It maintains a compiled-module cache plus a loading set and stack.
Encountering a module already on the loading stack produces a circular-import
diagnostic with the complete chain.

Dependencies are compiled before their importers. Each `CompiledModule`
contains its checked program, resolved imports, source path, and
`ModuleInterface`. Surface syntax is lowered to two operations:

- bind a module under a local namespace;
- bind one exported member under a local name.

Groups, globs, and unaliased relative imports expand into member bindings.
The checker resolves namespaces through dependency interfaces, preserves
complete function overload sets, and rejects collisions instead of merging
unrelated imports.

File module IDs use canonical paths. Standard module IDs retain their
`std::...` segments and currently map to `.fob` files below the repository's
`std/` directory. Resolution tries the longest valid standard-module prefix,
allowing the final segment to resolve as an exported member.

## Runtime

`RuntimeModules` initializes the graph dependency-first and caches
`ModuleValue` instances by `ModuleId`. Each instance owns a persistent
`Rc<RefCell<EnvFrame>>`. Module functions capture this exact reference.

Runtime environment bindings distinguish local values, module namespaces, and
imported-member aliases. Looking up an imported member reads through the
exporting module's environment, which preserves shared mutable state without
copying frames. External assignment through an imported or qualified member
is rejected.

Environment and diagnostic-path switches during initialization are restored
before an evaluation error is propagated.

## Deliberate limits

The first implementation does not include:

- circular imports or partially initialized modules;
- imports inside blocks or after top-level declarations;
- package manifests, directory modules, or configurable search paths;
- native standard-module registration;
- explicit re-exports or `pub import`;
- direct external mutation through `module::member`;
- merging overload sets imported from unrelated modules;
- module reloading;
- imported user-defined types, which depend on the future type-declaration
  representation.

The user-facing syntax and behavior are documented under
[modules and imports](../language/modules.md).
