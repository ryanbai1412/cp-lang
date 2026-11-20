# 1.3 Domains, sequences and views

Status: decided unless a section says *proposed*. Folded from `sequences.md`
(now in `archive/`); this copy is the source of truth.

**Changed on 2026-11-20 by `19-index-and-lends.md`**, which wins where the
two disagree. What a user writes here is almost unchanged; what changed is
that domains, slices, grids and strings are now library code over one
mechanism. The visible differences: boundaries are written `v.l` and `v.r`
(`v.before` and `v.after` are the long names), a slice is indexed by the
positions of its array and `Sub<V>` no longer exists, and slices and rows
are lent values that can't be stored or returned.


How positions, ranges, slices, generators and sequence views fit together.
The aim is G1: indexing the wrong array, or being off by one, is a compile
error, or can't be written at all. And all of it costs the same as the
hand-written C++ loop.

Status: domains, typed arrays, `.next` / `.prev`, `x in V` refinement,
`V.bounds`, tables, reductions, `.rev` and the `?V` niche are **built** in
the prototype. Generators, slices, `Range1`, sequence views and `sliding`
are **proposed**; generators are the prerequisite for most of the rest.

---------------------------------------------------------------------------

## 1. Positions are typed

### 1.1 Domains

A domain is a fresh index type. Two domains with the same numbers are
still different types:

```
let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain V = 1..=n
domain E = 1..=m

let w: [E]Int = read()
let deg: [V]Int = 0

for e in E: deg[u[e]] += 1      // fine: u: [E]V, so u[e] is a V
deg[e]                          // error: deg is indexed by V, e is an E
```

A domain value is stored as its 0-based ordinal in 4 bytes. Domains are
for things you allocate arrays over, so a domain whose declared size can
exceed 2^31 is a compile error; large coordinates stay `Int` until you
`compress` them. Its *label*
(what you wrote, `1..=n` here) is only for reading, printing and the trace:
`print(v)` prints the label, `v.ord` is 0-based, `v.rank` is 1-based.

Indexing `[V]T` with a `V` is never checked at runtime: a `V` can only
exist if it's in range. The checks move to the few places a `V` is
**created** from a number.

### 1.2 Creating positions

There are four ways to get a `V` from something else, and each says what
happens when the number is out of range:

```
let v = V(x)              // checked: crashes with the value and the domain if x is not in V
if x in V:                // flow fact: x is a V inside this branch, no check
    deg[x] += 1
let k = V.at(i)           // from a 0-based ordinal, checked
let k = V.try_at(i)       // ?V, no crash
```

`read()` creates positions directly: `let p: [V]?V = read()` reads labels
and checks each one, so data from input is typed from the moment it
arrives.

### 1.3 Moving between positions

Stepping off the end is the classic off-by-one, so every step that can
leave the domain returns an optional:

```
v.next          // ?V: none at V.last
v.prev          // ?V: none at V.first
v + d, v - d    // ?V: none when the step leaves V
v.offset(d)     // ?V: the same step, named
v.cyc(k)        // V: wraps around, for cyclic problems
u - v           // Int: distance between two positions of the same domain
```

`v + d` was an error in the first version of this section, with
`v.offset(d)` as the optional step. The ports did that step on raw
`.idx` with a hand-written guard instead, so the operator now is the
optional step (decisions.md 2.9 and §11, leaning):

```
if let u = v - k: ...       // skip at the edge
a[v + k ?? V.last]          // clamp
a[(v + 1)!]                 // assert
a[v + 1]                    // error: `v + 1` is a ?V
```

Because `none` is a value, the end case is handled where it happens:

```
for v in V:
    if let u = v.next and a[u] < a[v]: drops += 1
```

### 1.4 Bounds: the gaps between positions

Many algorithms need a position *between* elements: prefix sums,
insertion points, "the answer is after the last element". These get their
own type, `V.bounds`, with `n + 1` values. A position has a bound on each
side:

```
v.l             // V.bounds: the gap just left of v   (long name: v.before)
v.r             // V.bounds: the gap just right of v  (long name: v.after)
v.l(k), v.r(k)  // the far end of the k-wide window that ends / starts at v; v.l == v.l(1)
V.start, V.end  // the first and last gap
```

A prefix-sum array is indexed by bounds, not positions, so the `+ 1`
shifts disappear:

```
var pre: [V.bounds]Int = 0
for v in V: pre[v.r] = pre[v.l] + a[v]

fn sum(l: V, r: V) -> Int = pre[r.r] - pre[l.l]
fn last_k(v: V, k: Int) -> Int = pre[v.r] - pre[v.l(k)]      // the k elements ending at v
```

`pre[v]` is a type error, and so is `a[v.r]`. The C++ bug "did I mean
`pre[r]` or `pre[r + 1]`" has nowhere to live.

A bound has a position on each side, either of which may be missing:
`b.left: ?V` and `b.right: ?V` (`V.start.left` is `none`).

Binary search on a sorted array returns a bound, since the answer can be
"after everything":

```
let b = V.partition_point(v => a[v] < x)  // V.bounds: the gap before the first a[v] >= x
let first_big = b.right                   // ?V: none if every a[v] < x
let cnt_small = b - V.start               // Int: bounds subtract like positions
```

### 1.5 Derived domains

Other index types are built from domains and behave the same way:

```
enum Dir4: N | E | S | W                 // a field-less enum is a domain
domain Cell = (Row, Col)                 // product: [Row, Col]T is [Cell]T
let (X, vals) = compress(xs)             // fresh domain X, vals: [X]Int
let (C, comp) = scc(adj)                 // fresh component domain
let order: [V]Pre = r.tin                // Euler positions, a fresh domain
```

Fresh domains from functions are what make "index into the compressed
array with a compressed value" and "index a segtree over Euler order with
an Euler position" type-checked. Two calls to `compress` give two
incompatible domains.

### 1.6 When positions are plain numbers

Sometimes the index really is a number: a count, a value used as an index,
a list that grows.

```
let cnt: [0..=1e6]Int = 0      // count-indexed: indexed by Int, checked
cnt[a[v]] += 1                 // checked, unless a: [V]0..=1e6 proves it

var xs: List<Int> = []         // growable: indexed by Int, checked
xs.push(3)
let (D, arr) = xs.freeze()     // [D]Int with a fresh domain D, when it stops growing
```

The value-range checker (plan/30-compiler.md) removes the check when the
declared ranges prove the index fits, which covers most count arrays.

### 1.7 Two dimensions

```
let g: [Row, Col]Char = read()
g[r, c]                       // the only indexing form; g[r][c] is an error
g.row(r)                      // [Col]Char view, no copy
for (r, c) in Cell: ...
for d in Dir4:
    if let q = p.step(d) and g[q] != '#': ...     // step returns ?Cell
```

Going off the grid is a `none` from `step`, not an out-of-bounds read.

---------------------------------------------------------------------------

## 2. Ranges

### 2.1 `Range<D>` and `Range1<D>`

A range is a contiguous run of positions in one domain. It's stored as two
bounds (8 bytes), so it can be empty:

```
let s: Range<V> = l..=r       // inclusive end
let t: Range<V> = l..r        // exclusive end
let all = V.all

s.len                         // Int, 0 if empty
s.is_empty
s.first, s.last               // ?V
s.lo, s.hi                    // V.bounds, always defined (long names: s.before, s.after)
s.contains(v)
s & t                         // intersection, may be empty
```

`Range1<D>` is a range that's known to be non-empty, so `first` and `last`
give a `V` directly. Views that only ever produce non-empty blocks
(`runs`, `windows`, `chunks`) yield `Range1`:

```
for (s, x) in runs(a):        // s: Range1<V>
    longest max= s.len
    print("{s.first} {s.last}")      // no ?: the block is non-empty
```

A `Range1` converts to a `Range` freely; the other direction is
`s.nonempty() -> ?Range1<V>`. `windows(V, k)` and `chunks(V, k)` take
`k: 1..`, so a zero-length window is a type error rather than an empty
`Range1`.

### 2.2 Why ranges instead of `(l, r)`

A pair of positions doesn't say whether `r` is inclusive, and `r - l` vs
`r - l + 1` is a choice you make every time. A range makes the choice
once: `s.len` is the length, `s.hi` is the bound after the last
element.

A computed `l..=r` with `l > r` is the empty range, as in a C++ loop, and
only a literal one is an error. Making it a runtime error would catch
swapped endpoints, but would also reject the many correct loops where
emptiness is the edge case (`for v in l..=r.prev`). Where `l <= r` is a
precondition, say so with `where l <= r` on the input or a `Range1`
parameter type.

Ranges are also what query structures take:

```
pre.sum(s)                    // prefix sums over a range
seg.query(s)                  // segment tree
sparse.min(s)
```

### 2.3 Steps and reversal

```
for i in 0..n by 2: ...
for v in V.rev: ...
for v in (l..=r).rev: ...
```

---------------------------------------------------------------------------

## 3. Slices

A slice is a view of part of an array:

```
let part = a[s]               // s: Range<V>; part is a lent slice, indexed by V
part.len
part.min()
for v in part.positions: ...  // positions are plain V values
deg[v]                        // fine: a slice position indexes every [V] array
a[s].sort()                   // in-place sort of just that range (a is lent mutably for this statement)
```

- **No copy.** A slice is lent by `a` (19 §3.4): `a` plus the range.
  While `part` is in use `a` can't be changed, and a slice that is written
  through (`a[s].sort()`, `a[s].fill(0)`) holds `a` exclusively.
- **A slice can't be kept.** It lives only in the block that made it: it
  can be passed, bound with `let` and iterated, not stored in a `var` or a
  container, or returned (19 §3.5). Keep the range, which is a plain
  value, and write `a[s]` again where it is needed.
- **Positions keep their meaning.** A slice is indexed by `V` itself.
  Indexing it with a position that might be outside the window is checked
  against the range; the loops and reductions over a slice never need
  that check (19 §4.2). So `argmax(a[s])` returns a
  position you can use in every other `[V]` array, not an offset into the
  slice. This is the most common slice bug in C++ (`max_element(a + l,
  a + r) - a` vs `- (a + l)`), and it can't happen.
- To keep a part as its own array, copy it explicitly: `a[s].copy()`.
- A slice of a slice, `a[s][t]`, takes `t: Range<V>` (the original
  positions, not offsets), checked to lie inside `s`.

---------------------------------------------------------------------------

## 4. Mapping and tables

There are three ways to make "one value per position", which differ in
whether anything is stored:

```
let deg2 = [V] v => deg[v] * 2         // table: a new [V]Int, computed now
let deg2: [V]Int = [deg[v] * 2 for v in V]   // same thing, comprehension form
let lazy = deg.map(d => d * 2)         // view: nothing stored; lazy[v] computes on access
```

- **Tables** (`[V] v => ...`) allocate one `[V]T` and fill it once. The
  comprehension form is allowed when the declared type says it's `[V]T`.
- **Views** (`a.map(f)`) store nothing. Indexing a view calls `f` on
  `a[v]` **every time**, so use a view for cheap `f` read a few times,
  and a table otherwise. A view borrows `a`: it can be a local `let` or
  an argument, but it can't be stored in a struct or outlive `a`.
- **List comprehensions** (`[x * 2 for x in xs if x > 0]`) build a
  `List<T>`. They're indexed by `Int` because the filter changes the
  length.

A table over the reverse domain, `[V.rev] v => ...`, is still a `[V]T`:
the domain says which positions exist, `rev` only says the order they're
computed in.

A table may read **its own cells that are already filled**, which makes
one-dimensional DP a table:

```
let suf: [V]Int = [V.rev] v => a[v] + (v.next.map(u => suf[u]) ?? 0)
let dp: [V]Int = [V] v => max(a[v], (v.prev.map(u => dp[u]) ?? 0) + a[v])
```

The rule is deliberately narrow: inside its own definition, a table may
only be read at positions the checker can prove come earlier in the fill
order. In practice that's `v.prev` (or `v.offset(-k)`) when filling
forward and `v.next` when filling in reverse, plus positions known to be
`< v` from a comparison. Anything else is a compile error that suggests
`var dp: [V]Int` and a loop. There's no runtime "is this cell filled" bit,
so it costs nothing, and a read of an unfilled cell can't happen.

---------------------------------------------------------------------------

## 5. Generators: `Iter<T>`

### 5.1 What they are

A generator is a function that produces values with `yield`. It's how
every custom iteration is written, in the stdlib and in user code:

```
pub fn submasks(m: U64) yields U64:
    var s = m
    while true:
        yield s
        if s == 0: break
        s = (s - 1) & m

for s in submasks(full): dp[full] min= dp[s] + cost[full ^ s]
```

`Iter<T>` is the type of a generator that yields `T`. A parameter typed
`Iter<T>` accepts a generator, a range, a domain, an array (yielding its
elements), or a list. The receiving function is specialized for the
generator it's given (M3), which is how the generator's code ends up
inlined into the function's loop:

```
fn bfs<V, A>(adj: V -> Iter<A>, from: Iter<V>) -> [V]?Int
bfs(adj, from: [src])              // a literal list
bfs(adj, from: V.filter(v => shop[v]))
```

### 5.2 Rules

- **A generator runs inside the loop that consumes it.** The compiler
  pastes the consumer's loop body in at each `yield`. There's no iterator
  object, no allocation and no indirect call; it compiles to the
  hand-written loop.
- **Generator functions are values; running generators are not.** A
  lambda that yields (`let adj = (u: V) yields: ...`) and a function
  returning `Iter<A>` (`g.adj(u)`) are fine to pass around, the same as
  any function. What can't exist is a generator *in progress*: it can't
  be stored, returned from two different branches, or resumed later.
  `let it = runs(a)` is an error:
  ```
  error: generator runs(a) can't be stored in `let it`
  help: iterate it directly (for ... in runs(a)) or collect: List(runs(a))
  ```
- **Proposed revision (research/RECOMMENDATIONS.md, decision 4.4):** the
  compiler picks push or pull per use. A plain `for` uses push (the body is
  pasted in at each `yield`). `zip`, or storing a generator, uses pull: the
  generator becomes a small state machine, and `let it: Iterator<T> =
  runs(a)` is allowed for **non-recursive** generators, with its size
  visible in hover. A recursive generator in a pull position is an error
  that suggests collecting it or using an explicit traversal iterator. There
  are never hidden heap stacks. Benchmarks: push and pull are both within
  noise of hand-written loops (research/NOTES.md).
- **An `Iter` parameter is iterated at most once.** Iterating it twice
  would re-run the producer, which silently doubles its cost and repeats
  its effects; the error suggests collecting it into a `List` first.
- **`break`, `continue` and `return` in the consumer act on the consumer,**
  and leave the generator immediately, even through recursion.
- **`yield from g` forwards another generator,** including recursively:
  ```
  pub fn preorder(r: Rooted<V>, v: V) yields V:
      yield v
      for a in r.children(v): yield from preorder(r, a.to)
  ```
  Recursive generators compile to a recursive function that takes the
  loop body as an inlined lambda. Deep recursion is flagged by
  `cp budget`; the stdlib's own traversals (`r.pre`, `r.post`) use an
  explicit stack instead.
- **Borrows:** a yielded value is lent to the consumer until the next
  `yield`. A generator method on `mut self` borrows `self` for the whole
  loop, so the loop body can't also mutate it.

### 5.3 When you do need a stored iterator

Merging two streams, or pausing a traversal and resuming it later, needs
state that outlives one loop. That's an explicit struct implementing the
`Iterator<T>` trait:

```
struct Merge(a: List<Int>, b: List<Int>, i: Int = 0, j: Int = 0)
impl Iterator<Int> for Merge:
    fn next(mut self) -> ?Int = ...

for x in Merge(xs, ys): ...      // `for` accepts any Iterator too
```

The names are deliberately different: `Iter<T>` is a generator (free,
not storable), `Iterator<T>` is a struct (storable, and its size is
visible).

### 5.4 Adapters

Adapters on generators are generators, so a chain is still one loop:

```
V.filter(v => kind[v] == 0).map(v => (v, 0.0))
chain([src], extra_sources)
xs.enumerate()
g.take(k), g.take_while(p), g.skip(k)
g.flat_map(f), g.filter_map(f)
g.scan(m)                        // running fold with an operation value
```

`rev` exists only on things that know their reverse (ranges, domains,
arrays, slices). On a general generator it's an error with "collect first:
`List(g).rev`", since reversing needs storage.

`zip` of a domain or array with a generator is fine (only one side is a
generator). `zip` of two general generators needs both suspended at once;
that's open (§9).

---------------------------------------------------------------------------

## 6. Sequence views

### 6.1 What they are

Sequence views are stdlib generators for the loop shapes where off-by-one
bugs usually live. They yield typed positions and ranges, never raw ints.

### 6.2 Blocks: `runs`

Maximal blocks of equal elements, or of equal key:

```
for (s, x) in runs(a):                  // s: Range1<V>, x: the repeated value
    total += s.len * (s.len + 1) / 2

for (s, pos) in runs(a, by: x => x > 0) // pos: the key shared by the block
    if pos: best max= sum(a[s])
```

`runs` handles the last block (the one every hand-written loop forgets to
flush after the loop) and the empty array (nothing yielded).

Grouping elements that aren't adjacent is a sort first, then `runs`,
written out, since sorting allocates and costs O(n log n):

```
let byc = edges.sorted(by: e => e.c)
for (s, c) in runs(byc, by: e => e.c): ...
```

### 6.3 Neighbours, windows and chunks

```
for (u, v) in pairs(V): ...             // adjacent positions: v is u.next
for s in windows(V, k): ...             // every Range1<V> of length k, left to right
for s in chunks(V, k): ...              // V split into Range1<V> of length k, last may be shorter
for s in split(a, at: x => x == 0): ... // Range<V> between separators; may be empty
```

`windows(V, k)` with `k > n` yields nothing; `pairs` of one element
yields nothing. Each view defines its edge cases once.

Positional views take a domain or a range (`pairs(s)`, `windows(s, k)`
inside a range `s`) and yield positions; you read values with `a[u]`. Only
`runs` and `split` take an array, because they look at its values.

### 6.4 Prefix folds

```
let pre = prefix(a)                     // Prefix<V, Int>, indexed by V.bounds
pre.sum(s)                              // any Range<V>
let pre = prefix(a, Xor<Int>)           // any Group
for (v, acc) in scan(a, Max<Int>): ...  // running fold, generator
```

`prefix` takes a `Group` (it needs subtraction); `scan` takes any
`Monoid`.

### 6.5 Two pointers: `sliding`

For "for each right end, the leftmost left end where the window is still
valid":

```
let best = max(w.len for (w, sum) in sliding(V, init: 0,
    add: (sum, i) => sum + a[i],
    remove: (sum, i) => sum - a[i],
    ok: sum => sum <= k))
```

`sliding` yields one `(window, state)` per right end. It owns the pointer
movement, the order of add and remove, and the empty-window case (if
`a[r]` alone is invalid, the window is the empty range at `r.after`); you
write only the state change and the condition.

The state is an explicit value threaded through `add` and `remove`, not a
captured `var`. Two closures that both change the same captured `var`
(`add: i => sum += ...`, `remove: i => sum -= ...`) would be two mutable
borrows of `sum` at once, which the ownership rules forbid. The explicit
form also keeps the state visible, so it shows up in the trace and replays as a part. For big state (a
count array), the state is moved through each call, which compiles to
updating it in place.

`sliding` requires the condition to be monotone (if a window is valid,
so is every window inside it); debug runs check that at the windows it
visits.

### 6.6 The rest

`merge(a, b)` (sorted merge), `perms`, `subsets`, `compositions`,
`divisors`, `factorize`, `iterate(x, f)` (`x, f(x), f(f(x)), ...`) and the
graph and tree traversals (`bfs_order`, `r.pre`, `r.post`,
`r.children(v)`) are all generators under the same rules.

---------------------------------------------------------------------------

## 7. Reductions and search

### 7.1 Reductions

Reductions read like the math, with `for` and `if` clauses:

```
sum(a[v] for v in V if ok[v])
count(v for v in V if deg[v] == 1)
max(dist[v] for v in V)                  // Int: V is non-empty
min(a[v] for v in s)                     // ?Int: s might be empty
argmax(a[v] for v in V)                  // V: the position, not the value
all(x > 0 for x in a), any(...), find(...), first(...), last(...)
reduce(g, Gcd<Int>)                      // any Monoid
```

- `min`, `max`, `argmin` and `argmax` return `T` when the source is
  provably non-empty (a `Range1`, or a domain whose size has a lower
  bound of at least 1, like `1..=n` with `n: 1..`), and `?T` otherwise.
  Any `if` filter makes it `?T`. "Max of
  an empty range" is a `none` you have to handle, not `-1e18`.
- With a `for` clause, `argmax` returns the bound variable (`v`); on an
  array or slice directly, it returns the position. Either way it's
  typed, so it indexes the other `[V]` arrays. Ties go to the first.
- Reductions are lazy and left to right; nothing is collected.

### 7.2 Binary search

```
first_true(lo..=hi, x => feasible(x))    // ?Int: smallest x where feasible
last_true(lo..=hi, x => feasible(x))     // ?Int
V.partition_point(v => a[v] < x)         // V.bounds, for sorted arrays
xs.lower_bound(x)                        // Int
```

The predicate must be monotone; debug runs check it at the probed points
and report the first pair that breaks it.

---------------------------------------------------------------------------

## 8. Checking and cost

- **Index checks.** Indexing `[D]T` with a `D` has no check. Checks happen
  where positions are created (`V(x)`, `V.at`, `cnt[x]`, `xs[i]`), and are
  removed when flow facts or value ranges prove them. `cp build --checks`
  lists the ones left inside loops.
- **Sizes.** A position is 4 bytes, `?V` is 4 bytes (none is a spare
  value), a range is 8, a slice is a pointer plus a range.
- **Generators and views** compile to the loop you'd write: no objects,
  no allocation, no indirect calls. That's a design guarantee, to be
  verified against hand-written C++ once generators are built.
- **What does allocate,** and says so in the code: tables (`[V] v => ...`),
  list comprehensions, `List(g)`, `.copy()`, `sorted`, `freeze`.
- **The trace** prints positions with their domain (`v = 3:V`), ranges as
  `V[2..=5]`, and slices with their range, so the renderer never shows a
  bare ordinal.

---------------------------------------------------------------------------

## 8b. Escape hatches and checks

Decided (2.11): domain checking has explicit, greppable escape hatches, never
implicit ones. Proposed spellings:

```
a.raw[i]          // index a [V]T by a plain 0-based Int; checked in debug runs
V.at(i)           // Int ordinal -> V, checked
V.try_at(i)       // ?V
```

Check policy (proposed, decisions 7.1/7.2):
- Indexing with a domain position never needs a check (it can't be out of
  range by construction).
- `List` and `raw` indexing are checked unless range analysis proves them or
  the check can be hoisted out of the loop.
- `unchecked:` blocks drop the remaining index checks in the judge build for
  one hot loop; debug runs still check. Benchmarks show such checks cost
  30-50% in knapsack/matmul kernels (research/NOTES.md).

## 9. Open questions

1. **`zip` of two generators.** Proposed: allowed for non-recursive
   generators via pull lowering (§5.2 and research/RECOMMENDATIONS.md).
2. **Views as array arguments.** Should a fn taking a read-only `[V]T`
   also accept a view like `deg.map(f)`? That needs `[V]T` params to be
   generic over "anything indexable by V", which is free under
   specialization but makes signatures less concrete.
3. **Naming.** `runs` vs `blocks`.
4. **2-D windows.** `windows(Cell, (k, k))` for sub-grids.
5. **Views as `mut` targets.** `a[s].sort()` works on a slice; should
   `a.map(f)` views ever be writable? (I'd say no.)
