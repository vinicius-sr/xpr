# xpr

A small, fast expression compiler and interpreter in Rust.

`xpr` turns an arithmetic/comparison expression written as text into a compact
stack-based program (reverse Polish notation) and evaluates it.

## Why xpr exists

**The goal:** an *embedded* language — a small domain-specific language you drop
into a much larger Rust program, not a standalone product. The design therefore
favors cheap, repeated compile-and-run (evaluating the same shape of expression
hundreds of thousands of times, e.g. as a fitness function in an evolutionary
loop) and easy embedding over feature completeness.

**The reason:** it was also built as a way to learn how programming languages
work — scanning, parsing, compiling, and interpreting. It is deliberately small
so that each stage can be read and understood in an afternoon.

## Pipeline

    source text  →  tokens  →  RPN opcodes  →  stack machine  →  f64

1. **Scan** — tokenize the input (numbers, identifiers, operators, parentheses).
2. **Compile** — a recursive-descent parser emits an opcode vector in RPN.
3. **Interpret** — a small stack machine executes the opcodes and returns the result.

## How parsing works: recursive descent

The parser is written by hand as *recursive descent*: one function per grammar
rule, where each rule calls the next-higher-precedence rule to parse its
operands. The grammar, from lowest to highest precedence:

    expr       = equality
    equality   = comparison { ("==" | "!=") comparison }
    comparison = term       { (">"  | ">=" | "<"  | "<=") term }
    term       = factor     { ("+"  | "-")          factor }
    factor     = unary      { ("*"  | "/")          unary }
    unary      = ("-" | "!") unary | primary
    primary    = NUMBER | "(" expr ")"

Each rule does two things:

1. It parses its left operand by calling the rule *below* it (e.g. `term` calls
   `factor`). That call keeps descending until a `primary` is reached — a number
   or a parenthesized sub-expression.
2. It then **loops** while the next token is one of its own operators: parse the
   right operand (again via the rule below), and emit the operator.

That loop is what produces both correct precedence (a level only consumes its own
operators, so `*` binds tighter than `+`) and left-associativity (`1 - 2 - 3`
parses as `(1 - 2) - 3`).

There is no separate AST. As the parser descends and climbs, it writes opcodes
straight into a single vector in RPN: a `primary` pushes its constant, and an
operator matched in a loop pushes its opcode *after* both operand sub-trees have
already emitted theirs. That parse-and-emit step is what keeps compilation fast
and allocation-light — one growing `Vec`, no tree to build or walk.

## Supported syntax

| Category   | Operators                          |
|------------|------------------------------------|
| arithmetic | `+`  `-`  `*`  `/`                 |
| unary      | `-x` (negation)  `!x` (logical not: 1.0 if x is 0, else 0.0) |
| comparison | `==`  `!=`  `>`  `>=`  `<`  `<=`   |
| grouping   | `( ... )`                          |

## Types

There is exactly one value type: `f64`. There are no integers, strings, or a
distinct boolean — everything on the stack is a double.

Booleans follow the C convention: a comparison does not produce a special bool,
it produces a number — `1.0` for true, `0.0` for false — just like C's relational
operators yield an `int` of `0` or `1`. This keeps the stack machine trivially
simple (one homogeneous value type).

Division by zero is a runtime error.

## Native functions

The big win for the embedded language: expressions can call host Rust
functions, and the boundary is deliberately simple — one `find` function that
maps a name to a `MethodInfo` (id + arity), and one `Callable` trait whose
`call` method receives the id and an `&[f64]` argument slice. No FFI, no code
generation, no plugin system:

    sub(sum(1.0, 2.0), 3.0)     // reaches straight into your Rust program

That is the payoff of embedding — see `examples/cli.rs` for the minimal wiring
and `examples/symbolic_regression.rs` for host calls driving a fitness loop.

## Example

```rust
let value = xpr::Interpreter::compile_and_run("2 + 3 * 4", find, &callable)?; // Ok(14.0)
let value = xpr::Interpreter::compile_and_run("(1 + 2) * 3", find, &callable)?; // Ok(9.0)
xpr::Interpreter::compile_and_run("6 / 0", find, &callable); // Err(DivisionByZero)
```

where `find` maps a function name to a `MethodInfo` and `callable` implements the
`Callable` trait — see the doc example on `xpr::Interpreter` and
`examples/cli.rs` for a complete wiring.

## Roadmap

- **`if`** — conditional expressions, e.g. `if x > 0 { x } else { -x }`. The natural
  next step now that comparisons already yield a usable `1.0`/`0.0` and `!x`
  yields the negated condition.
- **`for`** — loops, to grow from single expressions into small programs.

## License

Apache-2.0 — see [LICENSE](LICENSE).
