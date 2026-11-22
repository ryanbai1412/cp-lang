# cpl-prover

The prover behind `invariant`, `law` and the range checker. It answers one
question, "does this hold for every value the variables can take?", and
gives one of three answers: proved, a concrete counterexample, or "not
proved" with the reason. It is pure Rust with no dependencies and no
external solver, and a typical query takes tens of microseconds.

Try it:

```
cargo run -p cpl-prover --example tour --release
```

## What it proves

**Index bounds.** Declare what is in scope, then ask.

```rust
use cpl_prover::Context;

let mut ctx = Context::new();
ctx.var("H", "1..=1000")?;
ctx.var("W", "1..=1000")?;
ctx.var("r", "0..H")?;
ctx.var("c", "0..W")?;
ctx.check_index("r * W + c", "0..H * W")?;   // proved
ctx.check_index("c * W + r", "0..H * W")?;   // fails for H = 1, W = 2, r = 0, c = 1
```

Facts come from `assume` (a dominating `if` or `while`, an `assert`, a
`requires`, an `invariant`). `assign` and `havoc` end the facts about the
old value of a variable, as an assignment or a call that writes it does.
Arrays and unknown functions are declared by the type of their results;
equal arguments give equal results, and nothing else is assumed.

**Overflow freedom.** `check_overflow` reports every arithmetic operation
of an expression, with the range it can take and inputs that overflow
when there are some. `check_sum` does the same for a loop accumulator.

```rust
ctx.array("a", "0..=1e9")?;
ctx.array("b", "0..=1e9")?;
ctx.check_overflow("a[i] * b[i] + n")?;      // both operations: cannot overflow Int
ctx.check_sum("a[i] * b[i]", "n")?;          // can overflow Int: n = 200000, a[i] * b[i] = 46116860184274
```

`check_traps` adds division by zero and `x!` on `none`. Each operation is
checked under the `if`, `and` or `or` that guards it.

**`law disjoint`.** Two lends of one container follow the rules of
`plan/19-index-and-lends.md` §3.6: reads never conflict, different fields
are separate, different projections of one receiver always conflict, and
two uses of the same projection are allowed exactly when its law is
proved from the facts in scope.

```rust
use cpl_prover::{Law, Lend};

let law = Law::parse("law disjoint(i, j) = i != j")?;
let (x, y) = (Lend::parse("mut cnt[i]")?, Lend::parse("cnt[j]")?);
ctx.check_lends(&x, &y, Some(&law))?;        // conflict: `i != j` does not hold for n = 1, i = 0, j = 0
ctx.assume("i < j")?;
ctx.check_lends(&x, &y, Some(&law))?;        // allowed: the law is proved
```

`check_disjoint_law` checks the law itself against the cells a lend
touches, and refutes a wrong one with two lends it accepts and the cell
they share.

**Invariants.** `check_loop` proves each invariant on entry and after the
body, from any state that satisfies the invariants and the guard.

```rust
use cpl_prover::{Loop, Step};

let window = Loop::new("a[r] - a[l] > k")?.invariant("l <= r")?.step(Step::assign("l", "l + 1")?);
ctx.check_loop(&window)?;   // proved when k >= 0; with a negative k: fails for n = 1, k = -1, r = 0, l = 1
```

**Algebraic laws.** `Structure` holds the laws of `spec/std/algebra.cp` in
the words they are written in there: `semigroup`, `monoid`, `group`,
`action`, `order`, plus `commutative`, `idempotent`, `distributive` and
any law you add with `with_law`.

```rust
use cpl_prover::Structure;

let report = Structure::semigroup().check(
    &[("T", "(sum: Int, pre: Int)")],
    &[("op", "(x, y) => (sum: x.sum + y.sum, pre: max(x.pre, y.sum + y.pre))")],
)?;
println!("{report}");
```

```
Semigroup breaks law assoc
  a = (sum: 0, pre: 0), b = (sum: 0, pre: 0), c = (sum: 1, pre: 0)
  op(op(a, b), c) = (sum: 1, pre: 1)   op(a, op(b, c)) = (sum: 1, pre: 2)
```

## What an answer means

A goal is proved over the mathematical integers, with every `Int`
variable in `Int.MIN..=Int.MAX`. That covers every run that does not
trap. A run that traps is not a failure of the goal: `a + 1 > a` is
proved, and `check_overflow("a + 1")` is what reports `a = Int.MAX`.

A counterexample is never the solver's word alone. Every candidate is run
through a separate evaluator (`eval.rs`, which shares no code with the
solver): the variables are of their types, the facts hold, the goal is
false, and nothing overflowed on the way. It is then shrunk toward zero.

"Not proved" means the goal left the fragment the solver decides and
sampling found no failure either. For `law disjoint` that is a conflict:
the plan asks for a proof, never a guess.

## How it works

1. **Lowering** (`lower.rs`) turns a term into polynomial constraints, and
   does most of the proving by normal form. Integer arithmetic becomes a
   polynomial, so `+` and `*` associate, commute and distribute by
   construction. `min` and `max` are flattened, sorted and distributed
   over `+`, which is the whole algebra of segment-tree monoids. `%` is
   reduced modulo its divisor before it is named, so modular sums and
   products associate. `^`, `&`, `|` and `gcd` are flattened and folded.
   What is left (an `if`, a quotient, an array element) becomes a fresh
   variable with a defining constraint that sleeps until it is needed.
2. **Bounds** (`solve.rs`) keep an interval for every linear form in play
   (`i`, `j - i`, `r*W + c`). This octagon-like layer decides most
   disjuncts without solving anything. Equalities are solved for a
   variable and substituted, so the branches of an `if` collapse into
   statements about the inputs.
3. **The Omega test** (`omega.rs`) decides what remains: a complete
   procedure for linear integer constraints, with exact equality
   elimination and dark shadows for the gap between rationals and
   integers. Products of variables are bounded by multiplying pairs of
   known constraints, which is what proves `r*W + c < H*W`.
4. **Confirmation** (`context.rs`, `eval.rs`) evaluates each candidate
   counterexample, and falls back to sampling the inputs when the
   arithmetic is beyond the solver (bit tricks, high degree).

`Context::to_smtlib` writes any query as an SMT-LIB 2 script. A test
feeds a few to `z3` when it is on the `PATH`; nothing needs it.

## Tests

`cargo test -p cpl-prover` runs the unit tests of each layer, the cases of
`plan/19-index-and-lends.md` (`tests/plan19.rs`) and of `decisions.md`
(`tests/decisions.rs`), the standard library's structures and the ways
they go wrong (`tests/laws.rs`), and differential tests that compare the
prover with brute force on thousands of random goals (`tests/fuzz.rs`).
