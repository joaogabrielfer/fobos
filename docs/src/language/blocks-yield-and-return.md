# Blocks, `yield`, and `return`

Blocks are opened by `=`, `->`, or `do` in the constructs that support them,
and closed by `end` when they span multiple lines:

```fob
let total :=
    let a := 10
    let b := 20
    yield a + b
end
```

`yield` evaluates a block expression. `return` exits the nearest function.
They are deliberately separate so a nested block can produce a value without
returning from its enclosing function.

```fob
fun add_one(x: Int): Int =
    let result :=
        yield x + 1
    end
    return result
end
```

The runtime models these as control-flow signals. A yield in statement
position can bubble through an enclosing block; a yield captured as a value is
the value of that block. This behavior is implemented for the current
interpreter, but collection/effect-handler semantics are still future design.
