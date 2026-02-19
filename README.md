# Lyra

A functional programming language with Hindley-Milner type inference, built from scratch in Rust.

## Features

- **Type inference** - Full Hindley-Milner with let-polymorphism. No type annotations needed.
- **Bytecode VM** - Programs compile to bytecode and run on a stack-based virtual machine.
- **Pattern matching** - Destructure ADTs, lists, tuples, and literals with exhaustiveness checking.
- **Algebraic data types** - Define custom types with constructors: `type Shape = Circle Int | Rect Int Int`
- **Tail call optimization** - Recursive functions run in constant stack space.
- **String interpolation** - `"hello {name}, you are {to_string(age)} years old"`
- **Record types** - `{ name: "Alice", age: 30 }` with dot access.
- **Pipe operator** - `[1,2,3] |> map(fn (x) -> x * 2) |> sum`
- **Module system** - `import "utils"` for multi-file programs.
- **REPL** - Interactive with multi-line input, syntax highlighting, and "did you mean?" suggestions.

## Quick Start

```
cargo build --release
./target/release/lyra                     # launch REPL
./target/release/lyra examples/showcase.lyra        # run a file
./target/release/lyra examples/showcase.lyra --vm   # run with bytecode VM
```

## Examples

```ml
-- Recursive fibonacci
let rec fib = fn (n) ->
  if n <= 1 then n
  else fib(n - 1) + fib(n - 2)

-- Quicksort with pattern matching
let rec quicksort = fn (lst) ->
  match lst with
  | [] -> []
  | pivot :: rest ->
    let less = filter(fn (x) -> x < pivot, rest) in
    let greater = filter(fn (x) -> x >= pivot, rest) in
    append(append(quicksort(less), [pivot]), quicksort(greater))

-- Pipeline with higher-order functions
let result = range(1, 101)
  |> filter(fn (x) -> x % 2 == 0)
  |> map(fn (x) -> x * x)
  |> sum

-- Algebraic data types
type Tree = Leaf Int | Node Tree Tree

let rec tree_sum = fn (t) ->
  match t with
  | Leaf(n) -> n
  | Node(l, r) -> tree_sum(l) + tree_sum(r)

-- Records and string interpolation
let person = { name: "Alice", age: 30 }
println("Name: {person.name}, Age: {to_string(person.age)}")
```

## Tests

```
cargo test
```

140 tests across lexer, parser, type inference, VM, and end-to-end integration.

## Architecture

```
src/
  lexer/       Tokenizer with string interpolation support
  parser/      Pratt parser for expressions, declarations, patterns, types
  ast/         AST node definitions
  types/       Hindley-Milner inference, unification, exhaustiveness checking
  compiler/    AST -> bytecode compilation with local/upvalue resolution
  vm/          Stack-based virtual machine with tail call optimization
  eval/        Tree-walking interpreter (alternative backend)
  stdlib/      50+ built-in functions
  repl/        Interactive REPL with rustyline
```

## Stdlib

| Category | Functions |
|----------|-----------|
| IO | `print`, `println`, `to_string` |
| Math | `abs`, `min`, `max`, `pow`, `float_of_int`, `int_of_float` |
| List | `length`, `head`, `tail`, `reverse`, `append`, `range`, `nth`, `take`, `drop`, `flatten`, `sum`, `product` |
| HOF | `map`, `filter`, `fold`, `zip`, `sort`, `any`, `all` |
| String | `str_length`, `str_concat`, `str_contains`, `str_split`, `str_chars`, `str_trim`, `str_uppercase`, `str_lowercase`, `str_replace`, `str_starts_with`, `str_ends_with`, `str_substring`, `string_to_int`, `int_to_string` |
