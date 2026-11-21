# 23. Optionals

**Layer:** core C4 + library. postfix `?`, `??`, `if let` are core; `!` (the `Unwrap` trait), operator lifting and `Max`/`Min` are library impls.

Status: [proposed].

**Why.** "Not reachable", "not found" and "no answer yet" are the values C++ encodes as `-1`, `INF` or
`LLONG_MIN`. A sentinel can be added to, compared and printed by mistake; `?T` can't, and it costs no bytes
when `T` has a niche. Each type has at most one niche, so `?(?T)` adds a one-byte flag around `?T` [2impl Q27].

| layer | pieces |
|---|---|
| core (C4) | the type `?T`, niche layout, `x ?? d` (lazy right side, `Never`), `if let`/`while let`, postfix `x?` and its extent rule |
| library | `x!` via `Unwrap` (23.1), operator lifting (23.2), `Eq` (structural), the `Max`/`Min` monoids with identity `none`, `max=`/`min=` skipping, `.flatten()`, `.all_some()` |
| sugar | `f(x?)` ≡ `if let v = x: f(v) else: none`, at the extent boundary |

**Rejected.** Sentinels (the bug class); NaN-like silent propagation through comparisons (C# lifted `<`);
`?.` as a separate operator (postfix `?` covers it).

## 23.1 Optionals

- `?T` is `T` or `none`. `T` promotes to `?T` implicitly. Niche representation: `?V`, `?Int` and range-typed
  `?Int` take no extra bytes (goal III; `Int` is i64 with a niche) [r10 10].
- **`x!` is the `Unwrap` trait** [proposed]: `trait Unwrap: type Out; fn unwrap(self) -> Out`, with `!` as its
  postfix operator. `?T` implements it (`Out = T`, traps on `none`); so do `[D]?T` (`Out = [D]T`) and
  `List<?T>` (`Out = List<T>`), which check every element in one pass and trap naming the first gap:
  "depth[17] is none". This ends a fill phase:
  ```cp
  let depth = depth!                  // [V]?Int → [V]Int, frozen from here on
  ```
  Cost: one pass, no copy. `?T` and `T` share a layout when `T` has a niche, so a dead `var` source moves its
  buffer; otherwise the elements are copied once. `.all_some()` stays the non-trapping form (`?[D]T`).
  Rejected: `!` as core-only (then arrays need a differently named method for the same idea).
- `x ?? d` defaults; `if let y = x where c:` and `while let y = x where c:` bind [p2 S8]. A
  tuple is not a `?` boundary, so `best max= (dp[j]? + w, j)` still skips; `?T` is not `Ord`, so tuples with
  optional fields don't order [p2 Q3 declined, r14 T1, T11].
- **Operators lift** [you, p3]: every arithmetic and bitwise operator (`+ - * / % & | ^ << >>`, unary `-`)
  lifts over `?T`, for any type that implements it: `none + 5` is `none`, and `?Int + ?Int` is `none` if
  either is. This is not NaN: the result is `?T` in the type, and printing or storing it as `T` needs an
  explicit `??`, `!` or `if let`. **Ordering doesn't lift**: `?T` is not `Ord`, so `dp[j] + c < best` is an
  error (C# lifts `<` to a silent `false`). `==` stays structural (`none == none`), as map keys and `@memo`
  need. Lint: `==` where one side is lifted arithmetic and both sides can be `none` ("both unreachable
  compare equal"). `min`/`max` on `?T` are the `Min`/`Max` monoid operations, where `none` is the identity,
  consistent with `max=` [r10 9]. So `max(?T, T)` is a `T`, with no seed: Fast B writes
  `max(incl[j], run[x] + excl[j])` with `incl: [prev]?Int` [r11 N36].
- **Postfix `x?`** [you]: needed only where operators don't reach: indexing `a[x?]`, call arguments `f(x?)`,
  fields `x?.len`, and tuples `(dp[j]? + w, j)`. If `x` is `none`, the nearest enclosing parenthesized group, call argument, left
  operand of `??`, lambda or `fn … =` body, or statement right-hand side is `none` [r11 N36]. A lambda whose
  body uses `?` returns `?T`, so `filter(x => a[x]? > 0)` is a type error, not a silent skip [r12 R7]. The `??` boundary is what makes
  `prev?.after ?? start` work [r10 23]. `t?.size` is this rule followed by `.size`, not a separate form.
  A `?` in an `if` or `while` condition is an error; write `… ?? false` so the fallback is visible [p1].
- **No `return` out of a lambda** [p1 Q2, r13 P7]: `return` in a lambda returns from the lambda. Early exit on
  `none` is `let ys = xs.map(f).all_some() ?? return none`, or a `for` loop.

  ```cp
  dp[b] max= dp[j] + a[j .. b].max()!            // + lifts; max= skips none
  ```
  The editor highlights the extent. A `?` whose extent is a whole statement that isn't assigned to an
  optional or a `max=`/`min=` target is an error ("the result of `x?` is dropped").
- **`max=` / `min=` treat `none` as the identity** [you]: a `none` target takes the right side; a `none` right
  side leaves the target unchanged. With a plain `Int` target and `?Int` right side, `none` means skip.
  Together with lifted operators this is exactly "skip unreachable states".
- `Max<T>` and `Min<T>` are monoids on `?T` with identity `none`, so segment trees need no `Inf` sentinel.

## 23.2 [p4] lifting is three impls
```cp
impl<T: Add> Add for ?T:                  // likewise every arithmetic and bitwise trait
    fn add(a, b) -> ?T = if let x = a, y = b: x + y else: none
impl<T: Eq> Eq for ?T                     // structural: none == none
// no `impl Ord for ?T`: that's "ordering doesn't lift"
```
The "operators lift" rule in 23.1 is these impls, not a language rule. `max=`/`min=` skipping is the
`Max`/`Min` monoid on `?T` (chapter 27).
