# Friction backlog, batch 17: statement first

Ten problems from `corpus/` were solved in CP Lang before their accepted C++
was opened. The ports are in
[`proto/ports/scout_batch_17/`](../proto/ports/scout_batch_17/). All ten build
with today's `proto/cpc` and agree with their originals on 100 small and 20
medium random inputs each (`stress.sh 100 6`, `stress.sh 20 2000`).
`proto/cpc/` and `crates/` were not changed.

One caveat about the method. The statements were not fetched:
`dmoj.ca/api/v2/problem/<code>` returns the name, limits, points and tags but
no statement text, and `dmoj.ca/problem/<code>` answers 403 with a bot
challenge. The ten were therefore chosen among published tasks whose
statements are well known (IOI, APIO, Baltic OI, NOI, the AtCoder DP contest)
and written from those. Input formats were guessed from the published tasks
and confirmed against the C++ afterwards; all ten guesses were right except
that two tasks are judged through a function on DMOJ (W11).

Overlap: four of the ten were also ported by batches that landed on `main`
while this one was in progress (`dpz` in `proto/ports/`, `ioi11p3` and
`apio07p2` in `scout_batch_13/`, `btoi19p3` in `scout_batch_9/`). They were
written here without sight of those ports, so only six problems are new.

Items are numbered `W1`.. so they can be cited.

## (a) Where the statement led somewhere else

Six of the ten ports ended up with a different algorithm or structure from the
accepted C++. In each case the C++ reached for a template the author already
had; starting from the problem, the smaller tool was the natural one.

**Frog 3 (`dpz`).** The C++ pastes the KACTL `LineContainer`, a multiset of
lines with floored-division intersections, and since that container answers
maximum queries it negates every line and walks the stones right to left. The
port wants the lowest line at `x = h[s]` for stones `s`, so it keeps a Li Chao
tree over the stones. Two lines are only ever compared at a stone:

```
fn add(f: Line, node: Node, lo: Stone, hi: Stone):
    let mid = Stone((lo.label + hi.label) / 2)
    let g = winner[node] ?? f
    let (keep, sink) = if value(f, mid) < value(g, mid): (f, g) else: (g, f)
    winner[node] = keep
```

There are no intersection points, so no product of two 10^17 values and no
128-bit arithmetic. That matters here because the prototype has neither a wide
integer nor a float (W12).

**Intervals (`dpw`).** The C++ file is 270 lines, most of it a generic lazy
segment tree. The port is twelve lines of code: `AddMax<Pos>` is exactly the
structure the sweep needs.

```
var best = AddMax<Pos>()
for p in Pos:
    if let q = p - 1: best.add(p, p, max(0, best.max(Pos.first, q)))
    for (l, score) in ending[p]: best.add(l, p, score)
print(max(0, best.max()))
```

**Rice Hub (`ioi11p3`).** The C++ binary-searches the right end for every left
end. From the statement the two-pointer argument is the first thing one sees
(dropping the leftmost field never makes a run dearer), and it is shorter.

**Backup (`apio07p2`).** The C++ is a divide and conquer that merges four
boundary states with a convex min-plus convolution, 132 lines. The port is the
greedy with undo: take the cheapest gap, leave behind a gap that costs
`left + right - taken`. The optional step `g - 1` on a domain value makes the
linked row of gaps read well:

```
var left: [Gap]?Gap = [Gap] g => g - 1
var right: [Gap]?Gap = [Gap] g => g + 1
```

**Super Piano (`noi10p2`).** The C++ binary-searches the value of the k-th
best chord with an order-statistics structure, then sweeps again with a `set`,
205 lines. The port keeps a heap of "start cut, range of end cuts, best end"
and splits a range when its best is taken, about 25 lines.

**Alpine Valley (`btoi19p3`).** The C++ answers online with three binary
lifting tables of `17 * n` entries. The port answers offline: in preorder the
last village seen at each depth is an ancestor of the current one, so one
table over depth holds the path to the exit and the query is a range minimum.

```
for v in tree.preorder:
    let here = Depth(tree.depth(v))
    turn[here] = none
    if let d = shop_under[v]: turn[here] = d - 2 * far[v]
    best_turn.refresh(here)
```

`Rooted`, `tree.preorder`, `tree.is_anc` and `SegTree.of(min, ...)` carried
this one; nothing had to be written by hand.

The other four (`ioi13p1`, `ioi12p4`, `ioi07p4`, `ioi08p4`) came out as the
same algorithm as the C++, without the recursion: `tree.preorder.rev` and an
explicit stack replaced every `dfs`.

## (b) Constraints the types could not say

Each of these is a statement constraint that a typed `read()` cannot express
today, so `cp.py stress` either rejects cases or needs a hand-written
generator.

**W1. Ordered pairs, `l <= r` (`dpw`).** Written as a `where`, the generator
throws away half the cases at size 6 and every case at size 3000 ("0 tests
agree (30 rejected by where)"), since all 3000 intervals must be ordered at
once. The port needs a generator function, a row type and a helper:

```
fn interval() -> Interval:
    let (a, b) = (rand(1, n), rand(1, n))
    (Pos(min(a, b)), Pos(max(a, b)), rand(-1e9, 1e9))
```

Wanted: `r: l..=n` inside a row type, as `let len_max: len_min..=n` already
works between two top-level reads.

**W2. Sorted input (`ioi11p3`, `dpz`, `apio07p2`).** Positions arrive sorted,
and in two of the three strictly increasing. Each port carries a five-line
generator that accumulates random steps. Wanted: a `sorted` or `increasing`
marker on the array type.

**W3. All values different (`ioi08p4`, `btoi19p3`).** The 2n teleporter
endpoints are distinct, and so are the shop villages. Teleporters needed a
shuffle written out by hand inside the generator. For the shops the port gives
up and lets duplicates through (harmless to both programs, but not the
statement's input). Wanted: `Perm<D>`, and a "k distinct values of D" input.

**W4. A bound that depends on later input (`noi10p2`).** `k` is at most the
number of chords, which depends on `n`, `len_min` and `len_max`, two of which
are read after `k`. The only place the condition fits is a `where` on the last
read, and it rejects 45% of small cases:

```
let chords = sum(min(n, s + len_max) - (s + len_min) + 1 for s in 0..=n - len_min)
let beauty: [Note]-1000..=1000 = read() where k <= chords
```

**W5. Shape constraints (`ioi13p1`, `ioi12p4`).** "The trails form a forest"
and "the blocks are connected with no holes" are not types. The forest
generator is one line. The city generator only builds one family of valid
cities (one unbroken run per street), so the port is tested on less than the
statement allows.

**W6. Input that is a string over a small alphabet (`ioi07p4`).** `[Ship]Char`
reads the line, but nothing says the characters are `M`, `F` or `B`, so a
generator is required. An `enum` with single-character tags read as an
unspaced string would say it.

**W7. The count is the first line, not a constraint.** `shops: [s]V` and
`m: 0..n` work, but a generator cannot be given for a scalar that depends on a
later one, which is what W4 ran into.

## (c) Bare-core friction and feature interactions

Three of these are wrong code from the prototype, not ergonomics.

**W8. A destructured list element dangles after a push (wrong answers).**
`ioi13p1` first read its queue like this:

```
while next < walks.len:
    let (u, d) = walks[next]
    for (v, t) in adj[u]:
        if v != came_from[next]: walks.push((v, d + t))
```

The emitted C++ binds `u` and `d` as references into the vector, and the push
reallocates it. With `n = 2` and one trail of length 1 the port printed 2. This
is undefined behaviour reachable from ordinary code, with no diagnostic. A
`let` of a list element should copy, or the borrow should forbid the push.

*Fixed.* A destructuring `let` copies (decision 15.4, batch 12 L0;
`crates/cpl-cli/tests/cases/redteam/let_destructure_push.cp`).

**W9. Tuple assignment compiles to nothing.** In `dpz`,

```
if s <= mid: (node, hi) = (Node(2 * node.label), mid)
```

builds, and assigns nothing (the C++ assigns to a temporary), so the loop
never ended. Next to it, `var (node, lo, hi) = (...)` declares three
constants. Both should either work or be errors.

*Fixed.* Assigning to a tuple is a compile error that says to assign each
part on its own line (`crates/cpl-cli/tests/cases/bad/tuple_assign.cp`); whether it should
assign is left to the language pass. `var (a, b) = ...` declares vars
(`crates/cpl-cli/tests/cases/redteam/var_destructure.cp`). Decision 15.12.

**W10. Programs that pass the front end and fail in `g++`.** A table built
from `rand` and narrowed to a range element type
(`let spots: [Field]1..=len = [Field] _ => rand(1, len)`), and a
`HashMap<(Int, Int), Int>`, both build and then fail to compile as C++.

*Fixed.* Both build and run: batch 6 S5 and batch 5 K1
(`crates/cpl-cli/tests/cases/redteam/rand_table_narrow.cp`, `pair_key_map.cp`).

**W11. Tasks judged through a function.** `ioi13p1` and `ioi12p4` are
`travelTime(n, m, l, a, b, t)` and `DistanceSum(n, x, y)` on DMOJ; the accepted
C++ has no `main`. The ports are whole programs, and the batch ships a small
`main` per original under `harness/` to compare them. `cpl build` has no way
to emit a solution as a function with a given signature, so these two could
not be submitted as written.

**W12. No float and no wide integer.** `ccc20s5` (a probability, printed to
six places) was picked and dropped: the prototype has no floating-point type.
The usual convex hull trick for `dpz` needs a product beyond 64 bits; the Li
Chao tree was chosen partly to avoid it.

**W13. Optionals are right for "not reached", then get in the way.**
`ioi13p1` started with `[V]?Int` distance tables. `?Int` has no ordering
("handle `none` explicitly"), so comparing two reached distances needed
`?? 0` on both sides. The port was restructured to return `(village,
distance)` pairs instead. The same shows up in aggregates: `min(... for v in
tree)` over a list known to be non-empty is `?Int`, and both uses end in
`?? 0`. And in `apio07p2`, after `l != none and r != none`:

```
cost[g] = cost[l ?? g] + cost[r ?? g] - c       // WISH: if let (a, b) = (l, r)
```

**W14. `if let` binds only as a statement, and immutably.** There is no
`if let` expression (`turn[here] = if let d = ...: ... else: none` is a parse
error), and the bound name cannot be reassigned, which ruled out the usual
"swap, keep the better one" in the Li Chao insert.

**W15. `.idx` and `.label`.** `Stone((lo.idx + hi.idx) / 2)` builds and traps
at run time with "domain value out of range": `.idx` is the zero-based offset
and the constructor takes the label. Every midpoint, `2 * node` and "next
element" in this batch is `D(x.label + 1)`-style noise:

```
node = Node(2 * node.label + 1)
lo = Stone(mid.label + 1)
```

Wanted: a midpoint and a successor on domain values (`mid(lo, hi)`,
`lo.next`), which would cover the binary-tree and binary-search cases without
leaving the domain.

**W16. Small missing pieces, each hit once or twice.** No `loop:` (written
`while true:`). No `Int.MAX`. No block lambda in a table constructor, so a
generator with running state is a `var` table and a loop. No `max(xs, by:)`
(a five-line `farthest`). No `MaxHeap`, so `noi10p2` stores `loss = -beauty`.
`[m] _ => ...` is rejected where `[I] _ => ...` over `domain I = 1..=m` is
accepted. A string literal cannot be indexed (`"MFB"[rand(0, 2)]`), and a
list literal is not a table. No `shuffle` or `swap` of two table cells in a
generator.

**W17. Output of mixed kinds.** `btoi19p3` prints `escaped`, `oo` or a number
per question, in input order, but answers them out of order. It keeps two
tables and decides at print time:

```
for a in Ask:
    if escaped[a]: print("escaped")
    elif let d = nearest[a]: print(d)
    else: print("oo")
```

An enum with a payload would hold this; the port did not try one because a
record with an enum field is a known `g++` failure.

## Verification and max-N risk

Small and medium inputs: all ten agree with the accepted C++ (`noi10p2` on the
55 of 100 small and 19 of 20 medium cases its `where` lets through).

Largest inputs, checked build at `-O2`, port only: `noi10p2` with
`n = k = 5e5` runs in 0.20 s and 49 MB; `ioi08p4` with `n = m = 1e6` in 0.30 s
and 86 MB; `ioi07p4` with `n = 1e5` in 0.12 s; `dpz` with `n = 2e5` in 0.02 s;
`btoi19p3` with `n = q = 1e5` on a path in 0.06 s and 33 MB. The other five
were not timed at full size. Of those, `ioi12p4` is the one to watch: it keys
an ordered `Map` by coordinate pairs (W10 ruled out the hash map), about 5e5
lookups at `n = 1e5`. `ioi11p3` declares the budget up to 2e15 as the IOI
task does, while the accepted C++ reads it into an `int`; the two were only
compared on budgets that fit.

Not tested: inputs at the full limits against the originals, and `ioi12p4`
on cities outside the generator's family (W5).
