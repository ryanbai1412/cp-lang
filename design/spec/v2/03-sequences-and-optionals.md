# Spec chapter 3: sequences, folds and optionals

Status: **draft v2** [proposed].

## 3.1 Three kinds of sequence

| kind | examples | re-iterable | indexable | stored |
|---|---|---|---|---|
| stream | `read` loops, `successors`, `gen` | no (one-shot) | no | no |
| view | `g.edges`, `x.map(f)` / `x.filter(p)` on a view, `xs.pairs()`, `s.windows(m)` | yes | no | no |
| array / slice | `[D]T`, `List<T>`, `a[l .. r]`, `g.adj(v)` | yes | yes | yes |

- `map` and `filter` on an **array** build an array (reusing the input's buffer when the input is a
  temporary and the element fits in the old one; `cp explain` reports misses) [r10 27]; on a **view** they build a
  view. A domain counts as the array of its positions, so `V.map(f)` builds `[V]T` [r4 5].
- `Range<D>.map(f)` is a view that keeps `D` as its index, so a scan over it is indexed by `D` [r9 9].
- Accessors return copies of values that fit in a register.
- `cp explain` lists every allocation.

## 3.2 Invalidation

Only length-changing operations on a `List` (`push`, `pop`, `resize`, `clear`) invalidate, and only views and
slices of that same `List`. Element writes never do.
- When the access paths are syntactically the same (same root variable, same index expressions, no write
  in between), it is a compile error. `let r = adj[u]` is a slice, not an alias of the `List` [r10 26].
- Otherwise debug checks a per-`List` generation count: `adj[w].push` while iterating `adj[u]` traps only if
  `w == u`.
- Views and slices can't be stored in struct fields or statics. Stdlib types (`Graph`, `Buckets`) store
  offsets and build slices on access; user libraries can do the same [r6 17].
- A slice is (pointer, length) into the element buffer, never a borrowed `List` header [r6 15].

## 3.3 Folds and scans

- One spelling per idea: `xs.map(f).sum()`, never `sum(by:)` [r8 9]. `max_by(key)`/`min_by(key)` return the
  element, `max()`/`min()` the value.
- Folds follow the scan principle [p2 S1]: `sum`, `count(where:)`, `all`, `any` have an identity and are total (a bare
  `xs.count()` names `.len`) [r14 T7];
  `max`, `min`, `reduce`, `first`, `last` return `?T` over a possibly-empty sequence and `T` over a nonempty one
  (chapter 1.10). List literals are nonempty; `map` keeps nonemptiness, and `flat_map` keeps it only when `f`
  returns `NonEmpty<…>` [r14 T2].
- **Ties** [p2 S4]: `max_by`/`min_by` return the first extremum; `max_by(key, last: true)` the last. `best max= (v, who)`
  compares every field, so ties go to the larger `who` [r14 T10].
- **Combinators** [p2 F1]: `xs.flat_map(f)`, `chain(a, b, …)`, `product(a, b)`, `xs.combinations(2)`,
  `xs.split_by(p) -> (yes, no)` [r14 T21], `(lo..=hi).step(k)` (traps on k ≤ 0). `flatten` concatenates a sequence
  of sequences and has one overload that skips `none` in a sequence of `?T`; `?T` itself is not a sequence [r14 T6]. All fuse (3.6), with early exit under `take`/`first`/`any`.
- **Selection** [p2 F2]: `xs.min_k(k, by:)` / `xs.max_k(k, by:)`, O(n), unordered (`.sorted()` is explicit), ties to the
  lower index [r14 T20]; a domain `k` indexes the
  result (`[Pick]Shop`).
- **Streams** [p2 F3]: `successors(first: ?T, next: T -> ?T)` is a one-shot stream; with `take_while` it replaces
  functional-graph walks and backward reconstruction. The eager-lazy lint never fires on a stream.
- **Ranges are always views** [p2 S10]: `(1..=n).map(f).sum()` never allocates; arrays and `List`s are the eager types.
- **Scans** [r8 4]: `prefix_sums`, `prefix_max`, `prefix_min`, `suffix_max`, `suffix_min`.
  - Over a slice: indexed by its elements, inclusive, total (`a[today].prefix_max(): [today]Int`).
  - `at: .cuts`: indexed by the slice's cuts; the empty prefix/suffix is `none`
    (`a[l .. r].suffix_max(at: .cuts): [l ..= r]?Int`).
  - Over `Range<D>.map(f)`: indexed by `D`; this beats the principle below [r13 P1].
  - **One principle:** a scan is over cuts when its monoid has an identity in the element type `T` itself, excluding `?` [r13 P1] (sums: `0` at the first cut),
    and over elements otherwise (max/min, whose identity is `none`, not a `T`) [r11 N7]. `at:` overrides [r10 Q3.2].
    Range sums are `pre[r.after] - pre[r.before]`, one spelling [r10 19].
- **Seeded scan** [p1 F6]: `xs.scan(init, step)` is indexed by cuts (the seed sits at the first cut), by the
  principle above. `xs.reduce(f) -> ?T` is the seedless fold. `xs.runs(by: key)` yields maximal groups of
  equal keys as nonempty slices, which convert to ranges (§1.8) [r13 P14].
- **Optional elements** [p1 S2, proposed default]: `min`, `max`, `sum` and `reduce` over a sequence of `?T`
  are compile errors that name the two spellings: `.flatten()` skips `none`, and `.all_some()` gives `none` at
  the first `none`. Scans over `?T` (Fast B's `suffix_max`) stay legal, because they keep every position. `max=`/`min=` and the
  `Max`/`Min` monoid are the deliberate skip spellings; their diagnostics name `.flatten()`/`.all_some()` too [r13 P12].
- **Search** [p1 F10]: `partition_point(p)` on a sequence or range of positions returns the **cut** between false and
  true (total); on an `Int` range `lo..=hi` it returns `lo..=hi + 1`. Debug samples `p` at most 20 times to
  check monotonicity, reported by `cp explain` [r13 P23]. A predicate with effects (a `Wire` query, a `var`
  write) is never sampled: it runs exactly ⌈log₂(n + 1)⌉ times, and `cp explain` says "no monotonicity
  check: `p` has effects" [r16 R1]. Eager consumers that evaluate in a fixed order once (`partition_point`,
  `first(where:)`, `fold`) accept effectful lambdas; a `@memo` call counts as pure everywhere [r16 R2].
- **Bounds** [p1 F7]: `take(k)` traps on a negative `k` (like `repeat`) and clamps above `len` [r13 P19]. `List.pop() -> ?T`.
  `xs.count(where: p)` counts without allocating.
- **Eager vs lazy** [p1 F13]: array `map`/`filter` are eager; `xs.view()` makes them lazy. A lint fires when an
  eager result is consumed only by a lazy adapter (`merge`, `take`, `first`, `any`), unless a lambda in the
  chain mutates a capture, since laziness reorders effects [r13 P8].
- **Windows**: `b.window(k)` (cuts), `s.windows(m)` (nonempty slices of length m, each knowing its start).
- **Neighbours** [you]: `xs.pairs()` yields adjacent `(prev, x)`; `xs.with_prev()` yields `(?T, T)` with
  `none` first, used as `let p = prev?.after ?? start`. Loops have no previous-iteration syntax.
- **Sets**: `seen.mark(v)` sets and returns whether it was new (`if not seen.mark(v): continue`). Its result
  may be dropped without a lint.
- **Cyclic** [p2 F5, r14 T17]: `xs.runs(by:, cyclic: true)` merges a wrapping run. Rotations are
  `K.walk(from: k).map(xs)` over a `ring` domain; no separate `rotated`/`cyclic_pairs`.
- Rejected: `roll` for rolling buffers (hides control flow; buffers stay `fill` + `swap`) [r8 10].

## 3.4 Optionals

- `?T` is `T` or `none`. `T` promotes to `?T` implicitly. Niche representation: `?V`, `?Int` and range-typed
  `?Int` take no extra bytes (goal III; `Int` is i64 with a niche) [r10 10].
- `x!` traps on `none`; `x ?? d` defaults; `if let y = x where c:` and `while let y = x where c:` bind [p2 S8]. A
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

## 3.5 Indicators

`Bool` doesn't convert to numbers implicitly. Write `Int(b)` or `Z(b)` [r8 8]:
`print(upto(r) - upto(l) + Z(l.digits().sum() % d == 0))`.

## 3.6 Lowering

| construct | C++ |
|---|---|
| view chain `xs.map(f).filter(p).sum()` | one fused loop, lambdas inlined, no `std::function` |
| array `map` | one allocation, or reuse of a temporary's buffer |
| scan | one pass into a buffer; a loop-local scan with a loop-invariant bound reuses one buffer per loop (Tourism r9: memory 0.93x accepted) |
| `?T` with niche | same width as `T` |
| `x?` | a branch to the extent's `none` result |
| `max=` with `?T` | compare-and-set with a `none` test, branch-free when possible |

## 3.6b Round-3 rules [p3]

- **Lazy adapters take pure lambdas.** A lambda passed to a lazy adapter (`view().map`, `filter`, `where`,
  `take_while`, any stream or range adapter) may not write captured `var`s or call mutating methods. A
  violation is an error that names `.list()` or a `for` loop. Eager array operations are unaffected. A lint
  fires when a `filter` view is consumed twice ("bind `.list()`").
- **Domain `map` is eager, and fusion is unobservable.** `V.map(f)` on a domain, cut domain or `Bits` builds
  `[V]T`. On an integer range it is a view (3.1). Because the lambda passed to a fold is pure (or the map is
  consumed once), the compiler fuses `V.map(f).sum()` into one loop with no array. An eager chain is fused only
  when at most one lambda in it is impure; otherwise it materializes and `cp explain` says so.
- **Semigroup folds.** `Max<T>`/`Min<T>` are semigroups on `T`; their monoid form is `?Max<T>` (identity
  `none`), which is what `max=`, optional folds and `init: none` use. `SegTree.fold(r)` returns `M` (not `?M`)
  when `r` is `NonEmpty` or a whole domain and `M: Semigroup`. The tree is stored over `?M` (niche), paying one
  `none` test per combine; a maybe-empty range still gives `?M`. `Dsu<V, S>` takes `S: CommutativeSemigroup`, so
  a component's min, max or hull needs no `none` identity or `!`.
- **Ties:** `max_by`/`min_by` keep the first extremum. A tuple `max=` compares every field, so
  `best max= (v, -i)` keeps the first and `(v, i)` the last. `ArgMax<K, I>(ties: first | last)` / `ArgMin`
  is the segment-tree payload, with `ties:` required, as with `Heap(order:)`.
- **New folds:** `xs.product()` (identity 1, total) and `xs.fold(init, step) -> S` (total, no buffer). The
  `Xor` monoid. `successors(first: T, …)` with a non-optional `first` is `NonEmpty`.
- `.list()` materializes streams too. `xs.sorted(by:)` on `[D]T` gives an anonymous `[D.count]T`.
- `zip` traps on unequal lengths; `zip(…, shortest: true)` stops at the shorter input. `List.chunks(k)`.
- `_ = f()` discards a result explicitly.
- **Niches are fixed per type, never per use.** `?Int` is always `INT64_MIN`, which makes `max=` branch-free.
  For a range type, the niche is `hi + 1` when it fits, else `lo - 1`. `Min<Int>` storage in a tree uses its own
  payload type with `INT64_MAX` reserved. `cp explain` shows every layout.

## 3.7 Open questions

1. Reviewer wants parentheses dropped as an `x?` boundary (parens would silently change meaning). You
   accepted parens; kept for now.
2. ~~Two scan defaults?~~ One principle: cuts iff the monoid has an identity in `T` [r10, r11].
3. Reviewer: drop `with_prev`, keep `pairs`. Kept, because you endorsed `prev ?? fallback`.
