# 30. Stress, alternatives, generators

**Layer:** library. `@alt`, `@gen`, `@check` are role markers declared by the stress library (chapter 14.4) and consumed by `cp stress`; they're stripped from submissions. Status: [proposed].

## 30.1 Alternatives

```cp
@alt(count_pairs, n: ..=8)
fn slow(xs: List<Int>) -> Int: ...
```

- `@alt(f)` marks another implementation of `f`: a brute or a second fast algorithm. `cp stress` runs each
  against `f` on generated inputs.
- **Size caps** name a program-level read or domain (`A: ..=8`) or, qualified, a `gen` binding
  (`input.n: ..=10`) [r4 16]. An uncapped alt uses the `stress` context's generic defaults
  (`with stress(max_size: 1000, time: 10s)`); caps live only on `@alt` [r10 22]. When both are static, a cap
  that excludes every generated input is a compile error; otherwise stress checks at start-up [r10 Q4.1].
- **Inside the alt, a cap refines the type** of what it names (`n: 1..=8`), so `Bits<0..bets.len>` meets
  `Bits`'s static ≤ 64 rule and widths shrink [r11 N3]. Entering a capped alt converts each capped value with
  a check, trapping with "input exceeds the alt's cap `n: ..=8`"; stress only generates within caps [r12 R3].
- An inner function's alt is substituted at the real call sites in whole-program stress and compared there,
  so its preconditions hold. Stressing it alone needs its own `@gen` [r4 19, 26].
- A function that mutates captured state can't have an `@alt`, except state declared `scratch var`: a buffer
  whose contents don't survive between calls. Debug poisons it on entry from outside the function's recursive group, never on recursive calls; `@memo`
  functions can't touch `scratch`. Stress runs twice with different poison patterns, so a result that
  depends on stale contents shows up [p1 Q4, r13 P9].
- **Whole-program alts** [p1 F9]: `@alt(program, caps)` on a function prints like the program; stress compares
  output. Under `--- with multitest() ---` it replaces the per-test closure, so the brute is written for one
  test and declared above the line (16.3 rule 7).
- Alts may capture per-test state (they are substituted where `f` is called); other tool roots can't. An alt of
  a nested function is declared in the same block, so it sees the same captures [p3 proposed].

## 30.2 Checkers

- **One shape, a relation:** `check: (input, ours, theirs) => Bool`, passed in `@alt(…, check: …)`. Equality
  is the default.
- **Unary:** a program-level `@check fn ok(input: In, output: Reader) -> Bool`, where `In` is the `@gen`
  return type or a `Reader` over the input text (for a grader: the argument tuple and return type) [r10 25]. It runs on every implementation's
  output; the relation runs between `f` and each alt [r4 17].
- A unary checker can't catch a wrong "impossible", so problems with an impossible answer also get a brute
  `@alt` with a relation comparing feasibility [r3].
- Floats: `approx(a, b, abs:, rel:)` is an ordinary function and the only float equality [r7].

## 30.3 Generators

- One system, `gen<T>(…)`, driven by types: `gen(1..=8)`, `gen<[V]1..=1e9>()`, `gen<Graph<V>>(simple: true)`,
  `gen<Perm<1..=n>>()`.
- Constraints types can't state go in `@gen fn input()`, which writes the input with `print` exactly as the
  judge would.
- The shrinker records every `gen` choice and minimizes them, so a failing case shrinks without a
  per-problem shrinker.
- `cp stress` reports: the smallest failing input, each implementation's output, and the first differing
  line, with values in the file's base.

## 30.4 Role markers

`@name(args)` attaches a library-declared role to the next declaration.

| marker | on | consumed by | what it does |
|---|---|---|---|
| `@alt(f, caps)` | fn | `cp stress` | another implementation of `f` (brute or second fast); stripped from submissions |
| `@check` | fn | `cp stress` | unary output checker |
| `@gen` | fn | `cp stress` | input generator for constraints types can't state |
| `@interactor` | fn | `cp stress` | the other end of the pipes for interactive problems |
| `@grader` | fn | bundler | exports a function with the judge's signature |
| `@memo` | fn | library | library decorator, chapter 14 |
| `@cpp("…", includes:, target:)` | fn | bundler | C++ override of an in-language body |

- The declaring library defines the role: a tool consumes it, or it wraps the declaration. A marker never
  affects code it isn't attached to.
- Tool roots (`@interactor`, `@gen`, `@check`) may capture only contexts, bindings above every wrap line, and statically sized types
  [r6 4].

Code Jam's `Case #{i}: ` is `--- with multitest(prefix: i => "Case #{i}: ") ---` (16.3) [p1 F8, r13 P17]. `@judge_fn` and `@mock` join the marker table.
