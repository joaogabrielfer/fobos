# Functions and calls

Named functions use `fun`, parameters, an optional return annotation, and a
block body. At module scope they are declarations; local function declarations
are also supported inside function bodies:

```fob
fun add(x: Int, y: Int): Int =
    return x + y
end
```

Every root program has one entry declaration with exactly this shape:

```fob
fun main(): () =
    echo(add(10, 20))
end
```

`main` cannot take parameters, be generic, or return another type. Imported
modules do not declare an entry point.

Functions are values. Lambdas use `->`:

```fob
let double := x -> x * 2
let add := (x, y) -> x + y
```

Calls can be written normally or piped with a dot. The piped value becomes the
first argument:

```fob
fun add(x: Int, y: Int): Int =
    return x + y
end

let value := add(10, 5)
let same_value := (10).add(5)
```

Function declarations may overload by parameter shape and type. Arguments can
be positional or named, and positional arguments cannot follow a named one:

```fob
fun move(x: Int, y: Int): Unit =
    echo(x <> y)
end

move(10, y = 20)
```

The parser accepts the `named` parameter design described in the old README,
but required named parameters are not yet implemented. Treat that syntax as a
proposal until it appears in the status page as implemented.
