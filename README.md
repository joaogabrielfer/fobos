# blorp

## Quickstart
To run a program:
```console
$ cargo run -- run <file>
```

## Testing
Use `cargo run -- generate-expected` to compile all .blorp programs in tests/ to their tokens and save them as .blorp.expected.

Use `cargo test` to run the unit tests and ensure the result is the expected

## Todo

- [x] variable definition
- [x] blocks
- [x] yield and return
- [x] function declaration
- [x] functions as values
- [x] function calling
- [x] while loops
- [x] if statements
- [ ] arrays
- [ ] for loops
- [ ] type checking
- [ ] custom types (adts)
- [ ] interfaces
- [ ] pattern matching
- [ ] generic
- [ ] ranges
- [ ] patterns
- [ ] effects

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
- `do` is used in blocks like `while` and `for` loops and `if` staments
- `in` is used in `match` statements

If after the block opener there is not a new line, it has an implicit `end` at the end of it.
```blorp
let my_var = 10
           ^    ^
       opener  implicit `end`

// is the same as
let my_var = do
    yield 10
end
```

### Yield and return

`return` returns that value from the nearest function

```blorp
fun foo(): Int = // returns 10
    return 10
end
```

`yield` returns that value from the nearest expression.

```blorp
let bar :=
    let a := 1
    let b := 2
    yield a + b
end
```

They can be used to disambiguate whether you want to return that value from the function, or evaluate that block, since blorp has no implicit returns

```blorp
fun foo(): Int =
    let bar :=
        let a := 10
        let b := 20
        return a + b // this here will return 30 from the function
    end

    return bar + 10 // and this code is unreachable
end

fun foo(): Int =
    let bar :=
        let a := 10
        let b := 20
        yield a + b // this now evaluates this whole block to 30 then yields it to bar
    end

    return bar + 10 // now it will return 40
end
```

Yield also has 2 different behaviours based on the context of the block. If it is an lhs expression, it bubbles that yield up to the outer expression, if it is an rhs expression, the block itself evaluates to that value

```blorp
let a :=
    if true do
        yield 10 // here these yields instead of making the if statement evaluate to 10, they bubble that yield to the outer `do` block and make it evaluate to 10
    end else do
        yield 20
    end
end

let b :=
    let foo := if true do
        yield 10 // now since it is a rhs value, its yield is captured by foo (b here evaluates to nothing since foo is not being yielded to it)
    end else do
        yield 20
    end
end

// just reminder that this is not implicit returns, so this is valid code:
let c := // evaluates to 10
    if true do
        yield 10
    end else do
        yield 20
    end

    yield 30 // unreachable code
end
```

As it is now, `yield` only returns the first value, though the plan on the future is make `yield` an effect handler for collections and streams, so that you could yield in a loop and collect an array.

### Function declaration
Functions are defined with the `fun` keyword succeded with the name and signature of the function, with the return type following the variable definition conventino of being between the `:` and the `=`
```blorp
fun add(x: int, y: int): int =
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
    "foo" <> rest => do
        let rest = rest.to_upper()
        println("starts with foo and ends with: {rest}")
    end
    "foobar" => println("it is foobar")
    other => println("i dont know what this is: {}", other)
end
```

## Future plans

### Generics
Generics are definied in between square braces before the name in function declarations

```blorp
fun [T] add(x: T, y: T): T =
    return x + y
end
```

### Ranges

Ranges are an instance of an operator that can be created using either the `range()` builtin or a `..=`/`..<` syntax. There is not a plain `..` variant

```blorp
for i in 0..=9 do // starts from 0 goes up to 9
    println(i)
end

var my_vec: Arr<Int> = Arr()
(0..<MY_UPPER_LIMIT).for_each(i -> my_vec.push(i))
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
type IntArrray = Arr<Int>

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

### Patterns

Patterns in blorp are a first class citizen

In pattern matching statements, everything you can match on, has to be an instance of the IntoPattern interface. Normal types that you would expect to be matched already implement by default

```blorp
match d in
    0..=10  => ...
    11..=20 => ...
    _       => ...
end
```

But you can also implement your own instance of IntoPattern for your types and use them in pattern matches

```blorp
match foo in
    MyPattern() => ... // here MyPattern is a type constructor
    _ => ...
end
```

You can use this alongside pattern pinning, by prefixing the variable with the ^ operator, that allows you to use the value of a variable as a pattern in a match statement

```blorp
let p := RegexPattern("foo.*(bar)+") // here an example of what could be a custom pattern

match s in
    ^p => ...    // matches anything that matches that regex pattern (it does not bind anything to p, it uses its value instead)
    "foo" => ... // matches "foo" exacly
    other => ... // matches anything else and bind that match to 'other'
end
```

### Yield and effect handlers

Have `collect` and `stream` effect handlers where they could catch the yields into a stream or an array

```blorp
let my_stream := stream do // would result in a Stream<Int> with values of 0 to +inf
    var i := 0
    while true do
        yield i
        i = i + 1
    end
end

let my_array := collect do // would result in a Arr<Int> with values of 0 through 9
    for i in 0..<9 do yield i
end
```

The difference from both is that stream are lazy list while collections are normal arrays
