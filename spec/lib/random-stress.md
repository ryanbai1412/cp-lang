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
- A unary `@check fn ok(input: In, out: Reader) -> Bool` runs on every
  implementation's output, for problems with many valid answers. Pair it
  with a brute that compares feasibility, since a unary checker cannot
  catch a wrong "impossible".
- Floats compare with `close(a, b, abs:, rel:)`.

## Interactors

`@interactor fn judge(w: Wire)` is the other end of the pipes for an
interactive problem; `cpl stress` and `cpl interact` run the program
against it with generated hidden data.

## Laws

`laws_hold(op, samples)` checks a value's declared laws; stress samples
laws of every operation value the program builds (`lib/ops-ranges.md`).
