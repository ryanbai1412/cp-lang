# Friction backlog

The prioritized list that Stage-1 scout passes feed and Stage-2 `proto/cpc`
sprints take from. A scout pass re-solves a batch of corpus problems
without touching the compiler and records every place where today's
prototype forces an awkward line. Each item gives the problems, the line we
have to write today, the general abstraction that removes it, and how many
problems it simplifies. Items already written up in
[`friction.md`](friction.md) or [`ergonomics-audit.md`](ergonomics-audit.md)
are only repeated when a new batch hits them again, and what those two
already fixed is left out.

Batch 3 (strings, number theory, geometry, interactive) has its own file,
[`friction-backlog-batch-3.md`](friction-backlog-batch-3.md), with items `G1` to `G25`.
Batch 5 (flow, matching, greedy, binary search, geometry, interactive) is in
[`friction-backlog-batch-5.md`](friction-backlog-batch-5.md), with items `K1` to `K21`.

## Batch 1 (2026-11-20)

Ten problems, sketched in
[`proto/ports/scout_batch_1/`](../proto/ports/scout_batch_1/): ccc18s5,
ccc19s5, cco18p1, ioi04p4, ioi10p3, ccc21s4, ccc22s4, ccc05s5, ioi11p2 and
cco20p1. The sketches use the syntax we want; each wished line carries the
item number below.

What was checked and what was not: every "today" claim marked *checked* was
compiled as a small probe with `proto/cp.py build` at commit `215171b`
(after the ergonomics audit), and
the error or output is quoted. The ten sketches themselves do not build
yet and have not been run against their originals. A count is the number
of sketches in this batch that carry the item's marker; an earlier port is
named only where `friction.md` or the port itself shows the same line. The
wider corpus was not counted except where a number is given.

### Status after the first Stage-2 sprint (2026-11-20)

Sixteen of the 21 items are resolved in `proto/cpc/` and `stdlib/`, and
eight of the ten sketches moved to `proto/ports/` and agree with their
originals under `proto/run_all.sh`: ccc18s5, ccc21s4, ccc22s4, cco18p1,
cco20p1, ioi04p4, ioi10p3 and ioi11p2. ccc19s5 (F4) and ccc05s5 (F13, F15)
are still sketches. Each item below starts with its status. `decisions.md`
§12 has the reasoning for every syntax or typing change.

Stressing found two bugs in the sketches themselves, which is the point of
running them: cco18p1 treated a tie in points as a rivalry (the original
needs a strict win), and cco20p1 let a friend sit outside `l..=r`.

### Suggested order for the Stage-2 sprint

1. F18, because it is a silent wrong answer.
2. F1: 4 of the 10 problems, and the same gap the audit lists as open
   under "Standard structures" and "Graph boilerplate".
3. F2, F5, F8, F9, F12: small typing and parser changes, 1 or 2 problems
   each.
4. F3, F6, F7, F19, F11, F10: one library piece each, 1 or 2 problems each
   counting earlier ports.
5. F17, F16, F4, F13, F14, F21, F15, F20: one problem each in this batch,
   or blocked on an open decision.

The audit landed while this batch was being written and removed two items
from it: the checked step (`r + step` is now a `?Row`, used as is by
ccc19s5) and `prefix(a)` over an `Int` array (used as is by ccc22s4).

### F18. A compound assignment on a missing map key (bug; ioi11p2)

**Resolved.** A missing key starts from the identity of the operation (`min=` / `max=` store the value); an operation without one is a compile error. `decisions.md` §12, `crates/cpl-cli/tests/cases/ergo/mapfold.cp`, `crates/cpl-cli/tests/cases/bad/map_fold_*.cp`; `proto/cpi` follows the same rule.

*Checked.* This prints `0 0 2`:

```cp
var m: HashMap<Int, Int> = []
m[3] max= -5
m[5] min= 5
m[6] += 2
print(m.get(3) ?? 7, m.get(5) ?? 7, m.get(6) ?? 7)
```

A missing key starts from zero, which was added for counting (`seen[x] +=
1`). Zero is the identity of `+` only, so `min=` and `max=` give a wrong
answer without any message. The general `f=` form is different again:
`m[4] gcd= 6` builds and then stops at run time with `key not in map`.
Small-to-large merging needs "keep the smaller", and today it must be
spelled out:

```cp
if let old = here.get(far): here[far] = min(old, hops)
else: here[far] = hops
```

Proposal: one rule for `m[k] f= x` on a missing key. It starts from the
identity of `f` where the library knows one (`+`: 0, `min`/`max`: insert
`x`, as `min=` on a `?Int` variable already does), and it is a compile
error otherwise. Simplifies 1 problem here; it is first because the
current behaviour is wrong without any message.

### F1. Ports cannot use the standard library (4 of 10; `friction.md` item 3)

**Resolved for these four, by stand-ins.** `use std.module.{Name}` parses and is checked against `stdlib/module.cp`; `Dsu`, `bfs`, `choose` and `Set(xs)` are compiled in with the library's names and meanings, and a library name without its `use` is an error. `cpc` still cannot compile `stdlib/*.cp` itself: that (structs with methods, generics) is the real fix and stays open as F22.

ccc18s5 (`Dsu`), ccc21s4 (`bfs`), ccc22s4 (`choose`) and ioi04p4 (`Set`).
*Checked:* `use
std.dsu.{Dsu}` gives `expected end of line (at 'std')`, and `Set` is an
unknown name. `stdlib/` runs on `proto/cpi/` only, and
`proto/cpc` has no generics, so ccc18s5, which needs a union-find over
planets and another over cities, has to write the structure twice:

```cp
var city_up: [City]?City = none
var planet_up: [Planet]?Planet = none
fn city_root(c: City) -> City: ...
fn planet_root(p: Planet) -> Planet: ...          // the same eight lines again
```

Wanted:

```cp
var cities = Dsu<City>.new()
if cities.unite(f.a, f.b): kept += f.cost * planets.count
```

Proposal: make `use std.x` work in ports. The cheapest route is to map the
names ports need (`Dsu`, `bfs`, `Fenwick`, `prefix`, `choose`, `Set`) to
builtins with the signatures `stdlib/` already has, as `SegTree` and
`Rooted` are today; the real route is generics in `proto/cpc`. One gap in
`stdlib/` itself: `choose` exists only on the modular table in
`stdlib/mod.cp`, and there is no plain `Int` one and no `Set`.

### F2. Prefix counts and running folds (2 of 10, 1 earlier port)

**Resolved.** `prefix` over a `Bool` array or fn counts; `scan(xs, f)` and `running_sum(xs)` are in the new `std.seq`.

ioi10p3 and cco20p1; earlier, aac4p2 (the audit's open "Prefix of
anything"). The audit added `prefix(a)` with `pre.sum(range)` and
`pre.sum(rows, cols)` for `Int` arrays, which ccc22s4 uses as is. Two
pieces are left. *Checked:* a `Bool` array gives `prefix takes a 1-D or 2-D
array of Int, got [R, C]Bool`, and `running_sum` is an unknown name. Today
ioi10p3 builds an `Int` table first, and cco20p1 keeps a running total by
hand:

```cp
let low = prefix([Row, Col] (i, j) => if rank[i, j] <= x: 1 else: 0)
var total = 0
for c in seen_by:
    total += c
    print(total)
```

Wanted:

```cp
let low = prefix([Row, Col] (i, j) => rank[i, j] <= x)   // a Bool array counts
for total in running_sum(seen_by): print(total)
```

Proposal: `prefix` of a `Bool` array counts the true cells, and `scan(a,
f)` gives the running fold of any two-argument function as a sequence
(`running_sum` is `scan(a, +)`), which is also what aac4p2's running `lcm`
needs.

### F5. `let ... else` over two optionals (1 of 10)

**Resolved.** Both spellings: `let (g, h) = (x.right, y.right) else:` and `let g = x.right and let h = y.right else:`.

cco18p1. The audit made a step an optional and added `b.right` for the
cell after a boundary, so the old item (moving inside a domain) is gone.
What is left is taking two of them at once and leaving early. *Checked:*
`let (g, h) = (x.right, y.right) else: return 0` gives `` `?` / let-else
needs a ?T, got (f0: ?G, f1: ?G) ``. The chained `if let g = x.right and
let h = y.right:` builds, but it puts the whole body of the DP one level
in, with the base case after it:

```cp
if let g = x.right and let h = y.right:
    let paired = ...
    return max(best(g.r, y), best(x, h.r), paired)
0
```

Proposal: `let ... else` accepts what `if let` accepts, either a tuple of
optionals or `let g = x.right and let h = y.right else: return 0`.

### F8. `none` does not pass through arithmetic (1 of 10)

**Resolved** for straight-line code: `e?` may sit inside an expression. Inside a branch, a lambda or a comprehension it is still an error that says to bind it first.

ccc21s4. *Checked:* `d[v]? + 1` gives `` `?`
must be the whole right-hand side of a let (prototype) ``. Today:

```cp
fn arrival(i: Stop) -> ?Int:
    if let walk = on_foot[line[i]]: return i.ord + walk
    none
```

Wanted: `fn arrival(i: Stop) -> ?Int: i.ord + on_foot[line[i]]?`.
Proposal: allow `e?` anywhere inside an expression of a fn that returns an
optional, not only as a whole `let`.

### F9. `min` and `max` cannot mix `Int` with `?Int` (1 of 10)

**Resolved.** `min` and `max` skip `none`; the result is a `T` when some argument is a plain `T`.

ioi04p4. *Checked:* `min(a * b, min(... for x in 1..=a / 2), ...)` gives
`min of [Int[1..=360000], ?Int, ?Int]`, because a reduction over a range
that may be empty is optional. `min` over optionals already skips `none`,
so this is one missing case. Today each inner reduction needs `?? inf`,
which brings back the sentinel. Proposal: `min`/`max` accept any mix of
`T` and `?T`; the result is `T` when at least one argument is.

### F12. A lambda that only reads a field (2 of 10)

**Resolved.** `.field` where a one-argument fn is expected.

ccc18s5, ioi11p2. *Checked:* `sorted(a,
by: .cost)` gives `expected an expression (at '.')`. Today: `sorted(flights,
by: f => f.cost)`. Proposal: `.field` where a function of one argument is
expected means `x => x.field`. Small. The earlier ports that sort by a key
use a tuple or a call (`by: r => (r.len, r.cost)`), which this does not
shorten.

### F3. Input that is laid out differently from the data (1 of 10)

**Resolved.** `read(columns)` for an array of records, and single-character `@tags` read packed.

cco18p1 reads `WLLW` and then a line of points for each team, and the
program wants one record per game. *Checked:* `[G]Outcome` with
`@tags("W", "L")` compiles, but on `WLLW` it stops with `unknown tag`: tags
are read as separate words. Today:

```cp
let results: [domain G]Char = read()
let points: [Game]1..=1e6 = read()         // two arrays, and 'W' compared by hand
```

Wanted:

```cp
type Season = [Game](outcome: Outcome, points: 1..=1e6)
let geese: Season = read(columns)
```

Proposal, two independent pieces: a tagged enum whose tags are single
characters reads from a packed word (this is also every `.`/`#` grid), and
`read(columns)` fills an array of records one field at a time. Grid
problems were not counted, but they are a large share of the corpus.

### F6. Windows of a domain (1 of 10)

**Resolved.** `D.windows(k)` and `r.windows(k)` give the ranges of k consecutive positions.

ioi10p3. *Checked:* `V.windows(2)` gives `type V has no method windows`.
With the audit's typed step and `span`, today's form builds and is
already free of raw indices:

```cp
for i in Row:
    if let j = i + (h - 1):
        let rows = span(i, j)
```

Wanted: `for rows in Row.windows(h)`, each window a range of `Row` that
`prefix(...).sum` and slicing accept. In a 2-D search it turns four nested
lines into one generator.

### F7. A fold that follows its source (1 of 10, 1 earlier port)

**Resolved.** `SegTree<D, T>.of(op, f)` with `refresh(i)` and `.all`, in `stdlib/segtree.cp` and in the stand-in.

ccc21s4; earlier, ccc21s5 (`for i in I: gcds.set(i, a[i])`). Today the
formula is written at the build and again after every change:

```cp
var fastest = SegTree<Stop, ?Int>(none, (a, b) => min(a, b))
for i in Stop: fastest.set(i, arrival(i))
...
swap(line[x], line[y])
fastest.set(x, arrival(x))
fastest.set(y, arrival(y))
```

Wanted:

```cp
var fastest = SegTree<Stop, ?Int>.of(min, arrival)
swap(line[x], line[y])
fastest.refresh(x, y)
```

Proposal: `SegTree.of(op, f)` keeps `f` and `refresh(i, ...)` recomputes
those leaves, so the formula exists once. Also take a named operator
(`min`, `gcd`) where an operation is expected, with its identity.

### F19. Range updates read back as runs (1 of 10, 1 earlier port)

**Resolved.** `Steps` in the new `std.steps`: `add(range, x)`, then `runs(clip)`.

cco20p1; earlier, ccc21s5, which builds a difference table and then takes
its prefix. Today
(cco20p1, following the original):

```cp
var events: List<(at: Int, change: Int)> = []
events.push((x - reach, 1))
events.push((x + reach + 1, -1))
sort(events, by: e => e.at)
// then a loop that groups equal `at`, clips to l..=r and remembers the previous point
```

Wanted:

```cp
var watchers = Steps<Int>()
watchers.add(x - reach ..= x + reach, 1)
for (span, count) in watchers.runs(l..=r): seen_by[count] += span.len
```

Proposal: a library `Steps<K>` (a sparse difference map) with `add(range,
x)` and `runs(range)`, which yields each maximal range with one value. The
grouping loop with its off-by-one end points is the whole difficulty of
cco20p1.

### F11. Ranges that wrap (1 of 10, 1 earlier port)

**Resolved.** `v.cyc(k)` and `pre.sum(from: v, len: k)`, which wraps around the domain.

ccc22s4; earlier, btoi17p3 (`tour[(i + 1) % tour.len]`, open in
`friction.md`). *Checked:* `p.cyc(3)` gives `V has no method
cyc`, though `plan/12` specifies it, and `pre.sum(p.., len: 2)` gives
`` `sum` on Prefix<V> takes 1 range(s) ``. The original doubles the array to
length 2c to avoid wrapping. Wanted: `ring.sum(p.cyc(1).., len: c / 2)`.
Proposal: implement `v.cyc(k)`, and let `prefix(...).sum` and slices take a
start and a length that may pass the end.

### F10. Binary search over a plain range (1 of 10)

**Resolved.** `first_where` and `partition_point` on any range, not only a domain's.

ioi10p3, and any "binary search the answer" problem. *Checked:*
`(1..=n).first_where(...)` gives `Range<Int> has no method first_where`; it
exists on domains only, so the port would declare `domain Answer = 1..=r *
c` for one line. Proposal: `first_where` and `partition_point` on any
range.

### F17. Taking a value out of an array slot (1 of 10)

**Resolved.** `take(place)` moves the value out and leaves the empty one. `reach[v] = here` as the last use of `here` still copies: see F23.

ioi11p2. *Checked:* `take` is an unknown name. Small-to-large needs to move
a child's map out of `reach[child]`, swap it with the local one if it is
larger, and store the result. Without a move, `var there = reach[child]`
copies the map and the algorithm loses its bound. Proposal: `take(place)`
returns the value and leaves the type's empty value, and `swap(a, b)`
works on two local containers.

### F16. Lengths on a rooted tree (1 of 10)

**Resolved.** `Rooted(edges, root, weight: .len)`; `tree.dist(a, b)` adds lengths, and `tree.kids` and `tree.root` are readable. In the library: `Rooted.weighted` and the `far` field.

ioi11p2. `Rooted(adj, root)` gives `parent`, `pos`, `depth`, `lca` and
`preorder`. *Checked:* `tree.children(v)` gives `no method children on
Rooted<V>`, and there is no weighted distance, so the port would rebuild
both with its own DFS. Wanted: `Rooted(roads, root, weight: .len)` with
`tree.children(v)` and `tree.dist(v)`. With those, the
bottom-up loop `for v in tree.preorder.rev` replaces recursion, which also
avoids the stack depth problem on a path of 2e5 cities.

### F4. A triangular index (1 of 10, 1 earlier port)

**Open.** Not started; ccc19s5 stays a sketch.

ccc19s5; earlier, coci20c6p3 (`choose[i, j]` with `j <= i`). Today the
triangle is a square table with half of it unused, the input needs a loop
with a row-dependent count, and every loop repeats `if c <= r`. Wanted:

```cp
type Cell = (r: Row, c: Row) where c <= r
let tri: [Cell]0..=1e9 = read()
for (r, c) in Cell: ...
```

Proposal: this is a user `Index` type (plan/19 section 3.1) whose `to_idx`
is `r * (r + 1) / 2 + c`; what is missing is deriving it from a `where` on
a tuple of domains, and reading it. Interval DP tables (`l <= r`) are the
same shape.

### F13. A sorted bag with rank queries (1 of 10)

**Open.** Not started; ccc05s5 stays a sketch (it also needs F15).

ccc05s5 (`seen.count_above(s)`), and ccc21s4's original, which keeps a
multiset only to read its minimum. Today: `compress` the scores, then a
`SegTree` of counts, with the `coord.first_ge(s)!` that `friction.md`
already lists. Proposal: `SortedBag<T>` with `add`, `remove`, `count_below`,
`count_above`, `min`, `max` and `nth`, written over `Fenwick` when all the
values are known up front and a balanced tree otherwise.

### F14. Merging two sorted sequences of different types (1 of 10)

**Open.** Not needed in the end: ccc18s5 puts both kinds of link in one `enum Link` list and sorts it by cost, which reads better than a merge.

ccc18s5 has flights between cities and portals between planets, and
Kruskal wants them in one order by cost. Today that takes a new enum and a
concatenated list, sorted again. Wanted: `for link in merge(a, b, by:
.cost)`, yielding `Left(x)` or `Right(y)`. Low priority: one problem.

### F15. Floats (1 of 10; decision 1.5 is open)

**Open.** Blocked on decision 1.5.

ccc05s5 prints an average to two decimals. *Checked:* `F64` is an unknown
name in the prototype. ccc20s5, whose accepted solutions use `long
double`, was left out of this batch for the same reason. This needs decision 1.5 before a sprint.

### F20. Grader-style originals (2 of 10; decision 6.8 is open)

**Open** (decision 6.8). Worked around: the two grader-style originals got a small `main` that stands in for the grader, marked as such in the file and counted as driver lines in the `.strip`.

The originals of ioi10p3 and ioi11p2 have no `main`: they are functions
called by a grader, so `cp.py stress`
cannot use the original as an oracle. 106 of the 747 accepted problems in
the corpus are IOI problems; how many are in this style was not counted. Until 6.8 is
decided, a small `main` wrapper beside the original would let them be
stressed.

### F21. Two small literals (1 of 10)

**Resolved.** `[k: v]` map literals and `[n - 1]T` counts.

ioi11p2. *Checked:* `let es: [n - 1](a: V, b: V) = read()` gives `expected
a type`, and `var m: HashMap<Int, Int> = [3: 4]` gives `expected an
expression (at ':')`. Today the first needs `domain Road = 1..n` for an
array that is only read and looped over, and the second is an empty map
plus an assignment. Proposal: any expression as an array count, and `[k:
v, ...]` as a map literal.

### New friction found while landing the batch

- **F22. `cpc` cannot compile the library.** Every library piece a port
  uses is a hand-written stand-in in `compiler.py` and `runtime.hpp`,
  kept in step with the `.cp` definition by hand. Each new structure costs
  a stand-in, and the two can drift. The fix is structs with methods and
  generics in `cpc`, or porting through the Rust front end.
- **F23. A last use is not a move.** ioi11p2 ends its loop with
  `reach[v] = take(here)`; `reach[v] = here` copies the map although
  `here` is never read again. Storing the last use of a local should move.
- **F24. A match arm cannot hold a one-line `if`.** ccc18s5 wanted
  `Flight(a, b) => if cities.unite(a, b): kept += cost * planets.count`;
  it parses as an `if` expression and asks for `else`. The arm has to
  become a block.
- **F25. A generator fn cannot name an anonymous count.** With
  `let roads: [n - 1](...) = read(gen: random_tree)` the fn has no way to
  write its return type, so ioi11p2 declares `domain Road = 1..n` only for
  that.
- **F26. Random valid input for a permutation.** ccc21s4 needs "station 1
  first, then a permutation, and two different stops per swap". With
  `where` alone 59 of 60 generated cases were rejected; it now has a
  hand-written `random_line`. `Perm<D>` has no generator hook for "fixed
  first element", and there is no `shuffle`.
- **F27. Swapping two cells.** `swap(mut line[x], mut line[y])` is
  rejected unless `x != y` is known, which is right, but the fix
  (`line.swap(x, y)`) is not suggested by the error.
- **F28. The first k positions of a domain** is spelled
  `Spot.all.prefix(c / 2)` in ccc22s4. `Spot.first(k)` would read better.

### Already good

Things this batch needed that built on the first try as probes, for the
record: `let x = opt else: return 0`, a range bounded by an earlier input
(`let r: l..=5`), `[0..=n]Int` tables, `@memo` over bounds, `fewest min= x`
on a `?Int`, `while let`, `sorted(a, by: e => e.cost)`, and from the audit
`if let below = r + step and let right = c + step`, `span(i, j)` and
`prefix(a).sum(range)`.

## Batch 2 (2026-11-20)

Ten graph, tree and range-query problems, ported on top of `stdlib/` and
run on the reference interpreter. Its items are in
[`friction-backlog-batch-2.md`](friction-backlog-batch-2.md), numbered B1
to B12.
