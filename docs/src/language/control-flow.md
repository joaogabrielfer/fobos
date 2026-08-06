# Control flow and collections

## Conditions

`if` is an expression and uses `do`/`end` blocks. An optional `else` provides
the other branch:

```fob
let label := if count > 0 do
    yield "non-empty"
end else do
    yield "empty"
end
```

## Loops

`while` repeats while its condition is true:

```fob
while count < 3 do
    count = count + 1
end
```

`for` iterates over ranges and arrays:

```fob
for i in range(3) do
    echo(i)
end
```

Ranges can be created with `range()` or `..=`/`..<`:

```fob
for i in 0..=3 do
    echo(i)
end
```

Arrays support indexing and the mutable `push` operation. Iteration, range
steps, and bounds are checked at runtime.
