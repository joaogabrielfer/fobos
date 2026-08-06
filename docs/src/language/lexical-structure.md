# Lexical structure

Identifiers, strings, integers, floats, booleans, keywords, operators, and
newlines are tokenized before parsing. Line and column positions are retained
for diagnostics and fixture output.

Implemented keywords include `let`, `var`, `fun`, `return`, `yield`, `if`,
`else`, `while`, `for`, `in`, `do`, `end`, `import`, `as`, and `pub`. Some
keywords are reserved for features that are not implemented yet.

The principal operators are:

```text
+  -  *  /  ==  !=  <  <=  >  >=  <>  ..=  ..<  !  .
```

`<>` combines values for output/string-like composition. `.` is the call
piping operator when used after an expression. See the [operator reference](../reference/operators.md)
for precedence and examples.
