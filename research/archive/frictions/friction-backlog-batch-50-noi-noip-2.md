# Friction backlog, scout batch 50: hard NOI and NOIP classics, part 2

Ten more accepted C++ submissions from the `noi*` / `noip*` bucket were
re-solved against the locked sheet. The ports live in
`proto/ports/scout_batch_50/`; the runnable forms in `today/` were checked
against the originals with `check.py` (the README there says what is and is
not covered). The task named the v2 sheet; `main` moved to v2.1 while the
batch was in progress, so the sketches are written against v2.1 and the items
below are what is still open there.

The ten: `noip02p4` (rectangle cover), `noi03p4` (tree detour), `noi04p5`
(rod-cutting DP on decimals), `noip08p4` (twin-stack sorting), `noi17p1`
(huge integer with bit queries), `noi20p5` (surreal trees), `noi22p1`
(majority over joined sequences), `noi22p4` (tree isomorphism after leaf
deletions), `noi23p1` (stroke union on a grid), `noip22p2` (constructive
card clearing).

## Dual LOC

Lines that are neither blank nor a pure comment, from `loc.py`. "Stripped"
drops the contest template (includes, debug macros, aliases, stream setup).

```
problem      raw stripped   v2 today
noip02p4      97       69   14    33
noi03p4       80       53   10    32
noi04p5       99       71   11    24
noip08p4     165      137   28    52
noi17p1      189      162   23    52
noi20p5      131      104   29    45
noi22p1      156      128   47    74
noi22p4      174      145   41    62
noi23p1      161      137   32    72
noip22p2     177      149   52    66
total       1429     1155  287   512
```

The sketches are 80% shorter than the raw originals and 75% shorter than the
template-stripped ones. The forms that build today are 64% and 56% shorter.

Part of that is the algorithm, not the language, and it should not be
credited to syntax. `noi17p1` drops an interval map of 32-bit words for two
growing numbers and a set of differing digits; `noi04p5` replaces a
precomputed `opt` table with a closed form; `noi22p1` replaces a segment-tree
merge with per-bit counts. Rethinking was the brief, but a C++ rewrite of the
same three ideas would also be much shorter than the originals. The four
ports that keep the original's algorithm (`noip08p4`, `noi20p5`, `noi22p4`,
`noip22p2`) are 535 stripped lines against 150, a 72% reduction.

## Where the sheet read well

`noi03p4` is ten lines because the whole solution is three shortest-path
calls and one generator:

```cp
fn farthest(d: [Home]?Int) -> Home: max_by(Home, v => d[v]!)!
let a = farthest(dijkstra(g, from: Home.first))
let da = dijkstra(g, from: a)
let b = farthest(da)
let db = dijkstra(g, from: b)
print(da[b]! + max(min(da[c]!, db[c]!) for c in Home)!)
```

The five `!` assume `dijkstra` returns `[V]?Int` (the sheet lists the
function but not its type), on a graph that is known to be connected (NP5 below).

`noi04p5`: `?Int` arithmetic that propagates `none`, `min` over a generator
and `@memo` over three ranges turn a 40-line table fill into the recurrence:

```cp
@memo
fn finish(at: 0..=100, pieces: 0..=n, parts: 0..=m) -> ?Int:
    if pieces == 0: return if at == 100 and parts == 0: 0 else: none
    min(finish(at + len, pieces - 1, parts - p) + a * len ** 2 + b * even_split(len, p)
        for len in 1..=100 - at for p in 1..=min(len, parts))
```

`noi17p1`: an array over `[Bool, Digit]` lets the sign of the update pick the
number it grows, and `Set.last_lt` is exactly "the highest lower digit where
the two differ":

```cp
let total = num[a > 0, d] + carry
num[a > 0, d] = total % BASE
...
let borrow = if u != w: u < w
             elif let h = differ.last_lt(d): num[true, h] < num[false, h]
             else: false
```

`noip08p4`: `[Bool]List<Int>` indexed by the 2-colouring removes the
duplicated "stack one / stack two" code the original carries.

## Friction in v2.1

### NP1. A place whose index expression grows the container

`noi20p5` walks a trie and marks two fresh children as terminal:

```cp
trie[step(o, BothGoRight)].ends = true     // step may push onto trie
```

The sheet says index and right-hand side are evaluated before the place is
taken, so this is fine on paper. The prototype evaluates the place first:
the today-form `ends[step(o, 2)] = true` died with "list index out of range"
under the harness and had to become

```cp
let (go_left, go_right) = (step(o, 2), step(o, 3))
ends[go_left] = true
ends[go_right] = true
```

This is the one item in the batch that produced a runtime failure rather
than a compile error, and it is a pattern every arena trie hits. It needs a
differential test in the Rust frontend, and the emitted C++ must not bind a
reference to `trie[...]` before calling `step`.

### NP2. Matching on a tuple of optionals does not narrow

`noi20p5` dispatches on which children exist:

```cp
match kid[v]:
    (none, none) => trie[o].ends = true
    (l, none) => (at, o) = (l, step(o, OnlyLeft))
    (none, r) => (at, o) = (r, step(o, OnlyRight))
    (l, r) =>
        if leaf(l!) and leaf(r!): ...
```

In the last arm both are known present (the `none` arms came first) but `l`
and `r` are still `?V`, so the body is full of `!`. The sheet has no
"present" pattern for an optional inside a tuple. A `some`-less spelling such
as `(l?, r?)`, or narrowing by arm order, would remove four `!` here.

### NP3. Two elements of one `List<Struct>` at once

`noi22p1` keeps sequences in `List<Seq>` and joins two of them. Draining one
element into another is two `mut` borrows of `seqs`:

```cp
for v in seqs[back].items.take(): seqs[front].push(v)
```

`take()` is the escape (it leaves an empty deque behind), and the sheet
does list `.take()` / `.absorb(mut other)` for small-to-large. What it does
not say is whether the direct form is legal:

```cp
seqs[front].absorb(mut seqs[back])         // two places in one List: allowed?
```

`swap(mut a[i], mut a[j])` is on the sheet, so two `mut` places of one
container are accepted at least there. If the same holds for any call (with
a trap when `front == back`), one line saying so removes the hesitation; if
it does not, the sheet should show the `take()` form as the idiom.

### NP4. Small-to-large with order: no front absorb

Same port: when the front sequence is the smaller one it has to be
prepended to the back one in order.

```cp
for v in seqs[front].items.take().rev:
    seqs[back].items.push_front(v)
    seqs[back].count(v, 1)
```

`Deque` has `push_front` but nothing that says "splice this deque in at the
front". Ordered small-to-large (sequence join, rope-like merges, Euler-tour
splices) is common enough in NOI problems for `absorb_front` / `absorb_back`
on `Deque` to earn a place; an unordered `absorb` on `HashMap` covers the
count half.

### NP5. `dijkstra` on a graph known to be connected

Shown above in `noi03p4`: if distances are `?Int`, every read carries `!`.
For a tree, `Rooted` has `.depth(v)` and `.dist(a, b)`, but the sheet does
not say whether they follow edge weights, and there is no "all distances
from `a`" on a tree. A weighted `tree.dists(from: a) -> [V]Int`, or
`dijkstra(...)!` lifting the whole array once, would make this port
punctuation-free.

### NP6. Derived ordering of enum events

`noi23p1` sorts sweep events and relies on "leave, then enter, then ask" at
equal `y`:

```cp
enum Event = Leave(x: Int) | Enter(x: Int) | Ask(lo: Int, hi: Int)
for (y, e) in sorted(events): ...
```

This is correct only if `enum` orders by declaration order of variants, and
the sheet does not say so. It is also fragile: reordering the variants
silently changes the answer. Every sweep-line problem has this tie-break.
The sheet should state the derived order explicitly; a `sorted(events, by:
e => (e.y, e.kind))` with a visible key is the safer habit to teach.

### NP7. Multiset difference of two small lists

`noi22p4` pairs off children of equal shape and keeps what is left on each
side. There is no one-liner, so the sketch goes through a `SortedBag`:

```cp
var theirs = SortedBag<(Int, Int)>((shape[c], c) for c in kids[v])
var left: List<Int> = []
for c in kids[u]:
    if let (s, d) = theirs.first_ge((shape[c], 0)) and s == shape[c]: theirs.remove((s, d))
    else: left.push(c)
```

and the today-form is a 12-line two-pointer merge. A `cancel(xs, ys, by:)`
returning the unmatched remainder of both sides is small, general (it is
also "diff two multisets" in hashing problems) and would be three lines
here. Low priority: one occurrence in this batch.

### NP8. Multitest after a shared header

`noi22p4` reads `tc_id tests k` once and then the cases; `k` is shared.
The sheet has `@multitest fn solve():` with "code above runs once", which is
the right shape, but it does not say where the case count comes from when
the header line carries other tokens around it. The sketch invents

```cp
let (tc, tests, k): (Int, 1..=500, 0..=5) = read()
@multitest(count: tests)
fn solve():
```

NOI-style inputs with a subtask id in front of the count are the norm from
2020 on (`noi22p4`, `noi23p1` and `noi17p1` all start with extra tokens), so
this needs a stated spelling.

### NP9. Reading a decimal with a fixed number of places as an exact integer

`noi04p5` gets prices like `1.5`. The sketch reads `F64` and rounds:

```cp
let (pa, pb, n, m): (F64, F64, 1..=100, 1..=100) = read()
let (a, b) = (round(pa * 10), round(pb * 10))
...
print("{F64(finish(0, n, m)!) / 10:.1f}")
```

It works, but it routes exact data through floating point twice. A
`read(scale: 10)` into `Int` and a `{x:/10.1}`-style fixed-point print would
keep the port all-integer. Low priority; "money with two decimals" is rare
but always a trap when it appears.

### NP10. Iterating over a few named mutable containers

`noip22p2` keeps three pools and removes a stack from all of them:

```cp
for pool in [mut empty, mut single, mut ripe]: pool.remove(s)
```

I do not think this is expressible (a list of `mut` borrows is a stored
borrow). The alternative is three lines, or `[Kind]Set<S>` indexed by an
enum, which is the better design and what the port should probably use; the
friction is that nothing nudges the writer there.

## Multi-feature collisions

Pattern arm against parallel assignment (NP2): `(l, r) => (at, o) = (l, ...)`
has a binding tuple and an assigning tuple on one line. It reads correctly
once you know the rule, but it is the densest line in the batch.

Index evaluation order against growable arenas (NP1 with `push_id`):
`trie[o].next[s] ??= trie.push_id(Node())` takes a place inside `trie` and
then grows `trie` on the right. With `List` backed by a vector this is a
dangling reference in naive C++. The sheet's rule (right side first) has to
hold for `??=` and compound assignment too, not only for `=`.

`@memo` against nested functions and locals (`noi22p4`): `prune` is
memoised, captures `shape`, `size` and `kids` from the enclosing per-test
function, and contains a nested recursive `place` that calls back into
`prune`. The memo table has to be per test and keyed on `(u, v)` only. The
today-form does this by hand with a `Map` declared inside the loop, which
confirms the semantics wanted; the sheet does not say that a nested `@memo`
is reset when its enclosing function is re-entered.

`?T` comparison against `Deque.last` (`noip08p4`, `noip22p2`):
`stacks[s].last == card` compares `?Card` with `Card` and is the cleanest
line in both ports. It depends on `none == x` being `false` rather than an
error; with `(stack.last ?? n + 1) > a[i]` right next to it, where ordering
does need a default, the two rules sit side by side and should both be on
the sheet.

## Prototype gaps (today-forms only)

These are gaps between `proto/cpc` and the sheet, recorded so the today-forms
are explainable; nothing under `proto/cpc/` was changed.

- Parallel tuple assignment to existing variables, `(at, o) = (l, step(...))`,
  is on the sheet but rejected ("a tuple is a value, not a place to assign
  to"). Three ports wanted it.
- A string literal with `{...}` passed to `print` is printed literally, with
  no diagnostic. `noi04p5` first printed `{best / 10}.{best % 10}`. Silent
  wrong output is worse than an error here.
- `m[(s, x)] += 1` on a `HashMap` with a tuple key emits C++ that does not
  compile (the braces of the tuple literal split a macro argument).
  `noi22p1` packs the key into one `Int` instead. Plain `=` on a tuple-keyed
  `Map` works.
- `const BASE = 1 << 30` followed by `total % BASE` is rejected with
  "`BASE` may be 0"; writing the literal `1073741824` is accepted.
- No `F64`, and `Str` can be neither indexed nor iterated, so `noi04p5`
  reads `1.5` as two integers around the dot.
- `List` has `.last()` as a method, not a property, and no `.items`.
- `Set` has `insert`, `remove`, `min`, `max`, `contains` only: `noi17p1`
  hand-rolls a max tree for `last_lt` and `noi23p1` a Fenwick for `rank`.
- No `Deque`, no struct arenas, no list slices, no `Graph` / `dijkstra` /
  `two_color`, no per-test `domain` inside a loop.
- Chained indexing `g[i][k][j]` on a product-domain array is an error with a
  good message pointing at `g[i, k, j]`.

## Findings about the originals

- `noip08p4` (`5185830.cpp`): its script is not always the lexicographically
  least. For `3 2 4 1 5` it prints `a a c a b b b d a b`, and
  `a a c a b b b a d b` is smaller and valid. An exhaustive search over all
  scripts confirms the smaller one; the port produces it.
- `noi22p1` (`4800882.cpp`) is randomized (20 probes per query). The port is
  deterministic.
- `noi22p4` (`4802121.cpp`) ignores the `k` it reads and uses 5.

## Suggested order

NP1 first: it is a correctness hazard in the emitted C++, not an ergonomic
one, and the prototype already gets it wrong. NP8 next, because it affects
every recent NOI input format. NP2 is a small pattern extension that pays
off in every binary-tree problem. NP3 and NP4 are one design question
(moving data between two elements of one container). NP6 is one sentence on
the sheet. NP5, NP7, NP9 and NP10 are library conveniences with one sighting
each.
