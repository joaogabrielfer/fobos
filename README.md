# blorp

## Quickstart
To run a program:
```console
$ cargo run -- run <file>
```

## Testing
Use `cargo run -- generate-expected` to compile all .blorp programs in tests/ to their tokens and save them as .blorp.expected.

Use `cargo test` to run the unit tests and ensure the result is the expected

## Langage syntax

### Variable definition
```blorp
let foo := 10    // immutable
var bar := "bar" // mutable
```
- The syntax is: `[let | var] <name> : <type> = <value>`
- If no type is provided, it is inferred
- Variables in blorp use camel_case syntax

### Blocks
Blocks in blorp are not declared with curly braces. Instead, they use `end` to denote the end of blocks and `=`, `->`,  `do`, `in` to indicate the opening of is, when succeded by a new line.
```blorp
let my_var =
    var (a, b) = (10, 20) // assignment with tuples
    return a + b          // explicit return
end
```

- `=` is used in variable and function definitions
- `->` is used in anonymous functions
- `do` is used in blocks like `while` and `for` loops and `if` staments
- `in` is used in `match` statements

if there is not a new line after the tokens, there is a implicit `end` at the end of the line (and the values are returned implicitly):
```blorp
let my_var = 10

// is the same as
let my_var = return 10 end
```

### Function declaration
Functions are defined with the `fun` keyword succeded with the name and signature of the function, with the return type following the variable definition conventino of being between the `:` and the `=`
```blorp
fun add(x: int, y: int): int =
    return x + y
end
```

### Generics
Generics are definied in between square braces before the name in function declarations

```blorp
fun [T] add(x: T, y: T): T =
    return x + y
end
```

### Functions as values
Function are treated as values in blorp.

Their type are defined with their parameters preceded by `->` and their return type:
```blorp
let suc: Int -> Int        // a function that takes and int and returns an int
let add: (Int, Int) -> Int // when there is more than one arg, it should be surrounded by parenthesis
```

Anonymous functions are declared with the following syntax: `<args> -> <body>`

```blorp
let suc := a -> a + 1  // if the function is a single expression, the value is automatically returned
let bar := (a, b) ->
    return a + b       // when it is not, you have to explicitly call return
end
```

They can also have their types anotated:

```blorp
let suc: Int -> Int = a -> a + 1
```

Though it is not needed, and also may worsen legibility

### Function calling
Functions are called either the normal way with parenthesis, or they can be called as methods using a `.`

In turn, calling a function with a `.` is the same as calling it as its first value

```blorp
let foo := (10).double().add(5) // returns 25
let foo := add(double(10), 5)   // it is the same as this
```

This syntax can be explored more to its limits, as a tuple `(a, b)` gets piped as `f(a, b)` when called using the dot syntax

```blorp
let added := (10, 20).add()         // returns 30

let foo := ((10).double(), 5).add() // the same as the first example
```

### Loops
blorp contains both for and while loops

While loops syntax is `while <cond> [block]`

```blorp
// will print "loop" indefinetly
while true do
    println("loop")
end

// it is the same, just only on one line
while true do println("loop")
```

For loops syntax is `for <item> in <iterator> [block]`

```blorp
// will print 0 to 9. range(n) returns a range from 0 to n
for i in range(10) do
    println(i)
end

// range can also take in 2 arguments, being the start and the end of the iterator
for i in range(5, 11) do pritnln(i)
```

### Pattern matching and if statements

If statements follow the normal syntax as `if <cond> [block] else [block]`

```blorp
if true do
    println("true")
end else do
    println("false")
end

// the same as
if true do println("true")
else    do println("false")
```

When there is more than one possible value, match staments are more idiomatic

The syntax is: `match <value> in [patterns]`, where a pattern is: `<pat> => [block]`

```blorp
let str := "foobar"

match str in
    "foo" + rest => do
        let rest = rest.to_upper()
        println("starts with foo and ends with: {rest}")
    end
    "foobar" => println("it is foobar")
    other => println("i dont know what this is: {}", other)
end
```

### Types and interfaces

Product types are defined with the `struct` keyword

```blorp
struct Point =
    x: Int,
    y: Int,
end
```

Sum types are defined with the `enum` keyword

```blorp
enum State =
    Active,
    Inactive
end
```

blorp allow for full algebraic data types with generics:

```blorp
struct User =
    id: Int,
    role: UserRole,
end

enum UserRole =
    Admin,
    Customer,
end
```

For simple type aliases, use `type` keyword

```blorp
type IntArrray = Arr(Int)

type MyUnitType // defines a type with only one value
```

Design patterns from functional programming language are also expressed here

```blorp
enum Result[T, E] =
    Ok(T),
    Err(T).
end
```

Interfaces are what allow constraints with generics

A type T implements the interface if all the functions inside the interface exist with those parameters. Inside interfaces, `Self` is a special type that represents the type of the thing

```blorp
// in this case, any type that implements to_string is considered of the interface Display
interface Display =
    fun to_string(Self) -> String
end
```

More complete example

```blorp
type Cat
type Dog

interface Speak =
    fun make_sound(Self): String
end

fun make_sound(_: Cat): String =
    return "meow"
end
// now Cat implements Speak

fun make_sound(_: Dog): String =
    return "woof"
end
// Dog too now

// interfaces allow you to talk about all members that implement this at once
fun [T: Speak] speak(speaker: T) :=
    println("spoke: {}", animal.speak())
end
```

# Todo
- Change generic types from things like `Arr<T>` to `Arr(T)`, so that angle brackets would be only used as an operator
