# Future plans (proposal)

This page preserves the full language-design backlog from the former README.
None of the syntax below is implemented unless the
[language status](../reference/language-status.md) page says otherwise.
Generic arguments use `<...>` throughout.

### Generics
Generics are written in angle brackets before a function name

```fob
fun <T> add(x: T, y: T): T =
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
type Result<T, E>: enum =
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

fun <T: Speak> speak(speaker: T) =
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
<T> (T, T -> Bool) -> T
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
fun <T> not(predicate: T -> Bool): T -> Bool =
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
type Result<T, E>: enum =
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
type Expr<T>: enum =
    Integer: Int is Self<Int>
    Boolean: Bool is Self<Bool>
    Add: (Self<Int>, Self<Int>) is Self<Int>
    Equal<U>: (Self<U>, Self<U>) is Self<Bool>
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
