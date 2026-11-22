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
Every range structure takes operation values and offers only the methods
its operations support. It picks a faster strategy when a law holds,
but only one that is never worse (the algorithm and its answers stay the
same). `cpl explain` shows which strategy it picked. **You name
the structure; the laws pick its strategy.**

```python
var st = SegTree(max, a)                    # point set, range max
var lz = SegTree(+, a, act: add)            # range add, range sum (lazy)
var mx = SegTree(max, a, act: add)          # range add, range max: no push-down
var du = SegTree(act: assign, a)            # range assign, point read (dual)
let sp = SparseTable(min, a)                # O(1) fold: overlapping halves
let fw = Fenwick(+, a)                      # prefix and range sums
let ps = Prefix(+, a)                       # static: O(1) range sums
```

## 1. Operations

### The traits

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

pub trait Commutative<T>: Op<T>        # op(a, b) == op(b, a)
pub trait Idempotent<T>: Op<T>         # op(a, a) == a
pub trait Selective<T>: Idempotent<T>  # op(a, b) is a or b
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
| `min`, `max` | `Monoid`, `Commutative`, `Selective` | far end of `T`; `none` on `?T` |
| `gcd` | `Monoid`, `Commutative`, `Idempotent` | `0` |
| `and`, `or` | `Monoid`, `Commutative`, `Selective` | `true`, `false` |

A bounded `Int` (`0..=1e9`) has a far end, so `min` and `max` are monoids
on it with no sentinel. An unbounded `Int` has no far end, so `min` on it is
only an `Op`, and a structure that needs an identity says
`min needs an identity on Int: use ?Int or a bounded type`.

### Your own operation

Inline, for the common case with no extra laws:

```python
let lo = Monoid(id: Low.empty, op: (a, b) => a.join(b))
let cat = Op((a: Str, b: Str) => a + b)     # no identity: a semigroup
```

With laws, as a named type. The `impl` with no body states a law. It is a
promise: `cpl stress` samples it and `laws_hold` tests it (the `law` tier
in 0-ownership.md).

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
`SegTree(+, a)` over `[D]MaxSub` then uses `MaxSub.zero` and `+`. Nothing
special is needed, because `+` is generic over `T: Add`.

## 2. Actions

An action is a monoid of tags (`F`) plus `apply`, which says what a tag does
to a folded value of `n` leaves. Tags compose **in time order**,
`tags.op(old, new)`, the same left-to-right rule as values, so there is no
separate `compose(new, old)` convention to get backwards.

```python
pub trait Action<M, F>:              # M is the fold op's type, e.g. type of `+`
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

## 2a. Orders

An order is the other operation value every contest program passes around:
`sort`, `min`/`max`/`argmax`, heaps, ordered sets, `bsearch` over sorted
data and Dijkstra all take one. One trait and one argument cover them all.

```python
pub trait Ord:
    fn cmp(self, b: Self) -> Ordering
    law total(a, b, c) = ...       # antisymmetric, transitive, total
```

- Ints, floats (NaN traps in debug), `Str`, `Bool` and `?T` (`none` first)
  are `Ord`. Tuples and records derive it lexicographically in field order.
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
let lightest = min(by: .w)                 # an Op on edges: keeps the lighter
var seg = SegTree(lightest, es)            # range "lightest edge"
let best = es.fold(max(by: e => (e.w, desc(e.id))))
```

The identity is `none` on `?T` or the far end of a bounded key, the same as
plain `min`. Ties keep the left operand, which matches "first of equal
keys" above, so a segment tree of `min(by:)` returns the leftmost minimum
with no index trick (this replaces `MinIdx`/`MaxIdx`).

### Heaps

`Heap<T>(by:)` pops the smallest key; `Heap(by: desc)` pops the largest.
This replaces the `MinHeap`/`MaxHeap` pairs (and the erasable, indexed and
meldable pairs) with one type each: `Heap`, `ErasableHeap`, `IndexedHeap`,
`MeldHeap`. `MinMaxHeap` stays, since it pops both ends.

### Dijkstra: a monoid and an order

A shortest-path search needs two things from weights: how a path's cost
grows by one arc, and which of two costs is better. That is a `Monoid` and
an `Ord`, the same two concepts:

```python
pub fn dijkstra<V, W: Ord, G: WeightedNext<V, W>, M: Monoid<W> = Plus>(
    g: G, from: Seq<(V, W)>, step: M = +, until: ?V = none,
) -> Paths<V, W, G.Arc>
```

```python
dijkstra(g, from: [(s, 0)])                          # sum of weights
dijkstra(g, from: [(s, 0)], step: max)               # bottleneck path
dijkstra(g.weighted(a => (a.w, 1)), from: [(s, (0, 0))])   # cost, then hops
```

- Correctness needs one law: extending a path never makes it better,
  `step(d, w) >= d`. That replaces "traps on a negative weight": debug
  builds check it on each relaxation and name the arc that broke it.
- Tuples add field-wise and compare lexicographically, so `(cost, hops)`
  needs nothing new.
- `bfs01`, `dial` and `bellman_ford` take the same `step:`. Floyd–Warshall
  takes `step:` too; it needs `Commutative` only when the graph is
  undirected.

## 3. One vocabulary

Every range structure uses these names, and has a method only when its
operations support it:

| Form | Meaning | Needs |
|---|---|---|
| `S(op, a)` | build from `[D]T`; `D` comes from `a` | |
| `S(op, fill: x)`, `S(op, [D] i => ...)` | build filled, or from a fn | |
| `s[i]` | the value at `i` | |
| `s[i] = x`, `s[i] += y` | point write, through a `lends mut` place | point updates |
| `s.fold(r)` | fold of `Range<D>` `r`, left to right | `Monoid` (`Op`: traps on empty `r`) |
| `s.all` | fold of everything, O(1) | `Monoid` |
| `s.prefix(c)` | fold of everything before cut `c` | |
| `s.apply(r, f)`, `s.apply_all(f)` | apply tag `f` to every position in `r` | `act:` |
| `s.right_while(from: c, ok)` | the last cut `d` with `ok(s.fold(c..d))`, `ok` monotone | `Monoid` |
| `s.left_while(to: c, ok)` | the first cut `d` with `ok(s.fold(d..c))` | `Monoid` |
| `s.clear()` | back to the build state, O(n) | |
| `for (i, x) in s.items` | every position and value, O(n) | |

- `right_while` and `left_while` return cuts, like `bsearch` in round 1.
  They replace `max_right` and `min_left`.
- `fold`, `apply` and `[]` take `Range<D>` and `D`, so `t.sub(v)` on a
  `[t.Pre]` tree and `..=lo.l` on a `[Rank.cuts]` tree just work.
- `t.fold_path(a, b, s)` and `t.apply_path(a, b, s, f)` from round 4 work on
  any structure that has `fold` and `apply`.

## 4. The structures

| Job | Structure | Build | Query | Strategy picked by laws |
|---|---|---|---|---|
| point write, range fold | `SegTree(op, a)` | O(n) | O(log n) | |
| range apply, range fold | `SegTree(op, a, act:)` | O(n) | O(log n) | tags `Commutative`: no push-down |
| range apply, point read | `SegTree(act:, a)` | O(n) | O(log n) | tags `Commutative`: no push-down |
| static fold | `SparseTable(op, a)` | O(n log n) | O(1) | `Idempotent`: overlapping halves; else disjoint |
| static prefix / range fold | `Prefix(op, a)` | O(n) | O(1) | `Group`: `fold(r)` by difference; else `prefix` only |
| dynamic prefix / range fold | `Fenwick(op, a)` | O(n) | O(log n) | needs `Commutative`; `Group`: `fold(r)` |
| sliding window fold | `Window(op)` | | O(1) amortized | `Selective`: monotone deque; else two stacks |
| offline range apply | `Diff(op)` | | O(1) per add, O(n) finish | needs `Group` |
| old versions | `PersistentSegTree(op, a, act:)` | O(n) | O(log n) | as `SegTree` |
| huge index range | `SegTree(op, over: lo..hi)` | O(1) | O(log range) | nodes made on first touch |
| chmin/chmax with sums | `BeatsSegTree(a)` | O(n) | O(log² n) amortized | its own structure |

**What merges:**
- `LazySegTree` and `DualSegTree` become `SegTree` with `act:`.
- `DisjointSparseTable` becomes `SparseTable`.
- `GroupFenwick`, `PrefixFenwick` and `XorFenwick` become `Fenwick`.
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

## 5. Specialization

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

```python
pub fn apply(mut self, r: Range<D>, f: F):
    if A.Tags: Commutative<F>:
        self.mark(r, f)          # no push
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
   `Fenwick(op)` without `Commutative` is an error that suggests `SegTree`.

## 6. Examples from the dreams

```python
# cf2096f: range add, range min, test for zero
var cover = SegTree<V, Int>(min, fill: 0, act: add)
cover.apply(s.a..=s.b, 1)
if cover.fold(s.a..=s.b) == 0: ...

# cf2152g: flip a subtree, read the root
var st = SegTree(Side.join, t.reorder(init), act: flip)
st.apply(t.sub(v), true)
print(st.all.now.ends)

# cf2084f: a custom monoid, prefix and suffix folds
var st = SegTree<Val, Pos>(Monoid(id: edge, op: op))
out[v] = if below: st.fold(..=v) else: st.fold(v..)

# window minimum
var w = Window(min)
for i in I:
    w.push(i, a[i])
    w.drop_before(i - k)
    ans[i] = w.fold
```

## 7. What this needs from core

- Trait aliases and associated types (W10a, already accepted).
- Bodiless `impl` as a law promise (exists for field traits, W10a).
- `if T: Trait` compile-time tests in generic bodies (new, W10b).
- A type's far end (`T.max`, `T.min`) for bounded types (round 1).
- Operator values having their own types (`+` is a value of type `Plus`),
  so `impl Action<Plus, Int> for Add` can pair actions with folds.

## Open

1. **`Op` without identity.** Should `fold` on an empty range trap, or
   should such structures return `?T`? This proposal traps (with a lint),
   since nearly every op has an identity once bounded types count.
2. **Range add, range sum on a `Fenwick`.** Two Fenwicks implement it today
   (`RangeFenwick`). It could be `Fenwick(+, act: add)`, which is the same
   pattern as `SegTree`, or it could stay a named variant.
3. **Heap names.** `Heap(by: desc)` reads well, but `MaxHeap` is what
   everyone searches for. A doc alias (search and hover find it) costs
   nothing; a real second name would break "one way".
