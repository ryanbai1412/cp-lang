# Random inputs, alternatives and checkers

Status: draft for 0.3.0. `std.rand` and the stress roles. Sources:
archived `design/spec/30-stress.md`, `plan/50-debugging.md`; decision
146 for laws. The tool that drives them is `tools/stress.md`.

## Randomness

- `Rng(seed)` is a deterministic splitmix64 stream.
  `rng.int(lo, hi)` (both ends inclusive), `rng.float()`, `rng.pick(xs)`,
  `rng.shuffle(mut xs)`, `rng.perm(D)`. There is no time-seeded `Rng`:
  programs that need randomness on the judge pass a seed they choose, so
  every run replays.
- Hashing salts (`HashMap`, string hashing) are fixed per build profile,
  not drawn from an `Rng`.

## Typed generators

`gen<T>(...)` generates any `Read` type within its contract, so the reads
are the input spec:

```
gen(1..=8)                         // an Int
gen<[V]1..=1e9>()                  // an array over a fresh domain
gen<Str of "ab">(len: 1..=10)
gen<Graph<V, (to: V, w: 1..=9)>>(simple: true, connected: true)
gen<Rooted<V>>(shape: random)      // also path, star, caterpillar, binary
gen<Perm<V>>()
```

- Constraints types cannot state go in `@gen fn input()`, which prints
  the input exactly as the judge would, using `gen` calls.
- Every `gen` choice is recorded, so the shrinker minimizes any failing
  case without a per-problem shrinker: it shortens collections, lowers
  numbers toward their range's low end, and removes edges, keeping only
  inputs that still satisfy every read contract.
- Sizes come from `with stress(max_size: ..., time: ...)` or the caps on
  an `@alt`.

## Alternatives

```
@alt(count, n: ..=8)
fn count_slow(xs: List<Int>) -> Int: ...
@alt(program, n: ..=10)            // a whole-program brute
fn brute(): ...
```

- `@alt(f, caps)` marks another implementation of `f`; stress compares
  them on generated inputs. Inside the alt, a cap refines the capped
  value's type (`n: 1..=8`).
- An alt of a nested function is substituted at the real call sites
  during whole-program stress, so its preconditions hold; stressing it
  alone needs its own `@gen`.
- Alts are stripped from submissions.

## Checkers

- A relation `check: (input, ours, theirs) => Bool` on an `@alt`;
  equality is the default.
- A unary `@check fn ok() -> Bool` runs on every implementation's
  output, for problems with many valid answers. Inside it `read()` reads
  the test's input and `read_output()` reads the output being checked,
  token by token like `read`, with the same typed forms (`read_output<T>()`,
  `let x: T = read_output()`); it rejects by returning `false` or by a
  failing `assert`. `cpl test` and `cpl judge d input output` use it in
  place of an exact compare (decision 375). Under `@!multitest` it is shared code above
  the divider and runs once per test, on that test's input and output. Pair it
  with a brute that compares feasibility, since a unary checker cannot
  catch a wrong "impossible".
- A checker sees only the input and the output, never the solution's bindings: a fact
  it trusts (`let m = n - k + 1`) is restated from `input`, so a wrong
  derivation in the solution cannot pass its own check (decision 374).
- Floats compare with `close(a, b, abs:, rel:)`.

## Interactors

`@interactor fn judge(mut sol: Interactor)` is the other end of the pipes
for an interactive problem. Its `read()` reads the hidden test,
`sol.send(x)` prints to the program and `sol.recv()` reads the program's
next token at the expected type; a failing `assert` rejects the run.
Under `@!multitest` it runs once per test, and the harness, not the
interactor, sends the program the test count (R10.4). `cpl run` runs both sides with
its input as the hidden test, and `cpl stress` generates the hidden test
from the interactor's reads. The program side is ordinary I/O with
`Wire()` (R10.4) (decision 375).

## Laws

`laws_hold(op, samples)` checks a value's declared laws; stress samples
laws of every operation value the program builds (`lib/ops-ranges.md`).
