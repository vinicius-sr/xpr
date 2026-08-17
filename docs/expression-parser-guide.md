# Building an Expression Parser That Follows Normal Math

Step-by-step guide to restructure `examples/expression.rs` from its current
single-level scanner into a recursive-descent parser with operator precedence.
You implement every step. This file only specifies *what*, *why*, and *how to
verify* — it deliberately contains no finished function bodies.

**The goal:** `"10 * 12 + 15"` must evaluate to **135**, not 270.

---

## The big picture (read this first)

A language implementation is a pipeline:

```
source text ──► scanner ──► parser ──► instructions (RPN) ──► evaluator (stack VM)
```

Your current file mixes scanning and parsing in one set of functions and emits
instructions as it goes. That's fine for a first version — but it hides *where*
decisions are made, which is exactly why the math went wrong. The restructure
keeps your existing machinery (`Scanner`, `buffer`, `next()`, `consume()`,
`OpCode`) and only reorganizes the parsing functions into three levels:

```
scan ──► expr  ──► term  ──► factor
          │          │          │
          │          │          └─ one value: number, x(n), pdiv(...), ( ... )
          │          └─ multiplication level:   factor { '*' factor }
          └─ addition level:                    term  { ('+'|'-') term }
```

Two rules produce normal math:

1. **Precedence = which level owns which operator.** `*` lives in `term`, one
   level below `expr`. When `expr` parses the right side of a `+` by calling
   `term`, any `*` inside it gets consumed *there*, before `Add` is pushed.
   Tighter operators are simply handled deeper.
2. **Left-associativity = a loop, not a single recursion.** `10 - 3 - 2` must
   be `(10-3)-2`. A `while` loop at each level does that; a single recursive
   call right-nests — which is your current 270 bug.

### Emission order: a decision

You currently store instructions in prefix order and read them *reversed* to
execute. Your mental model was correct — both views describe the same tree.
But from now on we emit **forward RPN** (execution order): operands first,
operator after both sides are complete. Then `instructions` is directly
executable front-to-back and nobody has to remember to reverse it.

Consequence: every existing test's expected vector changes order, e.g.
`[Pdiv, Const(12.), Const(3.)]` becomes `[Const(12.), Const(3.), Pdiv]`.

### The tree we are aiming for

```
"10 * 12 + 15"

          Add            ← pushed last: lowest precedence
         /   \
      Mult    15
      /  \
     10   12

RPN: 10 12 * 15 +
vec: [Const(10.), Const(12.), Mult, Const(15.), Add]
eval: 135
```

## Rules of the road

- After **every** step run `cargo test --example expression`. Each step ends
  with a checkpoint stating exactly what must pass.
- Never leave a previously-green test red. If a step regresses something, stop
  and fix it before moving on.
- When stuck, trace by hand: a table with columns
  `c | buffer | instructions | next action`. That skill is the real lesson.
- Keep the **buffer invariant**: `buffer` holds at most one pending token
  (identifier or number). It must be empty when any level's loop checks for an
  operator, and `factor` must leave it empty on return.

---

## Step 1 — Clean baseline

Remove the noise so you can see the machine:

1. Delete every debug `println!` (in `expr`, `operator`, `move_or_eof`,
   `factor`, `next`).
2. Delete `move_or_eof` — it is never called.
3. Inline the `func!` macro into its single use site (the `pdiv` arm). A
   one-use macro is indirection, not abstraction.
4. Fix the two stale assertions in `test_div_simple` and `test_x_simple`: they
   expect `scanner.c == ')'`, but `consume(')')` advances *past* it, so at end
   of input `c` is `char::MIN`. Assert that instead.

**Checkpoint:** 6 tests pass (empty, whitespace, div_simple, x_simple,
pdiv_complex, pdiv_x). The 4 binary tests fail with `InvalidTerm("")`. That is
your known-broken set.

## Step 2 — Understand the two bugs (no code)

Before changing any structure, hand-trace these two inputs through the current
code and identify why each fails. Do it on paper first, then compare.

**Trace A: `"10 * 12"`**

| # | c   | buffer | what happens                                                        |
|---|-----|--------|---------------------------------------------------------------------|
| 1 | `1` | ``     | `scan` → `next()`                                                   |
| 2 | `*` | `10`   | `expr`: `fill_buffer` stops at `*`                                  |
| 3 | `*` | `10`   | match `*` → `operator(Mult)`: pushes `Mult`                         |
| 4 | `*` | ``     | `factor` consumes the buffer → `Const(10)` — **but `c` still points at `*`; nobody advanced past it** |
| 5 | `*` | ``     | recursive `expr` sees `*` again → `operator(Mult)` again → `factor` on an empty buffer → `InvalidTerm("")` |

**Bug 1:** `operator()` never consumes the operator character.

**Trace B: `"x(0) * 12"`** (why `main`'s source fails)

The `x` arm of `factor` consumes through `)`, leaving `c == '*'`. But `expr`
only looks for operators *at entry* — after `factor` returns, it just returns.
The `*` dangles; `scan`'s second `expr()` call finds an empty buffer and
crashes with `InvalidTerm("")`.

**Bug 2:** a factor that consumes a group can leave a trailing operator that
nobody ever inspects.

**Why not just patch `operator()`?** Adding the missing advance fixes the
crash, but every operator would still sit at one level with right-nesting —
`"10 * 12 + 15"` would still give 270. The restructure is what buys normal
math; the patch only stops the bleeding.

## Step 3 — Write the spec (tests first)

Decide the target behavior and encode it as tests *before* touching the
parser. Update every test's expected vector to forward-RPN order, and add
value assertions through a tiny evaluator you write yourself:

```rust
/// Stack VM: executes `program` front-to-back over input variables `vars`.
/// Const(v) / Read(i) push; binary ops pop the right operand (top), then the
/// left, and push the result. Returns the single value left on the stack.
fn eval(program: &[OpCode], vars: &[f64]) -> f64 { /* you write this */ }
```

Pick `Pdiv` semantics deliberately — plain division, or PonyGE2-style "return
0.0 when |right| < ε" — and keep the tests consistent with your choice.

Target table (shorthand: `C10` = `Const(10.)`, `R3` = `Read(3)`):

| input                          | instructions (forward RPN)                    | value        |
|--------------------------------|-----------------------------------------------|--------------|
| `10 * 12`                      | `[C10, C12, Mult]`                            | 120          |
| `10 * 12 + 15`                 | `[C10, C12, Mult, C15, Add]`                  | **135**      |
| `2 + 3 * 4`                    | `[C2, C3, C4, Mult, Add]`                     | **14** (precedence) |
| `2 * 3 + 4 * 5`                | `[C2, C3, Mult, C4, C5, Mult, Add]`           | 26           |
| `10 - 3 - 2`                   | `[C10, C3, Sub, C2, Sub]`                     | **5** (left-assoc) |
| `x(3) - 2.4`                   | `[R3, C2.4, Sub]`                             | vars[3] − 2.4 |
| `pdiv(12, 3)`                  | `[C12, C3, Pdiv]`                             | 4            |
| `pdiv(pdiv(12,3), pdiv(3,56))` | `[C12, C3, Pdiv, C3, C56, Pdiv, Pdiv]`        | ≈ 74.67      |

The three bold rows are the ones your current design cannot produce — they are
the point of the exercise.

**Checkpoint:** almost everything is red now (the pdiv tests too, because their
vectors changed order). Don't panic — this is your target list. The
empty/whitespace error tests stay green.

## Step 4 — `factor()`: where values are born

Move value-parsing out of `expr` into a dedicated function:

```rust
fn factor(&mut self) -> Result<(), ExpressionError> {
    // 1. if self.c starts a token (is_alpha), call fill_buffer()
    // 2. dispatch on the buffer contents — the match you already have:
    //      "pdiv" → ...     "x" → ...     _ → number
    // 3. leave the buffer empty on return
}
```

- The `pdiv` / `x` / number arms are code your current `factor()` already has —
  *move* it, don't rewrite it.
- In the `pdiv` arm, push `OpCode::Pdiv` **after** both arguments are parsed
  (forward RPN).
- Remove `fill_buffer` from `expr` (it now lives in `factor`) and delete
  `operator()` entirely.
- Make `expr` a stub for now: it calls `self.term()?`, where `term` is also a
  stub that calls `self.factor()?`. Two one-line stubs keep everything compiling.

**Checkpoint:** `pdiv(12, 3)`, `x(12)`, and the nested-pdiv tests are green in
the new order. The binary tests are still red.

## Step 5 — `term()`: the multiplication level

```rust
fn term(&mut self) -> Result<(), ExpressionError> {
    self.factor()?;
    // TODO: loop while self.c == '*':
    //     a. advance past '*'
    //     b. parse the right side with self.factor()
    //     c. push OpCode::Mult
    Ok(())
}
```

The loop is the lesson: each iteration handles one more `*`, so
`10 * 2 * 5` becomes `(10*2)*5`. Before coding, write on paper what
instruction vector that input should produce — and why the loop (rather than a
single recursive call) gives you that shape.

**Checkpoint:** `10 * 12` is green. Add a test for `10 * 2 * 5` →
`[C10, C2, Mult, C5, Mult]` = 100. Anything containing `+`/`-` is still red.

## Step 6 — `expr()`: the addition level

```rust
fn expr(&mut self) -> Result<(), ExpressionError> {
    self.term()?;
    // TODO: loop while self.c is '+' or '-':
    //     a. remember which op, then advance past it
    //     b. parse the right side with self.term()   ← not factor!
    //     c. push the op you remembered
    Ok(())
}
```

Why does step (b) call `term` and not `factor`? Because the right side of a `+`
may contain `*`: in `2 + 3 * 4`, the multiplication must be fully consumed
inside that `term` call before `Add` is pushed. That one choice *is* operator
precedence.

**Checkpoint:** all core tests green, including 135, 14, and 5. Run
`cargo clippy --example expression` — no dead code left.

## Step 7 — Straighten out `scan()` and `main()`

- Delete the double-`expr()` hack in `scan()`. New shape: prime the first char
  (`next()`, error on empty input), call `self.expr()` once, then require
  `c == char::MIN` — anything left over is an error. Add a dedicated variant to
  `ExpressionError` for it (e.g. `TrailingInput { current: char }`). This also
  gives you a real error for `"10 12"` instead of the old silent two-term
  behavior.
- Replace `main`'s source with something the language actually supports yet,
  e.g. `"pdiv(x(0) * x(3), 81.34) - x(1)"`. Print the instruction vector and
  its `eval` value for a sample `vars` slice.

**Checkpoint:** `cargo run --example expression` prints a program and a number;
all tests green.

## Step 8 — Extensions (any order; this is how languages grow)

Each item is one small, isolated change:

1. **`/` operator** — one more arm in `term`'s loop plus an `OpCode::Div`
   (or reuse `Pdiv`).
2. **Parenthesized groups** — a new first case in `factor`: if `c == '('`,
   advance, call `self.expr()`, then `consume(')')`. This is where recursion
   pays off: `(x(0) + 1) * 2` now parses. Note `'('` is not alpha, so it never
   enters the buffer — check `self.c` before the buffer dispatch.
3. **More functions** — `psqrt(x)`, `np.sin(x)`, ... are new arms in `factor`:
   consume `(`, parse N `expr` arguments separated by `,`, consume `)`, push
   the op. Once you've added enough of them, your *original* `main` source
   parses — that's the finish line for this file's origin story.
4. **Unary minus** — harder: a leading `-5` in `factor` needs a new
   `OpCode::Neg` and evaluator support. Decide what `10 - -5` should mean
   before coding.
5. **Error positions** — track a character index in `Scanner` and include it in
   every error variant. Ten lines that save you hours once evolved programs
   start failing.

---

## When you're stuck

1. Trace on paper: `c | buffer | instructions | next call`.
2. Check the buffer invariant (Rules of the road).
3. Ask which *level* should own the current character. If two levels could both
   claim it, your grammar is ambiguous — that's a design bug, not a coding bug.
