# 22. Sequences

**Layer:** library. Three traits (`Seq`, `View`, `Stream`) and their adapters, all library. The one core dependency is the `pure` function type (chapter 14.2), which is why lazy-adapter lambdas can't mutate. Status: [proposed].

## 22.1 Three kinds of sequence

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

## 22.2 Folds and scans

- One spelling per idea: `xs.map(f).sum()`, never `sum(by:)` [r8 9]. `max_by(key)`/`min_by(key)` return the
  element, `max()`/`min()` the value.
- Folds follow the scan principle [p2 S1]: `sum`, `count(where:)`, `all`, `any` have an identity and are total (a bare
  `xs.count()` names `.len`) [r14 T7];
  `max`, `min`, `reduce`, `first`, `last` return `?T` over a possibly-empty sequence and `T` over a nonempty one
  (chapter 21.6). List literals are nonempty; `map` keeps nonemptiness, and `flat_map` keeps it only when `f`
  returns `NonEmpty<…>` [r14 T2].
- **Ties** [p2 S4]: `max_by`/`min_by` return the first extremum; `max_by(key, last: true)` the last. `best max= (v, who)`
  compares every field, so ties go to the larger `who` [r14 T10].
- **Combinators** [p2 F1]: `xs.flat_map(f)`, `chain(a, b, …)`, `product(a, b)`, `xs.combinations(2)`,
  `xs.split_by(p) -> (yes, no)` [r14 T21], `(lo..=hi).step(k)` (traps on k ≤ 0). `flatten` concatenates a sequence
  of sequences and has one overload that skips `none` in a sequence of `?T`; `?T` itself is not a sequence [r14 T6]. All fuse (22.4), with early exit under `take`/`first`/`any`.
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
  equal keys as nonempty slices, which convert to ranges (21.5) [r13 P14].
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

## 22.3 Indicators

`Bool` doesn't convert to numbers implicitly. Write `Int(b)` or `Z(b)` [r8 8]:
`print(upto(r) - upto(l) + Z(l.digits().sum() % d == 0))`.

## 22.4 Lowering

| construct | C++ |
|---|---|
| view chain `xs.map(f).filter(p).sum()` | one fused loop, lambdas inlined, no `std::function` |
| array `map` | one allocation, or reuse of a temporary's buffer |
| scan | one pass into a buffer; a loop-local scan with a loop-invariant bound reuses one buffer per loop (Tourism r9: memory 0.93x accepted) |
| `?T` with niche | same width as `T` |
| `x?` | a branch to the extent's `none` result |
| `max=` with `?T` | compare-and-set with a `none` test, branch-free when possible |

## 22.5 Round-3 rules [p3]

- **Lazy adapters take pure lambdas.** A lambda passed to a lazy adapter (`view().map`, `filter`, `where`,
  `take_while`, any stream or range adapter) may not write captured `var`s or call mutating methods. A
  violation is an error that names `.list()` or a `for` loop. Eager array operations are unaffected. A lint
  fires when a `filter` view is consumed twice ("bind `.list()`").
- **Domain `map` is eager, and fusion is unobservable.** `V.map(f)` on a domain, cut domain or `Bits` builds
  `[V]T`. On an integer range it is a view (22.1). Because the lambda passed to a fold is pure (or the map is
  consumed once), the compiler fuses `V.map(f).sum()` into one loop with no array. An eager chain is fused only
  when at most one lambda in it is impure; otherwise it materializes and `cp explain` says so.
- **Semigroup folds.** `Max<T>`/`Min<T>` are semigroups on `T`; their monoid form is `?Max<T>` (identity
  `none`), which is what `max=`, optional folds and `init: none` use. `SegTree.fold(r)` returns `M` (not `?M`)
  when `r` is `NonEmpty` or a whole domain and `M: Semigroup`. The tree is stored over `?M` (niche), paying one
  `none` test per combine; a maybe-empty range still gives `?M`. `Dsu<V, S>` takes `S: CommutativeSemigroup`, so
  a component's min, max or hull needs no `none` identity or `!`.
- **`max_count()` / `min_count() -> ?(T, Int)`** [proposed]: the fold of the `MaxCount<T>`/`MinCount<T>`
  monoid (identity `none`; equal maxima add their counts). It is the spelling for "the best value and how many
  times it occurs", replacing `max` followed by `count(where: x => x == best)`. Same monoid in segment trees.
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
