# Round 3: search and DP

Built on rounds 1 and 2: ranges have cut ends, binary search is a range
method, folds over `.keys` give positions, `scan` is indexed by cuts.

## The model in one paragraph

A DP is a table over the domains its state lives in, filled in an order the
code states, so the table type says what a state is: `[Pos.cuts]Int` for a
prefix DP, `[V.ranges]Int` for an interval DP, `[B.subsets]Int` for a mask
DP, `[V, K]?Int` for "best at v using k". Updates are `min=` / `max=`, which
return whether they improved, so reconstruction is one more line. Top-down
is `@memo(array) fn` (a dense table over the parameter types) or
`@memo(map) fn` (a hash map over the states visited); you pick, since
neither is better on every axis. The named speed-ups (D&C, Knuth, SMAWK,
convex hull, aliens, slope trick) are library functions that take the same
cost function the plain DP uses, so switching is a local edit.

## Search

Binary search lives on ranges (round 1). This round adds the rest.

| Form | Meaning | Cost |
|---|---|---|
| `r.first_true(ok)`, `r.last_true(ok)`, `r.bsearch(ok, last:)` | round 1 | O(log n) calls |
| `r.first_true(ok, near: c)` | galloping search from cut `c` outward: cost grows with the distance to the answer | O(log d) calls |
| `r.peak(by: f)`, `r.valley(by: f)` | an argmax / argmin of `f` unimodal on `r`, `?D`; first on plateaus that don't hide the peak | O(log n) calls |
| `(lo..hi).bsearch(ok, steps: 100)` with `F64` ends | the flip point of a real predicate | `steps` calls |
| `(lo..hi).peak(by: f, steps: 100)`, `.valley(...)` | real ternary / golden-section search, `F64` | `steps` calls |
| `parallel_bsearch(qs, r, apply, ok)` | every query's `first_true` over shared monotone updates `r` | O((q + \|r\|) log \|r\|) |
| `dinkelbach(solve, lo)` | max ratio: `solve(λ)` returns the best `(num, den)` at `λ` | O(log) solves |

`F64` ends make an **interval**: `x in i`, `i.l`, `i.r`, `i.mid`,
`bsearch`, `peak`, `valley`. It has no elements, so no `for`, `len` or
`first_true`. The default `steps: 100` shrinks a 1e18 span below 1e-12.

The answer-search idiom is just the range method, read as the statement:

```python
let t = (0..=1e18).first_true(t => can_finish(by: t))!   # smallest feasible time
let k = (1..=n).last_true(k => fits(k)) ?? 0             # largest k that fits, 0 if none
```

## Tables

```python
domain Pos = 0..n
var dp: [Pos.cuts]Int = inf          # prefix DP: dp[c] is the best for a[..c]
dp[0] = 0
for i in Pos.cuts:
    for j in ..i:                    # every earlier cut; j..i is the piece a[j..i]
        dp[i] min= dp[j] + cost(j..i)
```

- **The table's index type is the state.** `[V, K]?Int` for "best at v
  with k", `[Pos.cuts]` for prefixes, `[V.ranges]` for pieces,
  `[B.subsets]` for masks. A wrong-order index is a type error.
- **Unreached is `none`, not a sentinel.** `?Int` tables start `none`;
  `x min= y` on `none` takes `y`, and `x min= none` does nothing, so
  impossible states never need `inf` guards. `inf` exists for `Int`
  tables when you'd rather; adding to it is an overflow error in debug.
- **`min=` / `max=` return whether they improved**, so the argmin is kept
  where it's found:

```python
if dp[i] min= dp[j] + cost(j..i): from[i] = j
```

- **Layers.** A DP over k layers keeps two tables and moves:
  `repeat k - 1: best = layer(best)`. Assigning a returned table is O(1).
- **Reconstruction** walks the kept argmins: `for c in from.chain(Pos.cuts.last!)`
  yields `c, from[c], from[from[c]], ...` until `none`. `chain` is an
  `[D]?D` method (any parent array: DP choices, BFS back pointers, DSU).

### Interval DP: `V.ranges`

`V.ranges` is the domain of all non-empty ranges of `V`, stored as a
triangle (n(n+1)/2 slots). Iterating it goes by length, then left end, so a
piece comes after all its sub-pieces. `r.splits` yields every way to cut `r`
into two non-empty pieces, as `(left, right)` with `left.r == right.l`: one
pair per cut strictly inside `r`, so `len - 1` of them, none for one element.
O(1) to make, nothing allocated; the cut itself is `left.r`.

```python
var dp: [V.ranges]Int = 0                       # merging stones, matrix chain, ...
for r in V.ranges if r.len >= 2:
    dp[r] = r.splits.map((x, y) => dp[x] + dp[y]).min()! + w(r)
print(dp[V.range])
```

### Mask DP: `B.subsets`

`B.subsets` (written `Subset<B>`) is the domain of subsets of a small
domain `B` (at most 30 members): set algebra `s | t`, `s & t`, `s - t`,
`b in s`, `s.len`, and generators `s.subsets` (submasks, `s` first, empty
last) and `s.supersets`.

```python
var best: [Group.subsets]?Int = none
best[{}] = 0
for s in Group.subsets:
    for g in s.subsets if g != {}: best[s] max= score[g] + best[s - g]
```

`a.fold_subsets(op)` / `a.fold_supersets(op)` over `[B.subsets]T` are the
sum-over-subsets transforms, O(2^n · n); with `op` inverse, `unfold_*`.

### Top-down: `@memo fn`

```python
@memo(array) fn ways(i: Pos.cuts, tight: Bool, sum: 0..=9 * 18) -> Mod:
    if i == Pos.cuts.last!: return Mod(sum % k == 0)
    let hi = if tight: digit[i.r!] else: 9
    (0..=hi).map(d => ways(i.next!, tight and d == hi, sum + d)).sum()
```

- **The storage is spelled, never picked for you.** `@memo(array)` is a
  dense table: every parameter must be a domain, `Bool`, enum or ranged
  Int, else it's a compile error naming the parameter. Memory is the
  product of the sizes (here 19 × 2 × 163 slots), allocated when the
  enclosing block starts; `cpl explain` prints it. `@memo(map)` is a hash
  map over any hashable arguments: O(states visited) memory, a hash per call.
- **Bare `@memo` is an error** whose fix offers `array` when every
  parameter is bounded and `map` otherwise. An array spends memory on
  states the recursion never reaches and a map pays a hash per call, so
  neither majorizes and the choice is the contestant's.
- `@memo(into: dp)` names an existing `var dp: [..]?T` as the table, so the
  same cache can be read after the recursion (reconstruction, answers over
  every state).
- Recursion depth follows the judge profile's stack; `@memo` adds nothing
  per frame beyond the call.
- `ways.clear()` empties the cache (multitest).

## Speed-ups: same cost function, one edit

Each takes the `cost` the plain DP calls, so the plain version stays the
brute in `cpl stress`.

| Form | Plain DP it replaces | Cost |
|---|---|---|
| `monotone_minima(rows, cols, f)` | `row i: min over c of f(i, c)`, argmin monotone in `i` (D&C opt) → `[R]?(val, at)` | O((n + m) log n) calls |
| `smawk(rows, cols, f)` | same, `f` totally monotone (Monge) | O(n + m) calls |
| `knuth(V, w)` | `dp[r] = min over splits + w(r)` with Knuth's condition → `[V.ranges]Int` | O(n²) |
| `aliens(k, solve)` | "exactly k items" when the value is convex in k: `solve(λ)` returns `(val, count)` | O(log range) solves |
| `monge_path(n, cost)` | shortest 0 → n path with Monge `cost(j..i)` (LARSCH) | O(n log n) |
| `LiChao(max, over: r)`, `.add(line, on:)`, `.at(x)` | max of lines / segments at x | O(log \|r\|) |
| `LineContainer(max)` | max of lines, any order | O(log n) |
| `MonotoneHull(max)` | lines and queries both monotone | O(1) amortized |
| `min_plus_convolve(a, b, convex:)` | `c[k] = min a[i] + b[k - i]` | O(n + m) or O(n m) by shape |
| `SlopeTrick` (std.slope) | piecewise-linear convex DP | O(log n) per op |

D&C optimisation as a library call, the same layer cf868f writes by hand:

```python
fn layer(prev: [Cut]?Int) -> [Cut]?Int =
    monotone_minima(Cut.range, Cut.range, (i, j) => if j <= i: prev[j] + cost(j..i)).map(.val)
```

(`if` without `else` is `none` when false: the expression form of a
partial function.)

## Small patterns that recur

| Form | Meaning | Cost |
|---|---|---|
| `a.lis(strict: true)` | positions of one longest increasing subsequence, `List<D>` | O(n log n) |
| `a.prev_smaller()`, `a.next_smaller()` | `[D]?D` nearest smaller to each side (monotone stack); larger with `by: desc` | O(n) |
| `Window<D>(enter, leave)`, `w.move(to: r)` | stateful sliding piece (two pointers, Mo, D&C costs) | O(distance moved) |
| `MonoDeque(min)`, `WindowFold(op)` | sliding min / fold of any monoid | O(1) amortized |
| `backtrack(start, next)` | `yields Step<S>`; `step.prune()`, `break`, `return` | per node |

## Open

1. **`V.ranges` and `r.splits`.** A domain of pieces makes interval DP a
   table like any other. Alternative: `[V, V.cuts]Int` with `l <= r`
   checked by hand (C++'s way, half the table unused).
2. **`if` without `else` as `?T`.** Makes partial cost functions one line.
   Alternative: write `if c: x else: none`.
3. **`peak` / `valley`** for unimodal search, vs `ternary_max` /
   `ternary_min`. Picked for reading as the question, matching `first_true`.
