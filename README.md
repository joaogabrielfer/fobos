# Fobos

## Quickstart
To run a program:
```console
$ cargo run -- run <file>
```

To run the repl (WIP):
```console
$ cargo run
```

For debbuging:
```console
$ cargo run -- ast <file>    # generate the ast of the programs
$ cargo run -- tokens <file> # generate the tokens of the program
```

## Testing
Use `cargo run -- generate-expected` to compile all .fob programs in fixtures/ to their results and save them inside fixtures/expected

Use `cargo test` to run the unit tests and ensure the result is the expected

## Todo

```md
## Todo

- [x] variable definition
- [x] blocks
- [x] yield and return
- [x] function declaration
- [x] functions as values
- [x] function calling and piping
- [x] while loops
- [x] if statements
- [x] arrays
- [x] tuples
- [x] ranges
- [x] for loops
- [x] type checking
- [x] higher order functions
- [x] function overloading

### Language foundation

- [ ] module loading and imports
- [ ] named function arguments
- [ ] required named parameters
- [x] trailing commas
- [ ] boolean identifiers ending in `?`
- [ ] word forms for boolean operators
- [ ] operator sections
- [ ] spread and rest syntax

### Type system

- [ ] nominal wrapper types
- [ ] transparent type aliases
- [ ] structs and generated constructors
- [ ] tags
- [ ] algebraic data types
- [ ] anonymous structs and enums
- [ ] enum tag projection
- [ ] pattern matching
- [ ] generics
- [ ] interfaces
- [ ] custom patterns

### Compile-time system

- [ ] macros
- [ ] syntax types
- [ ] templates and interpolation
- [ ] hygienic macro expansion
- [ ] standard library macros

### Runtime and standard library

- [ ] standard library loading
- [ ] result and option types
- [ ] resource management
- [ ] C FFI
- [ ] streams and effect handlers

### Later plans

- [ ] GADTs
- [ ] refinement types
- [ ] derivation system
- [ ] user-extensible compile-time interfaces
```

## Language syntax

### Variable definition
```fob
let foo := 10    // immutable
var bar := "bar" // mutable
```
- The syntax is: `[let | var] <name> : <type> = <value>`
- If no type is provided, it is inferred
- Variables in Fobos use camel_case syntax

### Blocks
Blocks in Fobos are not declared with curly braces. Instead, they use `end` to denote the end of blocks and `=`, `->`,  `do`, `in` to indicate the opening of is, when succeeded by a new line.
```fob
let my_var =
    var (a, b) = (10, 20) // assignment with tuples
    yield a + b
end
```

- `=` is used in variable and function definitions
- `do` is used in blocks like `while` and `for` loops and `if` staments
- `in` is used in `match` statements

If after the block opener there is not a new line, it has an implicit `end` at the end of it.
```fob
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

```fob
fun foo(): Int = // returns 10
    return 10
end
```

`yield` returns that value from the nearest expression.

```fob
let bar :=
    let a := 1
    let b := 2
    yield a + b
end
```

They can be used to disambiguate whether you want to return that value from the function, or evaluate that block, since Fobos has no implicit returns

```fob
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

Yield also has 2 different behaviours based on the context of the block. If it is in a statement position, it bubbles that yield up to the outer expression, if it is in an value, the block itself evaluates to that value

```fob
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
Functions are defined with the `fun` keyword succeded with the name and signature of the function, with the return type following the variable definition convention of being between the `:` and the `=`
```fob
fun add(x: Int, y: Int): Int =
    return x + y
end
```

Variables from parameters are immutable by default, in case you want to mutate the function argument, use the keyword `var` before the type

```fob
fun inc(x: var Int): () =
    x = x + 1
end
```

### Functions as values
Function are treated as values in Fobos.

Their type are defined with their parameters preceded by `->` and their return type:
```fob
let suc: x: Int -> Int        // a function that takes and int and returns an int
let add: (x: Int, y: Int) -> Int // when there is more than one arg, it should be surrounded by parenthesis
```

Anonymous functions are declared with the following syntax: `<args> -> <body>`

```fob
let suc := a -> a + 1  // if the function is a single expression, the value is automatically returned
let bar := (a, b) ->
    return a + b       // when it is not, you have to explicitly call return
end
```

They can also have their types annotated:

```fob
let suc: Int -> Int = a -> a + 1
```

Though it is not needed, and also may worsen legibility

### Function calling
Functions are called either the normal way with parenthesis, or they can be called being piped using a `.`

In turn, calling a function with a `.` is the same as calling it as its first value

```fob
let foo := (10).double().add(5) // returns 25
let foo := add(double(10), 5)   // it is the same as this
```

This syntax can be explored more to its limits, as a tuple `(a, b)` gets piped as `f(a, b)` when called using the dot syntax

```fob
let added := (10, 20).add()         // returns 30

let foo := ((10).double(), 5).add() // the same as the first example
```

If you want to pass a literal tuple, you wrap it in another set of parenthesis

```fob
fun accepts_tuples(tup: (Int, Int)): Int =
    return tup.0 + tup.1
end

let foo := ((10, 5)).accepts_tuples()
let foo := accepts_tuples((10, 5))


```

### Loops
Fobos contains both for and while loops

While loops syntax is `while <cond> [block]`

```fob
// will print "loop" indefinitely
while true do
    println("loop")
end

// it is the same, just only on one line
while true do println("loop")
```

For loops syntax is `for <item> in <iterator> [block]`

```fob
// will print 0 to 9. range(n) returns a range from 0 to n
for i in range(10) do
    println(i)
end

// range can also take in 2 arguments, being the start and the end of the iterator
for i in range(5, 11) do println(i)
```

### If statements

If statements follow the normal syntax as `if <cond> [block] else [block]`

```fob
if true do
    println("true")
end else do
    println("false")
end

// the same as
if true do println("true")
else    do println("false")
```

### Ranges

Ranges are an instance of an operator that can be created using either the `range()` builtin or a `..=`/`..<` syntax. There is not a plain `..` variant

```fob
for i in 0..=9 do // starts from 0 goes up to 9
    println(i)
end

var my_vec: Arr<Int> = Arr()
(0..<MY_UPPER_LIMIT).for_each(i -> my_vec.push(i))
```

## Future plans

### Generics
Generics are defined in between square braces before the name in function declarations

```fob
fun [T] add(x: T, y: T): T =
    return x + y
end
```

### Pattern matching

The syntax is: `match <value> in [patterns]`, where a pattern is: `<pat> => [block]`

```fob
let str := "foobar"

match str in
    "foo" <> rest =>
        let rest = rest.to_upper()
        println("starts with foo and ends with: {rest}")
    end
    "foobar" => println("it is foobar")
    other => println("i dont know what this is: {}", other)
end
```


### Types and interfaces

Types follow a syntax similar to value bindings. `type` can be thought of as a binding that operates on types

Type names must begin with a capital letter

#### Nominal types

Using `:=` creates a new nominal type with the type on the right as its representation

```fob
type Email := String
type UserId := Usize
type Entry := (Int, String)
```

A nominal type is distinct from its representation even when both have the same runtime layout

```fob
fun send_email(email: Email) =
    // ...
end

let value: String = "foo@example.com"

send_email(value) // type error
send_email(Email(value))
```

A single-value nominal type receives a positional default constructor

```fob
let email := Email("foo@example.com")
```

A tuple-backed nominal type also receives a positional constructor

```fob
let entry := Entry(10, "foo")
```

#### Type aliases

Aliases use the `alias` declaration form

```fob
type IntArray: alias = Arr<Int>
```

An alias does not create a new type

```fob
let first: IntArray = [1, 2, 3]
let second: Arr<Int> = first
```

Recursive aliases are not allowed

#### Structs

Product types use the `struct` declaration form

```fob
type Point: struct =
    x: Int
    y: Int
end
```

Structs receive a default constructor whose arguments must be passed by name

```fob
let point := Point(
    x = 10,
    y = 20,
)
```

Positional struct construction is not allowed

```fob
Point(10, 20) // type error
```

Anonymous structs are allowed anywhere a type can appear

```fob
type Rectangle: struct =
    position: struct =
        x: Int
        y: Int
    end

    width: Int
    height: Int
end
```

Every anonymous type has a unique nominal identity

```fob
Mouse::position != Rectangle::position
```

If an anonymous type belongs to a public field, its associated type is also public

#### Tags

Tags represent a finite collection of values with no attached payload

```fob
type Direction: tag =
    North
    South
    East
    West
end
```

Their values are accessed through the type namespace

```fob
let direction := Direction::North
```

A tag can optionally declare its underlying integer representation

```fob
type Opcode: tag<U8> =
    Add = 0x01
    Subtract = 0x02
    Jump = 0x10
end
```

#### Enums

Enums are tagged unions. Every variant associates a tag with a payload type

```fob
type Result[T, E]: enum =
    Ok: T
    Err: E
end
```

A variant without a declared payload has the unit type as its payload

```fob
type State: enum =
    Inactive
    Loading
    Active: Connection
end
```

Enum variants are constructors

```fob
let success := Result::Ok(10)
let error := Result::Err("failed")
```

Enums may also be anonymous

```fob
type Payload: struct =
    len: Usize

    kind: enum =
        Message: String
        Redirect: Url
    end
end
```

Anonymous nested types are accessed through their containing type

```fob
Payload::kind
```

Every enum has an automatically generated tag type

```fob
Result::tag
Result::tag::Ok
Result::tag::Err

Payload::kind::tag
Payload::kind::tag::Message
Payload::kind::tag::Redirect
```

Every enum value has a read-only `.tag` projection

```fob
if payload.kind.tag == Payload::kind::tag::Redirect do
    // handle a redirect without accessing its payload
end
```

The representation of an enum tag can be configured explicitly

```fob
type Token: enum(tag = U8) =
    Let
    Identifier: String
    Integer: Int
end
```

The tag projection does not expose or refine the variant payload. Pattern matching is the only way to access the value stored inside a variant

```fob
match payload.kind in
    .Message(message) =>
        handle_message(message)
    end

    .Redirect(url) =>
        handle_redirect(url)
    end
end
```

Contextual variant names can be used when the enum type is already known

```fob
payload.kind.tag == .Redirect
```

#### Constructors

Generated constructors behave like ordinary functions

A user-defined function with the same name and parameter signature as a generated constructor shadows it

```fob
pub type Email := String

pub fun Email(value: String): Result<Self, EmailError> =
    if value.valid_email?() do
        return Ok(Self(value))
    end

    return Err(.InvalidFormat)
end
```

`Self(...)` is a privileged raw constructor. It is only available inside a constructor for the type being created

For wrapper and tuple-backed types it receives positional values

```fob
return Self(value)
return Self(id, name)
```

For structs it receives named fields

```fob
pub fun Point(): Self =
    return Self(
        x = 0,
        y = 0,
    )
end
```

A constructor may return the type directly or wrap it in a construction result such as `Option<Self>` or `Result<Self, E>`

Return types do not normally participate in function overloading. Constructors are temporarily a special case because a custom fallible constructor may shadow an infallible generated constructor with the same parameters

Generated constructors follow the visibility of their type. Explicit constructors follow their own `pub` modifier

```fob
pub type Email := String

fun Email(value: String): Self =
    // public type with private construction
    return Self(value)
end
```

#### Interfaces

Interfaces constrain generic types by describing the functions that must exist for them

```fob
interface Display =
    fun to_string(value: Self): String
end
```

A type implements an interface when functions matching all the interface requirements are available for that type

```fob
type Cat := ()
type Dog := ()

interface Speak =
    fun make_sound(value: Self): String
end

fun make_sound(_: Cat): String =
    return "meow"
end

fun make_sound(_: Dog): String =
    return "woof"
end

fun [T: Speak] speak(speaker: T) =
    println("spoke: {}", speaker.make_sound())
end
```

Interface derivation and automatic forwarding from nominal representations are still undecided

### Patterns

Patterns in Fobos are a first class citizen

In pattern matching statements, everything you can match on, has to be an instance of the IntoPattern interface. Normal types that you would expect to be matched already implement by default

```fob
match d in
    0..=10  => ...
    11..=20 => ...
    _       => ...
end
```

But you can also implement your own instance of IntoPattern for your types and use them in pattern matches

```fob
match foo in
    MyPattern() => ... // here MyPattern is a type constructor
    _ => ...
end
```

You can use this alongside pattern pinning, by prefixing the variable with the ^ operator, that allows you to use the value of a variable as a pattern in a match statement

```fob
let p := RegexPattern("foo.*(bar)+") // here an example of what could be a custom pattern

match s in
    ^p => ...    // matches anything that matches that regex pattern (it does not bind anything to p, it uses its value instead)
    "foo" => ... // matches "foo" exacly
    other => ... // matches anything else and bind that match to 'other'
end
```

### Yield and effect handlers

Have `collect` and `stream` effect handlers where they could catch the yields into a stream or an array

```fob
let my_stream := stream do // would result in a Stream<Int> with values of 0 to +inf
    var i := 0
    while true do
        yield i
        i = i + 1
    end
end

let my_array := collect do // would result in a Arr<Int> with values of 0 through 8
    for i in 0..<9 do yield i
end
```

The difference from both is that stream are lazy list while collections are normal arrays

### Trailing commas

Allow trailing commas for multi-line array and tuple literals as well as function call

```fob
let my_arr := [
    1,
    2,
    3,
]
```

```fob
let my_tuple := (
    1,
    true,
    "foo",
)
```

```fob
foo(
    1,
    2,
    true,
    false,
)
```

### ? in Identifiers

Functions that return a boolean should, by convention, have their names end with `?`

```fob
empty?(value)
valid?(user)
positive?(number)

value.empty?()
user.valid?()
number.positive?()
```

The `?` is considered part of the identifier and is not an operator

### Modules

Modules are accessed using the `::` operator

```fob
let result := std::math::abs(-10)
let file := std::fs::open("foo.txt")
```


This also applies to functions accessed from modules

```fob
let empty := std::string::empty?(path)
```

They can be imported into the current scope and then used with the piped syntax

```fob
use std::string::{empty?}

let empty := path.empty?()
```

### Macros

Macros are compile-time functions that receive and return syntax instead of runtime values

Macro declarations use curly braces instead of parenthesis

```fob
macro unless{
    condition: std::syntax::Expression,
    body: std::syntax::Block,
}: std::syntax::Expression =
    template
        if !${condition} ${body}
    end
end
```

Macros are also called using curly braces

```fob
unless{foo.empty?(), do
    println("foo is not empty")
end}
```

This makes normal function and macro calls visually different

```fob
foo(...) // runtime function call
foo{...} // compile-time macro call
```

Macro arguments are passed as syntax and are not evaluated before the macro is expanded

Like normal function calls, macros can also use the piped `.` syntax

```fob
foo.assert{}
assert{foo} // the same as above
```

The lhs of the pipe is passed as the first macro argument

```fob
value.foo{a, b}
foo{value, a, b} // the same as above
```

Syntax types are exposed through the `std::syntax` module

```fob
std::syntax::Expression
std::syntax::Identifier
std::syntax::Block
std::syntax::Lambda
std::syntax::Declaration
std::syntax::DeclarationList
std::syntax::Parameter
std::syntax::ParameterList
std::syntax::Type
std::syntax::Pattern
std::syntax::MatchArm
std::syntax::MatchArmList
```

These are compile-time types and can only be used inside macro declarations and future compile-time functions

Macros use `template` to construct syntax

`${value}` inserts one syntax node into the template, while `$..{values}` inserts all the syntax nodes contained in a sequence

```fob
macro define_function{
    name: std::syntax::Identifier,
    parameters: std::syntax::ParameterList,
    return_type: std::syntax::Type,
    body: std::syntax::Block,
}: std::syntax::Declaration =
    template
        fun ${name}($..{parameters}): ${return_type} =
            ${body}
        end
    end
end

define_function{
    foo,
    (x: Int, y: Int),
    Int,
    do
        return x + y
    end,
}
```

Identifiers created inside a macro template are hygienic by default, meaning they do not conflict with variables from the place where the macro is called

Syntax passed into a macro keeps its original source span, while generated syntax also stores where the macro was called and where it was declared

Macros are expanded before name resolution and type checking

### Assertion macros

`assert` takes a boolean expression, crashes if it is false and returns the same value if it is true

```fob
let foo := true

foo.assert{"foo should be true"}
```

Since the value is returned, it can continue being piped

```fob
foo.assert{"foo should be true"}.do_other_thing()
```

`assert_not` does the opposite. It crashes if the boolean expression is true

```fob
path.empty?().assert_not{"path cannot be empty"}
```

`assert_that` takes a value and a predicate of that value, crashes if the predicate returns false and then returns the original value

Its conceptual type is:

```fob
[T] (T, T -> Bool) -> T
```

```fob
foo.assert_that{
    (> 2),
    "foo should be greater than 2",
}.do_other_thing()
```

Since predicates are normal first-class functions, they can be passed directly

```fob
path.assert_that{
    empty?,
    "path should be empty",
}
```

To assert that the path is not empty, the predicate can be negated using the `not` function

```fob
path.assert_that{
    empty?.not(),
    "path cannot be empty",
}
```

The expression above is the same as:

```fob
path.assert_that{
    not(empty?),
    "path cannot be empty",
}
```

`not` is a normal higher-order function that takes a predicate and returns its negation

```fob
fun [T] not(predicate: T -> Bool): T -> Bool =
    return value -> !predicate(value)
end
```

### Operator sections

Binary operators can be partially applied by leaving one of their operands out inside parenthesis

```fob
(+ 1) // x -> x + 1
(> 2) // x -> x > 2
(2 >) // x -> 2 > x
```

These operator sections are normal function values

```fob
let increment := (+ 1)
let greater_than_two := (> 2)

let result := increment(10)
let valid := greater_than_two(5)
```

They can be used with higher-order functions

```fob
let incremented := values.map((+ 1))
let positive := values.filter((> 0))
```

They can also be used as predicates for macros like `assert_that`

```fob
foo.assert_that{
    (> 2),
    "foo should be greater than 2",
}
```

The order of the operands matters for operators that are not commutative

```fob
(> 2) // x -> x > 2
(2 >) // x -> 2 > x
```

### Spread, splice and rest syntax

The `..` token represents the general idea of spreading or collecting multiple values, but its exact behaviour depends on where it is used

When used after a value inside a function call, it spreads the value into multiple arguments

```fob
let tup := (1, 2)

foo(tup..)
foo(1, 2) // the same as above
```

It can also be used inside tuple and array literals

```fob
let tup := (1, 2)
let extended := (0, tup.., 3)

let values := [1, 2]
let extended_values := [0, values.., 3]
```

Inside macro templates, `$..{value}` splices multiple syntax nodes into the surrounding syntax

```fob
template
    fun ${name}($..{parameters}): ${return_type} =
        ${body}
    end
end
```

Inside patterns, prefixing a binding with `..` collects the remaining values

```fob
match values in
    [] => println("empty")
    [only] => println("one value: {}", only)
    [head, ..tail] => do
        println("head: {}", head)
        println("tail: {}", tail)
    end
end
```

Initially, only one rest pattern is allowed and it must be the last element of the pattern

```fob
[head, ..tail]       // valid
[a, b, ..rest]       // valid
[..start, last]      // not valid initially
[a, ..middle, last]  // not valid initially
[a, ..first, ..last] // invalid
```

A plain `..` is not a range operator. Ranges use only `..<` and `..=`

### Result propagation

`Result` should eventually be a normal algebraic data type from the standard library

```fob
type Result[T, E]: enum =
    Ok: T
    Err: E
end
```

The `or_return` macro propagates an error from a function

```fob
fun read_config(path: String): Result<Config, IOError> =
    let text := fs::read(path).or_return{}
    let config := parse_config(text).or_return{}

    return Ok(config)
end
```

It evaluates the result once and expands to the equivalent of:

```fob
match result in
    .Ok(value) => yield value

    .Err(error) => return .Err(error)
end
```

Since `return` exits the nearest function, an `Err` is returned from the function where `or_return` was called

For functions with different error types, the error can first be converted using `map_error`

```fob
fun read_config(path: String): Result<Config, AppError> =
    let text :=
        fs::read(path)
            .map_error(AppError::IO)
            .or_return{}

    return parse_config(text)
end
```

### Resource management

The `with` macro provides a scoped way of working with resources

```fob
let text :=
    fs::open("file.txt").with{file ->
        return file.read_all()
    end}.or_return{}
```

The piped form above is equivalent to:

```fob
with{
    fs::open("file.txt"),
    file ->
        return file.read_all()
    end,
}.or_return{}
```

Because `with` is a macro, the resource expression is passed as syntax and is only evaluated where the expanded code places it

The resource expression must only be evaluated once

The lambda creates a function boundary around the resource usage. Returning from the lambda does not skip the resource cleanup

`with` should expand into a standard library resource helper that guarantees the resource is closed after the lambda finishes, including when the lambda returns an error

```fob
std::resource::with(resource, callback)
```

The actual cleanup behaviour cannot be implemented safely by only placing `close` after the body, since an early return or runtime error could skip it. It requires support from the runtime, destructors or an unwind-safe standard library primitive

### Named function arguments

Parameter names are part of function types

```fob
let add: (x: Int, y: Int) -> Int
```

Ordinary function parameters can be passed either positionally or by name

```fob
fun move(x: Int, y: Int) =
    // ...
end

move(10, 20)
move(x = 10, y = 20)
move(10, y = 20)
```

Parameters marked with `named` must be passed by name

```fob
fun connect(
    address: String,
    named timeout: Duration,
) =
    // ...
end

connect("localhost", timeout = seconds(10))
```

This is invalid

```fob
connect("localhost", seconds(10))
```

Overloads may differ by parameter types and required parameter names

```fob
fun find(named id: UserId): User :=
    // ...
end

fun find(named email: Email): User :=
    // ...
end

let first := find(id = user_id)
let second := find(email = address)
```

Two overloads cannot be distinguished only by their return types

Overloads whose valid call forms would be ambiguous are rejected

```fob
fun foo(x: Int) :=
    // ...
end

fun foo(y: Int) :=
    // invalid because foo(10) would be ambiguous
end
```

### Function return syntax

Omitting a function return annotation means that the function returns the unit type

```fob
fun no_op() =
    println("nothing")
end
```

The function above is syntax sugar for:

```fob
fun no_op(): () =
    println("nothing")
end
```

A return type can be inferred by keeping the colon before the inferred assignment operator

```fob
fun add(x: Int, y: Int):=
    return x + y
end
```

An explicit return type continues to use `: Type =`

```fob
fun add(x: Int, y: Int): Int =
    return x + y
end
```

Therefore the three forms have different meanings

```fob
fun foo() =       // returns ()
fun foo() :=      // inferred return type
fun foo(): Type = // explicit return type
```

Returning a non-unit value from a function whose return annotation was omitted is a type error

```fob
fun foo() =
    return 10 // type error
end
```

### Pattern matching arm blocks

The fat arrow opens a pattern arm

If the arm body is on the same line, the arm has an implicit `end`

```fob
match value in
    .None => println("none")
    .Some(value) => println(value)
end
```

If the body begins on the next line, the arm must be closed explicitly

```fob
match value in
    .None =>
        println("none")
        recover()
    end

    .Some(value) =>
        println(value)
        use(value)
    end
end
```

The body is indented one level past the pattern. The arm's `end` is aligned with the pattern

### Boolean operators

Fobos may support both word and symbolic forms for boolean operators

```fob
a and b
a && b

a or b
a || b

not a
!a
```

Both forms of each operator would have the same precedence, associativity and short-circuit behaviour

Bitwise operators remain separate

```fob
a & b
a | b
a ^ b
```

The preferred canonical style for formatted Fobos code is still undecided

### Later type system extensions

#### GADTs

GADTs extend enums by allowing each variant to specify a more precise result type

The `is` keyword may be used to declare that result

```fob
type Expr[T]: enum =
    Integer: Int is Self[Int]
    Boolean: Bool is Self[Bool]
    Add: (Self[Int], Self[Int]) is Self[Int]
    Equal[U]: (Self[U], Self[U]) is Self[Bool]
end
```

Pattern matching on a GADT would refine its generic type inside each arm

GADTs require type equality constraints and bidirectional type checking and are not an immediate implementation goal

#### Refinement types

Refinement types describe a nominal type whose valid values must satisfy a predicate

```fob
type Id: refinement(self) =
    Int
where
    self > 0 and self < MAX_ID
```

Construction may eventually require a runtime check, a static proof or an explicitly unsafe raw constructor

The exact construction and proof model is still undecided
