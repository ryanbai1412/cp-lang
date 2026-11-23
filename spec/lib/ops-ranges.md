# Round 5: operation values and range structures

Built on rounds 1 to 4: ranges are `Range<D>` over a domain, cuts are
`D.cuts`, trees expose `t.Pre`, and parameters ask for the weakest
capability they use. This doc proposes how an operation is written as a
value, how its laws are stated, and one vocabulary for every structure that
folds or updates ranges.

## The model in one paragraph

An **operation value** is a value whose type implements `Op<T>`, `Monoid<T>`
or `Group<T>`. `+`, `max`, `gcd` and `Monoid(id: e, op: f)` are all
operation values. **Laws** (`Commutative`, `Idempotent`, `Selective`) are
traits with no methods, implemented by the operation's type as a promise. An
**action** is a monoid of tags plus how one tag changes a folded value.
An **order** is a key fn (`by:`), and a **semiring** is two ops. Sequence
folds, fast powers, matrices, convolutions, tree DP and Dijkstra take these
same values. Every range structure takes operation values and offers only the methods
its operations support. It picks a faster strategy when a law holds,
but only one that is never worse (the algorithm and its answers stay the
same). `cpl explain` shows which strategy it picked. **You name
the structure; the laws pick its strategy.**

<!-- given:
  domain V = 0..8
  let a: [V]Int = 0
  let l = V(1)
  let r = V(5) -->
```python
var st = SegTree(max, a)                    // point set, range max
var lz = SegTree(+, a, act: add)            // range add, range sum (lazy)
var mx = SegTree(max, a, act: add)          // range add, range max: no push-down
var du = SegTree(act: assign, a)            // range assign, point read (dual)
let sp = SparseTable(min, a)                // O(1) fold: overlapping halves
let fw = Fenwick(a)                         // prefix and range sums
let ps = Prefix(a)                          // static: O(1) range sums
```

## 1. Operations

### The traits

<!-- spec-only: excerpt of spec/std -->
```python
pub trait Op<T>:
    fn op(self, a: T, b: T) -> T
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))

pub trait Monoid<T> = Op<T> + HasId<T>
pub trait HasId<T>:
    @get fn id(self) -> T
    law ident(a: T) = op(id, a) == a and op(a, id) == a

pub trait Group<T> = Monoid<T> + HasInv<T>
pub trait HasInv<T>:
    fn inv(self, a: T) -> T
    law inverse(a: T) = op(a, inv(a)) == id

pub trait Commutative<T>: Op<T>        // op(a, b) == op(b, a)
pub trait Idempotent<T>: Op<T>         // op(a, a) == a
pub trait Selective<T>: Idempotent<T>  // op(a, b) is a or b
```

`op(a, b)` puts `a` on the left. Every fold runs left to right in domain
order, so a non-commutative operation (matrix product, string concat,
affine composition) just works.

### Built-in operation values

| Value | Implements | Identity |
|---|---|---|
| `+` | `Group`, `Commutative` | `T.zero` |
| `*` | `Monoid`, `Commutative`; `Group` on `Mod<P>` without zero | `T.one` |
| `^` | `Group` (`inv` is itself), `Commutative` | `0` |
| `\|`, `&` | `Monoid`, `Commutative`, `Idempotent` | `0`, all bits |
| `min`, `max` | `Monoid`, `Commutative`, `Selective` | far end of `T`; `None` on `?T` |
| `gcd` | `Monoid`, `Commutative`, `Idempotent` | `0` |
| `and`, `or` | `Monoid`, `Commutative`, `Selective` | `true`, `false` |

A bounded `Int` (`0..=1e9`) has a far end, so `min` and `max` are monoids
on it with no sentinel. An unbounded `Int` has no far end, so `min` on it is
only an `Op`: structures over it still build, and their folds return
`?Int` (section 2).

### Your own operation

Inline, for the common case with no extra laws:

```python
let lo = Monoid(id: Low.empty, op: (a, b) => a.join(b))
let cat = Op((a: Str, b: Str) => a + b)     // no identity: a semigroup
```

With laws, as a named type. The `impl` with no body states a law. It is a
promise: `cpl stress` samples it and `laws_hold` tests it (the `law` tier
in 05-ownership.md).

```python
struct Lcm
impl Monoid<Int> for Lcm:
    @get fn id(self) = 1
    fn op(self, a, b) = lcm(a, b)
impl Commutative<Int> for Lcm
impl Idempotent<Int> for Lcm
```

Or adding a law to an inline value with a wrapper. These are plain library
fns returning wrapper types, so user libraries can write more of them:

```python
let lo = Monoid(id: Low.empty, op: join).commutative()
```

A record type can also be its own operation by implementing `Add`.
`SegTree(+, a)` over `[D]MaxSub` then uses `MaxSub.empty()` as the identity and `+`. Nothing
special is needed, because `+` is generic over `T: Add`.

## 2. Combining operations

Bigger operations are built from smaller ones with plain library fns, so no
summary needs a hand-written struct unless it carries real logic.

| Form | Meaning | Laws kept |
|---|---|---|
| `each(+, max)`, `each(s: +, hi: max)` | one op per component, on tuples or records | those all parts share |
| `op.rev()` | `(a, b) => op(b, a)` | all |
| `op.opt()` | the op on `?T`, `None` as identity: any `Op` becomes a `Monoid` | all |
| `Monoid(id:, op:)`, `Group(id:, op:, inv:)` | from lambdas | none until stated |
| `.commutative()`, `.idempotent()` | state a law on an inline value | adds it |

<!-- given:
  domain V = 0..8
  let a: [V]Int = 0
  let l = V(1)
  let r = V(5) -->
```python
var st = SegTree(each(+, max, min), a.map(x => (x, x, x)))   // sum, max, min at once
let (s, hi, lo) = st.fold(l..r)

var nm = SegTree(each(sum: +, hi: max), a.map(x => (sum: x, hi: x)))
nm.fold(l..r).hi
```

`each` is named, not a bare tuple `(+, max)`, because `(max, +)` already
reads as a semiring in CP writing. A tuple of ops is a compile error that
suggests `each(...)` or `Semiring(...)`, so neither meaning is implied.

**Summaries with real logic stay named library types**, each one an `Add`
(or `Op`) impl plus a constructor from one element: `MaxSub` (best
subarray), `MinCount`/`MaxCount` (extreme and multiplicity), `Affine`
(composition of `a x + b`), `Hash` (polynomial string hash, a `Group`), and
`Mat` under a semiring. `MinIdx`/`MaxIdx` go: `min(by:)` keeps the left
operand on ties, so it already returns the first minimum (section 3).

**Reversal.** A fold along a path goes one way up and the other way down.
That is free for a `Commutative` op. Otherwise the value type implements
`Reversible` (`x.rev()`: the fold of the same elements read backwards), as
`MaxSub` and `Hash` do. `fold_path` requires one of the two; asking without
either is a compile error naming both fixes.

**An `Op` without an identity** still builds every structure; its folds
return `?T`, `None` for an empty range. The `?` is in the return type, so it
is visible, and it is exactly `op.opt()`.
## 3. Orders

An order is the other operation value every contest program passes around:
`sort`, `min`/`max`/`argmax`, heaps, ordered sets, `bsearch` over sorted
data and Dijkstra all take one. One trait and one argument cover them all.

<!-- spec-only: excerpt of spec/std -->
```python
pub trait Ord:
    fn cmp(self, b: Self) -> Ordering
    law total(a, b, c) = ...       // antisymmetric, transitive, total
```

- Ints, floats (NaN traps in debug), `Str` and `Bool` are `Ord`; `?T` is
  not (chapter 2): sort optionals with a key such as `by: x => (x == None, x ?? 0)`. Tuples and records derive it lexicographically in field order.
- **`by:` takes a key fn**, and that is the one way to pass an order:
  `sort(mut es, by: .w)`, `argmax(a, by: x => x.score)`, `Heap(by: .d)`.
  Omitted, it is the element's own `Ord`.
- **Descending is `desc(k)`** on the key (exists in `algebra.cp`), so a
  mixed order is one tuple: `by: e => (e.w, desc(e.name))`. A bare `desc`
  reverses the whole order: `sort(mut a, by: desc)`.
- **No comparator argument.** An order a key can't express (fractions
  compared by cross-multiplying, polar angle) is a small type with
  `impl Ord`, used as the key: `by: p => Angle(p)`. Then the order has a
  name, gets the `total` law sampled by stress, and works in every API.
- Every sort is stable; ties keep input order. `min`, `argmin` and the
  others return the **first** of equal keys, `max` and `argmax` too.

### Orders make `min` and `max` monoids

A total order is exactly what makes `min` and `max` `Selective` (section
1). So `min(by: k)` and `max(by: k)` are operation values too, and one
order serves both sorting and folding:

```python
let lightest = min(by: .w)                 // an Op on edges: keeps the lighter
var seg = SegTree(lightest, es)            // range "lightest edge"
let best = es.fold(max(by: e => (e.w, desc(e.id))))
```

The identity is `None` on `?T` or the far end of a bounded key, the same as
plain `min`. Ties keep the left operand, which matches "first of equal
keys" above, so a segment tree of `min(by:)` returns the leftmost minimum
with no index trick (this replaces `MinIdx`/`MaxIdx`).

### Heaps

`Heap<T>(by:)` pops the smallest key; `Heap(by: desc)` pops the largest.
This replaces the `MinHeap`/`MaxHeap` pairs (and the erasable, indexed and
meldable pairs) with one type each: `Heap`, `ErasableHeap`, `IndexedHeap`,
`MeldHeap`. `MinMaxHeap` stays, since it pops both ends. `MinHeap<T>` and
`MaxHeap<T>` remain as type aliases for `Heap<T>()` and `Heap<T>(by: desc)`,
since they are what everyone searches for; `cpl fmt` leaves them alone.

### Dijkstra: a monoid and an order

A shortest-path search needs two things from weights: how a path's cost
grows by one arc, and which of two costs is better. That is a `Monoid` and
an `Ord`, the same two concepts:

```python
pub fn dijkstra<V, W: Ord, G: WeightedNext<V, W>, M: Monoid<W> = Plus>(
    g: G, from: Seq<(V, W)>, step: M = +, until: ?V = None,
) -> Paths<V, W, G.Arc>
```

```python
dijkstra(g, from: [(s, 0)])                          // sum of weights
dijkstra(g, from: [(s, 0)], step: max)               // bottleneck path
dijkstra(g.weighted(a => (a.w, 1)), from: [(s, (0, 0))])   // cost, then hops
```

- Correctness needs one law: extending a path never makes it better,
  `step(d, w) >= d`. That replaces "traps on a negative weight": debug
  builds check it on each relaxation and name the arc that broke it.
- Tuples add field-wise and compare lexicographically, so `(cost, hops)`
  needs nothing new.
- `bfs01`, `dial` and `bellman_ford` take the same `step:`. Floyd–Warshall
  takes `step:` too; it needs `Commutative` only when the graph is
  undirected.

## 4. Actions

An action is a monoid of tags (`F`) plus `apply`, which says what a tag does
to a folded value of `n` leaves. Tags compose **in time order**,
`tags.op(old, new)`, the same left-to-right rule as values. The
`std.algebra.Action` struct spells the same thing as function composition:
its `compose(new, old)` applies `old` first, so `compose(new, old) ==
tags.op(old, new)`. With add-3 then assign-5, both give assign-5.

```python
pub trait Action<M, F>:              // M is the fold op's type, e.g. type of `+`
    type Tags: Monoid<F>
    @get fn tags(self) -> Self.Tags
    fn apply(self, f: F, x: M.T, n: Int) -> M.T
    law hom(f, a, b) = apply(f, op(a, b), 2) == op(apply(f, a, 1), apply(f, b, 1))
    law comp(f, g, x) = apply(tags.op(f, g), x, 1) == apply(g, apply(f, x, 1), 1)
```

**Stock actions are generic over the fold op**, so one name works on every
fold it makes sense for, and the pairing is checked:

| Action | On `+` | On `min` / `max` | On other ops |
|---|---|---|---|
| `add` | `x + f * n` | `x + f` | error: no `impl` |
| `assign` | `f * n` | `f` | `f` folded `n` times with `op` (O(log n)) |
| `affine` | `a * x + b * n` | error: not monotone | error |
| `flip` | swaps `now` / `flipped` of a `Flip<T>` | same | same |

`SegTree(gcd, a, act: add)` is a compile error, `add does not act on gcd`,
not a wrong answer. Today's combined names (`AddMax`, `AddSum`,
`AssignMin`) become `SegTree(max, act: add)` and so on.

A custom action, written inline:

```python
let shift = Action(tags: +, apply: (d, l: Low, n) => (..l, cover: l.cover + d))
var low = SegTree(Low.join, [Cut] c => Low(0, c, c, 0), act: shift)
```

`n` is computed from the node, never stored, so an action that ignores it
costs nothing extra. A lambda that takes `(f, x)` without `n` is accepted.

## 5. Semirings

Two ops, where `mul` distributes over `add`: what matrix products, path
algebras and convolutions are generic over.

```python
pub trait Semiring<T>:
    type Add: Monoid<T> + Commutative<T>
    type Mul: Monoid<T>
    law distrib(a, b, c) = mul(a, add(b, c)) == add(mul(a, b), mul(a, c))
    law absorb(a) = mul(add.id, a) == add.id

Semiring(+, *)            // counting; the default for numbers
Semiring(min, +)          // shortest paths (tropical)
Semiring(max, +)          // longest paths, best scores
Semiring(or, and)         // reachability
Semiring(max, min)        // bottleneck
```

The pair is written with the ops themselves, so the names
`min_plus`/`max_plus`/`bool_ring` go. Every number type is a
`Semiring(+, *)` by default, so `over:` is only ever written for the others.

<!-- given:
  domain V = 0..3
  let d: [V, V]?Int = None
  let adj: [V, V]Int = 0
  let k = 2 -->
```python
let m = Mat(over: Semiring(min, +), d)          // d: [V, V]?Int
let k_steps = pow(m, k)                         // shortest walk of exactly k arcs
let ways = pow(Mat(adj), k)                     // number of walks: (+, *)
```

- `Mat<T>(over: S)` multiplies under `S`; `m * m2` is `S.mul`-of-`S.add`.
  `det`, `inverse` and `rank` need a field (`Mod<P>`, `F64`), and exist only
  then.
- Floyd–Warshall is the closure of a `(min, +)`-like semiring: `step:` is
  its `mul`, `min(by:)` its `add`.
## 6. Folds on plain sequences

The same values fold any `Seq`, with no structure:

| Form | Meaning | Cost |
|---|---|---|
| `xs.fold(op)` | left to right; `T` for a `Monoid`, `?T` for an `Op` | O(n) |
| `xs.sum()`, `xs.prod()` | `fold(+)`, `fold(*)` | O(n) |
| `xs.min()`, `xs.max()`, `xs.min(by:)` | `fold(min)` and friends; `?T`, `None` when empty | O(n) |
| `xs.scan(op)` | the running folds, lazily: a generator | O(1) per item |
| `pow(x, k, op = *)` | `x` folded with itself `k` times by squaring | O(log k) ops |

`pow` is the one fast power: numbers, `Mod<P>`, matrices under any
semiring, permutations (`op: compose`), `Affine` maps, and any user monoid.
`k = 0` needs a `Monoid`; an `Op` traps on it.

```python
let f = pow(Affine(a, b), k, op: then)     // apply x -> a x + b, k times
let p = pow(perm, 1e18, op: compose)       // a permutation applied 10^18 times
```
## 7. One vocabulary for range structures

Every range structure uses these names, and has a method only when its
operations support it:

| Form | Meaning | Needs |
|---|---|---|
| `S(op, a)` | build from `[D]T`; `D` comes from `a` | |
| `S(op, fill: x)`, `S(op, [D] i => ...)` | build filled, or from a fn | |
| `s[i]` | the value at `i` | |
| `s[i] = x`, `s[i] += y` | point write, through a `lends mut` place | point updates |
| `s.fold(r)` | fold of `Range<D>` `r`, left to right | `T` for a `Monoid`, `?T` for an `Op` |
| `s.all` | fold of everything, O(1) | `Monoid` |
| `s.prefix(c)` | fold of everything before cut `c` | |
| `s.apply(r, f)`, `s.apply_all(f)` | apply tag `f` to every position in `r` | `act:` |
| `s.right_while(from: c, ok)` | the last cut `d` with `ok(s.fold(c..d))`, `ok` monotone | `Monoid` |
| `s.left_while(to: c, ok)` | the first cut `d` with `ok(s.fold(d..c))` | `Monoid` |
| `s.clear()` | every leaf back to `fill` (or the identity), pending tags dropped, O(n) | |
| `for (i, x) in s.items` | every position and value, O(n) | |

- `right_while` and `left_while` return cuts, like `bsearch` in round 1.
  They replace `max_right` and `min_left`.
- `fold`, `apply` and `[]` take `Range<D>` and `D`, so `t.sub(v)` on a
  `[t.Pre]` tree and `..=lo.l` on a `[Rank.cuts]` tree just work.
- `t.fold_path(a, b, s)` and `t.apply_path(a, b, s, f)` from round 4 work on
  any structure that has `fold` and `apply`.

## 8. The structures

| Job | Structure | Build | Query | Strategy picked by laws |
|---|---|---|---|---|
| point write, range fold | `SegTree(op, a)` | O(n) | O(log n) | |
| range apply, range fold | `SegTree(op, a, act:)` | O(n) | O(log n) | tags `Commutative`: no push-down |
| range apply, point read | `SegTree(act:, a)` | O(n) | O(log n) | tags `Commutative`: no push-down |
| static fold | `SparseTable(op, a)` | O(n log n) | O(1) | `Idempotent`: overlapping halves; else `DisjointSparseTable(op, a)` |
| static prefix / range sum | `Prefix(a)` | O(n) | O(1) | `+` only: `prefix(c)`, `fold(r)`; other ops scan with `a.scan(op)` |
| dynamic prefix / range sum | `Fenwick(a)` | O(n) | O(log n) | `+`/`-` only: `fold(r)`; other ops use `SegTree` |
| sliding window fold | `Window(op)` | | O(1) amortized | `Selective`: monotone deque; else two stacks |
| offline range add | `Diff` | | O(1) per `add(r, x)`, O(n) finish | `+`/`-` only |
| old versions | `PersistentSegTree(op, a)`, `PersistentLazySegTree(op, a, act:)` | O(n) | O(log n) | as `SegTree` |
| huge index range | `SegTree(op, over: lo..hi)` | O(1) | O(log range) | nodes made on first touch |
| chmin/chmax with sums | `BeatsSegTree(a)` | O(n) | O(log² n) amortized | its own structure |

**What merges:**
- `LazySegTree` and `DualSegTree` become `SegTree` with `act:`.
- `DisjointSparseTable` stays its own type: `SparseTable` needs `Idempotent`.
- `GroupFenwick`, `PrefixFenwick` and `XorFenwick` become `Fenwick`.
- `RangeFenwick` goes: range add with range sum is `SegTree(+, a, act: add)`,
  iterative and without push-down since `add` tags commute. `Fenwick` takes
  no `act:`.
- `MonoDeque`, `WindowFold` and `sliding_fold` become `Window`.
- `SparseSegTree` and `SparseLazySegTree` become `SegTree(over:)`.
- `AddMax` and the other combined action names become `act: add` and the
  other stock actions.

**What stays named, because it's a different structure:** `Fenwick`
(half the memory of a `SegTree`, and faster), `SparseTable`,
`PersistentSegTree`, `BeatsSegTree`, the 2D forms, `WaveletMatrix` and
`MergeSortTree`.

**Why `act:` and not a separate `LazySegTree`:** the structure is
determined by what you hand it: an op, an action, or both. The cost is
still visible. `act:` in the constructor is where tag memory appears, and
`cpl explain` shows the layout.

`Fenwick2D<R, C, T>` is point add and rectangle sum over a grid
(O(log R log C)); `Line(k, m)` is the `k x + m` that `LiChao` and
`MonotoneHull` store.

## 9. Specialization

Ryan's example is that lazy propagation is unnecessary when tags commute.

**Without push-down.** If the tags' monoid is `Commutative` (`add` on
`max`, `add` on `+`, `xor` flips), the order in which a node's ancestors'
tags apply doesn't matter.
- `apply(r, f)` leaves `f` on the O(log n) covering nodes and updates the
  folds on their ancestors.
- `fold(r)` adds up the tags on each covering node's path from the root as
  it climbs.
- Nothing is ever pushed down. That is about half the memory traffic of a
  propagating tree, with the same O(log n) bound and the same answers.
- A point write `s[i] = x` still pushes along its one path, O(log n).

**How a library writes it.** A generic body can test a bound, and the test
is decided when the fn is instantiated:

<!-- spec-only: excerpt of spec/std -->
```python
pub fn apply(mut self, r: Range<D>, f: F):
    if A.Tags: Commutative<F>:
        self.mark(r, f)          // no push
    else:
        self.push_path(r)
        self.mark(r, f)
```

- `if T: Trait` is a compile-time test on a type.
- Each branch is checked with the bound it assumes.
- The branch not taken is never emitted.

This is a new power for user libraries too (POWERS W10b). It replaces
duplicate structures with one body.

**Rules that keep specialization honest:**
1. **It majorizes.** A law may choose a strategy that is never worse in
   time or memory, and has the same answers on lawful inputs. A law never
   changes the complexity class you were promised, and it never switches
   to a different named structure.
2. **Visible.** `cpl explain` shows "SegTree: tags commute (add on max),
   no push-down".
3. **Laws are promises.** A wrong `impl Commutative` gives wrong answers,
   never undefined behaviour. Stress runs sample every promised law on
   random inputs and say which one failed. `--laws=off` turns the
   specializations off, so you can compare.
4. **Missing laws are errors, not silent wrongness.** `t.fold_path` with a
   non-`Commutative` op whose `T` doesn't implement `Reversible` is a
   compile error. Today it silently assumes "its own reverse".
   `Fenwick` is `+`-only; any other op is an error that suggests `SegTree`.

## 10. Groups: subtract instead of store

An inverse turns "fold of `l..r`" into "fold up to `r` minus fold up to
`l`". Every structure that can use this, does, under the same rule: the
method exists when the op is a `Group`.

| Structure | With a `Group` |
|---|---|
| `Prefix(a)` (`+` only) | `fold(r)` in O(1) |
| `Fenwick(a)` (`+` only) | `fold(r)` in O(log n); `s[i] = x` (read, then add the difference) |
| `Window(op)` | `pop()` subtracts: O(1) worst case, no second stack |
| `Diff` (`+` only) | `add(r, x)` offline in O(1), then `finish()` in O(n) |
| `Potentials(op)` | union-find with relative values (below) |

**Union-find with potentials.** "Value of `b` is value of `a` plus `w`"
constraints, checked online, are a `Group` on a DSU:

```python
var p = Potentials<V>(+)
if not p.relate(a, b, w): print("CONTRADICTION")   // val[b] = val[a] + w
p.diff(a, b)                                       // ?Int: val[b] - val[a], None if unrelated
```

It works for any group: `^` for parity constraints, `Affine` for
"`b = 2a + 3`" problems, permutations for orientation puzzles. The existing
`DsuOf(join:)` stays for per-set summaries; its `join` is an `Op`.

**Shared undo.** `Dsu`, `DsuOf`, `Potentials`, `Window` and the persistent
structures that support stepping back share one pair: `mark() -> Mark` and
`undo_to(m)`. Rollback Mo and offline dynamic connectivity take any
structure with that pair.
## 11. Offline and sqrt structures

- **Mo's algorithm keeps closures, not op values.** Mo exists for answers
  that are not decomposable folds (distinct count, mode); with a monoid you
  would use a segment tree. Its signature stays `mo(queries, add:,
  remove:, answer:)`, with `order: .hilbert` as an option instead of a second
  fn. Rollback Mo is `mo(queries, add:, keep: s)` where `s` has
  `mark`/`undo_to`, so no `remove` is needed.
- **Sqrt blocks** are `Blocks(op, a, act:, size:)` with exactly the
  `SegTree` vocabulary. They exist for operations with no cheap
  composition (e.g. "sort this block", rebuild per block) and for the
  O(1)-update, O(sqrt n)-query trade.
- **Offline divide and conquer over time** (`timetree.cp`) takes any
  structure with `mark`/`undo_to`.
## 12. Trees and DP

Every tree and DP facility takes the same values:

| Facility | Takes | Needs |
|---|---|---|
| `t.fold_path(a, b, s)` | any structure with `fold` over `t.Pre` | `Commutative` or `Reversible` |
| `t.up(op, lift:, finish:)` | children folded with `op` | `Monoid` |
| `t.reroot(op, lift:, finish:)` | same, every root | `Monoid` (no inverse: prefix and suffix folds) |
| `Lift(t, val, op)` | binary lifting of path folds | `Op`; non-commutative needs `Reversible` |
| `LiChao(min, over:)` | lower envelope of lines | a `Selective` op (`min`, `max`, `min(by:)`) |
| `Window(op)` | sliding window fold, monotone queue | `Selective` for the deque strategy |
| `dijkstra(step:)` | path cost growth | `Monoid` + `Ord` on `W` |

`Lift` changes from `(op, identity)` lambdas to one op value. `LiChao`'s
`pick` becomes the same selective op used everywhere, so `min(by: c =>
(c.cost, c.used))` breaks ties with no special support.
## 13. Convolutions

One fn, two choices: how indices combine, and which semiring values live in.

```python
convolve(a, b)                       // c[i + j] += a[i] * b[j]
convolve(a, b, on: ^)                // c[i ^ j]  (Walsh–Hadamard)
convolve(a, b, on: |)                // c[i | j]  (subset-sum transform)
convolve(a, b, on: &)                // c[i & j]
convolve(a, b, on: subset)           // i | j with i & j == 0
convolve(a, b, on: gcd)              // c[gcd(i, j)]  (Dirichlet-style)
convolve(a, b, over: Semiring(max, +), concave: .b)   // max-plus, b concave
```

- The index op is an operation value (`on:`), so the eight
  `convolve_or`/`_and`/`_xor`/... names collapse to one (done in 0.3.0; `std.ntt`).
- The value type picks the algorithm, which is visible in its type: `Mod<P>`
  with an NTT prime uses NTT; any other `Mod` or `Int` uses three primes and
  CRT; `F64` uses FFT. `cpl explain` names it.
- `over:` other than `Semiring(+, *)` has no fast transform in general, so
  it requires a stated shape (`concave: .b` gives the O(n log n) SMAWK
  path; debug checks the shape). Without one it is an error, not a hidden
  O(n²).
- `cyclic: true`, and `convolve_all(polys)` for many at once, stay.
## 14. Cost model

What a contestant can rely on, matching goal 3 (abstractions cost what C++
costs):

- **Operation values are free.** `+`, `max`, `gcd` are zero-size values of
  their own types. A structure is monomorphized on the op's type, so the
  op is inlined into the loop: no function pointers, no virtual calls. A
  lambda passed to `Monoid(id:, op:)` is also a distinct type and inlines the
  same way.
- **Identities are constants** when the op's type knows them (`T.zero`, the
  far end of `T`), and one stored field otherwise. They are never stored
  per node.
- **Spans are computed, not stored.** The node's span `n` comes from its
  level in a power-of-two layout. An action that ignores `n` costs nothing.
- **Tags cost memory only with `act:`**: one `F` per internal node, plus one
  bit per node for "has a tag" only when `F` has no spare value for "no
  tag" (a niche, as for `?T`).
- **Laws cost nothing at run time.** They select code at compile time; only
  stress and debug-mode checks evaluate them.
- `cpl explain` on a structure prints its layout, bytes per element, and
  each strategy chosen and why.
## 15. Testing laws

A law is a promise; the tools make it cheap to check:

- `laws_hold(op, samples)` evaluates every law the value promises on sampled
  inputs and returns the first failure: `"Lcm breaks assoc at (4, 6, 10)"`.
- `cpl stress` samples the laws of every op and action used in the run,
  drawing inputs from the values that actually flow through the structure,
  so a law that fails only on real data is found too.
- `--laws=off` disables every law-driven strategy, so a wrong answer that
  disappears with it is a broken promise, not a broken algorithm.
- Debug builds check the laws that are cheap per operation: Dijkstra's
  `step(d, w) >= d`, `Ord`'s antisymmetry on the compared pair.
## 16. Examples from the dreams

<!-- spec-only: fragments of ported solutions -->
```python
// cf2096f: range add, range min, test for zero
var cover = SegTree<V, Int>(min, fill: 0, act: add)
cover.apply(s.a..=s.b, 1)
if cover.fold(s.a..=s.b) == 0: ...

// cf2152g: flip a subtree, read the root
var st = SegTree(Side.join, t.reorder(init), act: flip)
st.apply(t.sub(v), true)
print(st.all.now.ends)

// cf2084f: a custom monoid, prefix and suffix folds
var st = SegTree<Val, Pos>(Monoid(id: edge, op: op))
out[v] = if below: st.fold(..=v) else: st.fold(v..)

// window minimum
var w = Window(min)
for i in I:
    w.push(a[i])
    if w.len > k: w.pop()
    ans[i] = w.all!
```

## 17. What this needs from core

- Trait aliases and associated types (W10a, already accepted).
- Bodiless `impl` as a law promise (exists for field traits, W10a).
- `if T: Trait` compile-time tests in generic bodies (new, W10b).
- A type's far end (`T.max`, `T.min`) for bounded types (round 1).
- Operator values having their own types (`+` is a value of type `Plus`),
  so `impl Action<Plus, Int> for Add` can pair actions with folds.

## Open

1. **`convolve(on:)` for `subset`.** `subset` is not an operator, so it is
   a named index op the library defines. That is fine as long as user
   libraries can define more (they can: it's a value with a transform impl).

Settled in review: no `act:` on `Fenwick` (range add/sum is a `SegTree`), `MaxHeap`
as an alias, `each(...)` for products of ops.
