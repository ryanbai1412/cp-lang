# Friction backlog, Scout Batch 32: binary search, sweep-line, compression, intervals

Ten unported corpus problems were solved against the locked v2 spec
(`docs/one-page-cheat-sheet.md`) in `proto/ports/scout_batch_32/`. This note
goes through the five v2 features the batch was meant to test, says what each
one bought in real code, and then lists the friction that is left as `X1` to
`X8` (the same labels as the `WISH` comments in the ports).

The short version: sorted lookups (`first_ge`, `last_le`, `rank`) carried
most of the batch, more than `bsearch_first` did, and together with `desc(x)`
none of them needed a second attempt. `compress(all)` was needed once, in a
shape it does not quite fit. `MonoDeque` and `slide_until` could not be
tested: nothing left in the corpus needs them.

## bsearch_first / bsearch_last

Half of the C++ sources contain a binary search, but only one of them is a
search on the answer. The rest are `lower_bound` / `upper_bound` calls on
sorted data, which in v2 are sorted lookups and are covered in the next
section. The answer-search problems in the corpus have mostly been ported
already; four earlier picks for this batch (`mmcc14p3`, `ioi09p2`, `noi15p1`,
`cco22p3`) were landed by other batches while this one was in progress.

The one answer search is the bouncing-balls port (`996warmup8p3`). The C++ is

```cpp
long l = 0, r = long(1e9) * 2 * L / n + 2;
if (n >= 1e5) r = 2e18;
while (l < r) {
    long mid = (l + r) / 2;
    auto [b1, b2, _] = simu(mid);
    if (b1 >= kl || b2 >= kr) r = mid; else l = mid + 1;
}
```

and in CP Lang the loop, the midpoint and the bound juggling are one call:

```cp
let t1 = bsearch_first(0..=4e18, t => any(done(after(t).hits)))!
```

The range argument made it natural to pass one generous bound instead of the
C++'s two special-cased ones. The port needs the simulated state at the
answer as well, and that is where the search API shows a seam; see X7.

## Sorted lookup: first_ge, last_le and rank

This was the most useful feature in the batch, more than `bsearch_first`.
Four different C++ idioms collapse into the same small family of calls.

`lower_bound` on an array, in the LIS of `iati21a22`:

```cp
if let i = tails.first_ge(v): tails[i] = v
else: tails.push(v)
```

The C++ pads the array with `1e9` sentinels and a `-1` at the front so that
`*lower_bound(...) = b` always lands somewhere. The optional result makes the
padding unnecessary.

An iterator-erase walk over a `std::set`, in the interval sweep of `noimps23p1`:

```cpp
for (auto it = gvis.lower_bound(max(start, l)); it != gvis.end() && *it <= r; it = gvis.erase(it))
```

```cp
while let x = unswept.first_ge(max(from, l)) and x <= r:
    unswept.remove(x)
    for j in starting[x]: board(j)
```

`while let ... and cond` is exactly the right shape for "next element, if it
is still in range". No iterator invalidation rule to remember.

Counting values in a half-open interval, in `aac5p3`, via `SortedBag.rank`:

```cp
ans[u] = xs.rank(part.hi) - xs.rank(part.lo)
```

A `std::map` used as a prefix-minimum table, in Hermes (`ioi04p2hard`). The
C++ walks iterators with `upper_bound`, `prev`, `begin()` checks and
`erase`-in-a-loop. With `last_le` and `first_ge` returning optional entries,
the whole structure is five lines:

```cp
struct MinUpTo:
    m: Map<Int, Int> = Map()
    fn offer(mut self, k: Int, v: Int):
        if let (_, best) = m.last_le(k) and best <= v: return
        while let (k2, v2) = m.first_ge(k + 1) and v2 > v: m.remove(k2)
        m[k] = v
    fn upto(self, x: Int) -> ?Int = if let (_, v) = m.last_le(x): v else: none
```

`upto` returns `none` where the C++ returns `1e18`, and the caller's `min=`
skips it.

## compress(all) with xs.D and xs.at(val)

Only one of the ten problems needs compression in CP Lang, and that is itself
a result. The C++ of `iati21a22` compresses both rotated axes so that it can
bucket points by x; the port sorts by a key and never needs dense ids. In
`noimps23p1` and `ioi04p2hard` an ordered `Set` or `Map` keyed by the raw
value does the job. Sorting with `by:` and ordered containers remove most
reasons to compress.

The one real use is Measures (`ceoi22p5`): all positions are known up front,
and each person needs a fixed leaf of a segment tree in sorted order. The
typed domain pays off, since the tree is declared over `slot.D` and sized by
the compression:

```cp
let slot = compress((x, i) for (i, x) in xs.items)
var st = SegTree<slot.D, Span>(Span(), (a, b) => a.then(b))
for (i, x) in xs.items:
    st.set(slot.at((x, i)), Span.of(x))
```

But this is rank compression, not value compression: two people at the same
position need two leaves, and `compress` merges equal values. The port gets
around that by compressing `(x, i)` pairs, which works and is not obvious; see X4.

## Bare operators and desc(x)

`desc(x)` was used twice.

In a sort key, where C++ needs a comparator lambda or a negation that
overflows on `INT_MIN`:

```cp
let vs = sorted((u: p.x + p.y, v: p.x - p.y) for p in pts, by: p => (desc(p.u), desc(p.v)))
```

On a scalar, `sorted(xs, by: p => desc(p))`, which is slightly heavier than
C++'s `greater<>()`. A `sorted(xs).rev` reads better there; not a real problem.

Bare operators showed their value in Salesman (`rpeng3`). The C++ segment
tree is filled with `-1e18` and every query result has an offset added to it,
which is only safe because the sentinel is far from overflow. With an
optional element type and the bare `max`, there is no sentinel:

```cp
var from_low = SegTree<Pos, ?Int>(max)
var from_high = SegTree<Pos, ?Int>(max)
```

`max` skipping `none` is exactly the identity the tree needs. The cost shows
up one line later, when the folded `?Int` has to be shifted; see X1.

The explicit `(id, op)` form was used for the four-field node of `ceoi22p5`
and was unremarkable: `SegTree<slot.D, Span>(Span(), (a, b) => a.then(b))`,
with the identity coming from the struct's field defaults.

One observation from writing the runnable versions: `stdlib/` and the
interpreter have none of `bsearch_*`, `first_ge`, `compress` or ordered-set
lookups yet, so every runnable port in `today/` writes those out by hand. The
gap between the ideal and the runnable file is the best per-problem measure
of what the v2 prelude buys; for `ioi04p2hard` it is 30 lines of list
shuffling in place of the five-line `MinUpTo`.

## MonoDeque and slide_until: not exercised

No port in this batch uses either, and that is a finding about the corpus
rather than about the features. A scan of all unported accepted submissions
for `deque` found four: `noip10p1` (a FIFO cache, where plain `Deque` is the
right tool), `dmopc20c4p4` (which only uses the back, so it is a stack), and
the much larger `ioi17p2` and `noi22p1`, which use a deque as an indexable
list. None is a monotone queue, and no two-pointer window turned up under the
binary-search or sweep tags either. Earlier batches have consumed those problems.

So the locked `MonoDeque` and `slide_until` designs got no new evidence here,
for or against. If they need more pressure, the problems have to come from
outside the corpus (DMOJ statement-first scouts), not from another corpus batch.

## Remaining friction

### X1: arithmetic on an optional

Salesman keeps "best profit so far" in `SegTree<Pos, ?Int>(max)`. Arriving at
position `x` costs a shift of the folded value, and the natural line is

```cp
fn arrive(x: Pos) -> ?Int = max(from_low.fold(..=x) - x * down, from_high.fold(x..) + x * up)
```

Optionals are not operands of `+` and `-`, so the port unwraps each side:

```cp
fn arrive(x: Pos) -> Int:
    var best: ?Int = none
    if let v = from_low.fold(..=x): best max= v - x * down
    if let v = from_high.fold(x..): best max= v + x * up
    best!
```

The same thing happens inside the merge of `ceoi22p5`, where a child's
optional maximum is shifted by the sibling's size and the port needs a
`shift(v: ?Int, by: Int) -> ?Int` helper, and in `ioi04p2hard`, where the
formula `min(below.upto(p) + p, above.upto(-p) - p)` becomes four lines.
`min`, `max`, `min=` and `max=` already skip `none`, and `v + k -> ?V` already
propagates for indices. The missing rule is "`opt + k` and `opt - k` with a
plain operand are `none` when `opt` is `none`". Without it, choosing `?Int`
over a `-1e18` sentinel is safer and longer, which is the wrong trade to
offer. Suggested priority: high; it appeared in three of ten ports, every
time directly next to a `min` or `max`.

### X2: ordered Map has fewer lookups than Set

The cheat sheet gives `Set` and `SortedBag` four lookups (`first_ge`,
`first_gt`, `last_le`, `last_lt`) and the ordered `Map` two (`first_ge`,
`last_le`). The prefix-minimum table of `ioi04p2hard` needs "first key
strictly greater than k" on a map and writes it as

```cp
while let (k2, v2) = m.first_ge(k + 1) and v2 > v: m.remove(k2)
```

`k + 1` is fine for `Int` keys and not available for tuple or `F64` keys.
Giving `Map` the same four names as `Set` costs nothing in concepts.
Suggested priority: medium, and cheap.

### X3: LIS is still written by hand

With `first_ge` the inner step is two lines, which is fine. But the choice
between strict and non-strict (`first_ge` versus `first_gt`) is the classic
off-by-one of this algorithm, and `iati21a22` additionally needs the tie order
in the sort to agree with it. A userland `lis(xs, strict: true) -> Int` would
make that choice a named argument. Suggested priority: low; pure library.

### X4: rank compression with ties

`compress(all)` gives one id per distinct value. Measures needs one id per
element, in sorted order, with equal values next to each other. The port
compresses pairs and looks the pair up again:

```cp
let slot = compress((x, i) for (i, x) in xs.items)
...
st.set(slot.at((x, i)), Span.of(x))
```

It works, and `slot.D` is a proper domain for the tree. The friction is that
nothing on the cheat sheet suggests it, and the first instinct,
`compress(xs)`, compiles and gives wrong answers on duplicates. The C++
does the same thing with a sort of `(value, index)` pairs and an inverse
array, six lines. A direct form would be a sorting permutation with a typed
inverse, for example `let rank = ranks(xs)` returning `[I]R` with `R` a fresh
domain of the same size. `Perm<V>` with `.inv()` is close to this already.
Suggested priority: medium; "sort by A, index a tree by rank" is the standard
offline-sweep setup.

### X5: intersecting ranges

`aac5p3` passes a half-open value interval down the tree and clamps each
child's share to the parent's:

```cp
route(kid.v, u, max(cuts[i], part.lo)..min(cuts[i + 1], part.hi))
```

`Range` has `.is_empty` and `.len`, so an intersection operator fits the type:
`(cuts[i]..cuts[i + 1]) & part`. `&` already means intersection on `Set` and
`Bitset`. Suggested priority: low to medium; interval clamping shows up in
most sweep and divide-and-conquer code.

### X6: mirroring a 1D problem

`noimps23p1` solves "going right" and gets "going left" by running the same
function on reflected coordinates. The reflection `n + 1 - x` is applied to the
start, to both ends of every interval (swapped), and to every answer:

```cp
fn flip(x: Int) -> Int = n + 1 - x
let left = ends(flip(start), [(l: flip(iv.r), r: flip(iv.l)) for iv in ivs])
print(" ".join("{x}" for x in right | Set(flip(x) for x in left)))
```

With positions as plain `Int` this is acceptable. With a nominal
`domain Pos = 1..=n`, which is what the spec encourages, there is no way to
write `flip` without `.to_idx` arithmetic and an unwrap. A `p.mirror -> Pos`
on indices (and `r.mirror` on ranges, which also swaps the ends) would let
this port keep its typed domain. For that reason the ideal port falls back to
`Int` positions. Suggested priority: medium; "solve one direction, mirror for
the other" is common, and today it pushes people off domains.

### X7: binary search whose predicate computes the answer state

In `996warmup8p3` the predicate simulates all balls up to time `t`, and the
code after the search needs that simulated state at the answer. The options are:

```cp
let t1 = bsearch_first(0..=4e18, t => any(done(after(t).hits)))!
let (hits, balls) = after(t1)          // simulated a second time
```

or the witness-returning form, which needs a named helper because the lambda
syntax is a single expression:

```cp
fn first_done(t: Int) -> ?State:
    let s = after(t)
    if any(done(s.hits)): s else: none

let (t1, (hits, balls)) = bsearch_first_some(0..=4e18, first_done)!
```

Both are fine and the second is what the port uses. The friction is in
choosing: the one-line version is so natural that the recomputation is easy
to miss, and `bsearch_first_some` is the least obvious of the search names.
No language change suggested; the docs for `bsearch_first` should show this
exact case and point at `bsearch_first_some`. Suggested priority: low.

### X8: building ordered containers from a sequence

The cheat sheet shows only empty constructors (`Set<T>()`, `SortedBag<T>()`).
Three places in this batch want a filled one:

```cp
let xs = SortedBag<Int>(vals)            // aac5p3: all query values
var unswept = Set<Int>(1..=n)            // noimps23p1: every position
right | Set(flip(x) for x in left)       // noimps23p1: mapped set
```

The ports assume a constructor that takes any iterable. If that is the
intent it should be on the cheat sheet; if not, each of these is a three-line
loop. Suggested priority: medium, and cheap: one sentence in section 9.

## What did not cause friction

- Event sweeps. `ucrpc21e` is a list of `(t, team, d)` records, one `sort`,
  and one loop; an `enum Team` indexes the two running rates, so
  `max(rate)` replaces `*max_element(begin(points), end(points))`.
- Grouping a sweep by day. In `rpeng3`, `(by_day[t] ??= []).push(...)` on an
  ordered `Map` and `for (_, day) in by_day.items` replace an array of half a
  million vectors.
- Nested functions that update outer variables: every `offer`, `leave` and
  `board` helper is a plain `fn` with no capture list.
- `while let x = stack.pop()` for worklists and `aside.pop()!` where the
  input guarantees an element.

## Numbers

Across the ten problems: 895 raw and 561 library-stripped C++ lines against
231 lines of CP Lang (74% and 58% shorter). Per-problem figures and how
"stripped" is counted are in `proto/ports/scout_batch_32/README.md`.
