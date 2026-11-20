# Friction backlog, batch 5: flow, matching, greedy, binary search, geometry, interactive

A Stage-1 scout pass over ten submissions: noi06p4, cco23p6, noip10p3,
cco22p1, ioi13p5, btoi17p1, ioi06p2, cco18p4, yac6p3 and etsr23r3p2. The
sketches are in [`proto/ports/scout_batch_5/`](../proto/ports/scout_batch_5/)
and the forms that run today in its `today/` folder. This file follows
the format of [`friction-backlog.md`](friction-backlog.md) (batch 1). Its
items are numbered `K1`... so they do not collide with `F` (batch 1), `B`
(batches 2 and 4), `G` (batch 3) or `H` (batch 8); an item an earlier
batch already has is named with that batch's number.

What was checked: nine of the ten have a `today/` form that agrees with
its original (`proto/ports/scout_batch_5/check.sh`; the README there
says how each was checked and at what sizes). The work was done at
commit `f4f53ad`. `722c0ea` changed `proto/cpc` before this batch was
pushed, so every probe below was run again at `de5b419`: all give the
same result except K3, which is fixed, and `check.sh` passes there. Every
"today" snippet below is a line of a `today/` form, and every error
message quoted was produced by a small probe with `proto/cp.py build`. The "wanted" snippets are lines of the sketches and do not
build. A count such as "3 of 10" is the number of the ten sketches that
carry the item's marker. Corpus counts are from a regular expression over
the 710 accepted C++ problems in `corpus/submissions/`, so they are
estimates, and the pattern is named each time.

## Suggested order for a Stage-2 sprint

1. K1, K2, K4 and K5 are bugs: C++ that does not compile, output in the
   wrong order, a generator that writes a different format, and a table
   of 10^12 slots. (K3 was one and is fixed.)
2. K6: 3 of 10 sketches and 4 of 9 `today/` forms, and 51 corpus
   problems write the loop it replaces.
3. K8 and K9: 1 and 2 of 10; each is a few library functions.
4. K13, K14, K16: one library piece each; 1, 2 and 2 of 10.
5. K7, K10, K11, K12, K15, K18, K19, K21: 1 or 2 of 10 each.
6. K17 waits for decision 1.5 (floats); K20 for decision 6.8 (graders).

## K1. A map keyed by a pair builds, and the C++ does not (fixed in Stage-2 sprint #2; cco18p4)

*Checked.* This is accepted by `cp.py build`:

```cp
var seen: HashMap<(Int, Int), Int> = []
seen[(1, 2)] = 3
print(seen.get((1, 2)) ?? 0)
```

and g++ then stops with "static assertion failed: hash function must be
invocable with an argument of key type" (`CpHash` has no overload for a
tuple). A pair of domain values, `HashMap<(Row, Col), Int>`, fails the
same way. The cache in `today/cco18p4.cp` packs the pair by hand:

```cp
var seen: HashMap<Int, Int> = []
if let known = seen.get(r.label * (cols + 1) + c.label): return known
```

Proposed: hash any record of hashable fields. 1 of 10; in the corpus, 5
problems keep a map as a cache (pattern: `static map<`, or a
`map<pair<int, int>, ...>` named `cache` or `memo`).

*Fixed.* A record whose fields are numbers, positions, `Bool`, `Char` or such records hashes field by field, so `HashMap<(Int, Int), V>` and `HashSet<(Row, Col)>` build; a key that cannot be hashed is a compile error from `cp.py build`, not from `g++`. Locked by `proto/tests/redteam/pair_key_map.cp`.

## K2. `print` writes its first word before a later argument asks (fixed in Stage-2 sprint #2; cco18p4)

*Checked.* With `fn q(x)` that calls `ask("?", x)`:

```cp
print("!", q(n))
```

the solution sends `! ? 4` on one line: `print` has written "!" before
`q` runs and writes its question. `today/cco18p4.cp` avoids it with
`let best = cost(r, c)` on the line before. Proposed: `print` evaluates
every argument before it writes anything. This is the plain reading of
the line, and the local judge only reports a wrong conversation, with
nothing that points at the `print`. 1 of 10; any interactive problem
whose last answer may need one more question.

*Fixed.* `print` evaluates every argument, left to right, before it writes anything (decision 15.1). Locked by `proto/tests/redteam/print_args_first.cp`.

## K3. A bound that is read is not enforced (fixed in `722c0ea`; ioi06p2)

Kept as a record; batch 8 found the same bug as its H1. *Checked at
`f4f53ad`.* With

```cp
let w: 3..=1000 = read()
let bw: 3..=w = read()
let cw: 1..=bw - 2 = read()
```

the generator at size 6 writes `3 4 1`, `4 4 3` and `6 4 3` for seeds 1 to
3: `bw` above `w`, and `cw` above `bw - 2`. The emitted C++ shows why:
each bound is replaced by the widest value it could have,
`gen_int(3LL, 1000LL)` for `bw` and `gen_int(1LL, 998LL)` for `cw`, and
the checked read uses the same constants. So a program built with
`-DLOCAL` reads `6 4 3` and prints it without complaint, while a constant
bound is checked there ("input out of declared range").
`today/ioi06p2.cp` repeats every bound in a `where`:

```cp
let ch: 1..=bh - 2 = read() where bw <= w and bh <= h and cw <= bw - 2 and ch <= bh - 2
```

and 86 to 93 % of the generated inputs were then rejected. At `de5b419`
the generator writes `3 3 1`, `4 4 2` and `6 6 2` for the same seeds, a
`-DLOCAL` build stops on `6 4 3` with "input out of declared range",
and the `where` has been removed from `today/ioi06p2.cp`. Nothing left
to do.

## K4. A generated `[P]List<P>` is written without the lengths (fixed in Stage-2 sprint #2; btoi17p1)

*Checked.* `let lists: [P]List<P> = read()` reads a count and then that
many values for each `P`, and the default generator writes that format.
With `read(gen: random_pals)`, where `random_pals` returns a
`[P]List<P>`, the generator wrote `7 1` and then one flat line of 14
numbers with no counts, and the port stopped with "unexpected end of
input". So a symmetric relation could not be generated, and
`today/btoi17p1.cp` is checked by a script (`feed.py`). Proposed: a
`gen:` function's value is written by the same code as the default
generator's. 1 of 10.

*Fixed.* The value of a `gen:` function is written in the format `read()` reads for that type: a list after its length, a table row by row. Locked by `proto/tests/gen/gen_fn_lists.cp`, which `proto/run_all.sh` now generates from and reads back.

## K5. `@memo` over two large domains allocates the whole table (fixed in Stage-2 sprint #2; cco18p4)

*Checked.* On a 6311529 x 432463 grid this stops with `std::bad_alloc`:

```cp
@memo
fn cost(r: Row, c: Col) -> Int:
    let total: 0..=4e9 = ask("?", r, c)
    total
```

At most 170 of the cells are ever asked. Proposed: `@memo` uses a hash
table when the product of its domains is past a size limit (or when the
function asks, since the number of questions is bounded), and warns at
build time otherwise, as the loop-size warning already does. 1 of 10;
the same 5 corpus problems as K1.

*Fixed.* A `@memo` whose domains multiply to more than 2^26 points at their declared maximum sizes stores only the points asked for, as `@memo(hash)` does (decision 15.3); `into:` and `order:` on such a function are a compile error that gives the size. Locked by `proto/tests/redteam/memo_sparse.cp`.

## K6. Searching a range of numbers or positions (3 of 10; batch 1 F10)

noip10p3, ioi13p5 and cco22p1; the `today/` form of cco18p4 searches
the same way. *Checked:*
`(0..=1e9).first_where(...)` is "Range<Int> has no method
`first_where`", `S.last_where(...)` is "type S has no method
`last_where`", and `Row.first_where(row_turns)` with a named function is
"expected a one-parameter lambda `x => ...`". So today a search needs a
domain declared for it and a lambda that unwraps the label:

```cp
domain Anger = 0..=1e9
print(Anger.first_where(x => peaceful(x.label))!)
```

and "the furthest end from x" is the loop by hand, in `today/cco22p1.cp`:

```cp
var lo = x
var hi = Spot.last
while lo < hi:
    let mid = Spot((lo.label + hi.label + 1) / 2)
    if consistent(x, mid): lo = mid
    else: hi = (mid - 1)!
```

Wanted:

```cp
print((0..=1e9).first_where(peaceful)!)
let reach = [Spot] x => (x..).last_where(y => consistent(x, y))!
print((1..=t).first_where(done_in) ?? -1)
```

Proposed: `first_where` and `last_where` on every `Range`, of numbers or
of a domain, open-ended included, taking a function name as well as a
lambda. The `?? -1` line is what makes the optional result pay: ioi13p5's
original needs a separate test for "impossible". 51 corpus problems write
`while (l < r)` with a midpoint.

## K7. The minimum of a convex function (cco18p4)

The original is 30 lines of golden-section search. The port searches the
slope instead, and today that needs a helper per axis, because an `if`
cannot be a lambda's body (*checked*: "expected an expression (at
'let')") and the step off the end is an optional:

```cp
fn row_turns(r: Row) -> Bool:
    if let s = r + 1: return cost(s, Col.first) >= cost(r, Col.first)
    true
let r = Row.first_where(r => row_turns(r))!
```

Wanted: `let r = argmin_convex(Row, r => cost(r, Col.first))`.
`plan/20-stdlib.md` lists ternary search as `Unimodal<T>`; this is the
name it could have at the call. It should say in its doc how many times
it calls the function (2 per halving), since in an interactive problem
that is the budget. 1 of 10; the wider corpus was not counted.

## K8. A graph of some of the edges, and its 2-colouring (noip10p3; batch 1 F1)

Four of the ten would take their graph code from `std` and cannot from
`proto/cpc` (F1, below). Beyond F1, noip10p3 shows two gaps. First, the
adjacency lists of a filtered edge list are built by hand:

```cp
var foes: [Inmate]List<Inmate> = []
for (a, b, anger) in feuds:
    if anger > x:
        foes[a].push(b)
        foes[b].push(a)
```

`Graph.undirected` takes a whole `[E]` array, so wanted is
`Graph.undirected(f for f in feuds if f.anger > x)`: any sequence of
records with two vertex fields. Second, `stdlib/graph.cp` has `bfs`,
`topo`, `scc` and `components` but no 2-colouring, so the 12 lines of
`peaceful` in `today/noip10p3.cp` have nothing to call. Wanted:
`two_colour(adj) -> ?[V]Bool`, `none` when an odd cycle exists.

## K9. Heaps: biggest first, a look at the top, up to k at once (2 of 10; batch 2 B5)

ioi13p5 and btoi17p1. *Checked:* `MaxHeap<Int>` is "unknown type
MaxHeap", `h.peek()` is "MinHeap<Int> has no method `peek`", and the
size is `h.len()` with brackets while a list's is `xs.len` (`h.len` is
"MinHeap<Int> has no `.len`"). So `today/ioi13p5.cp` negates:

```cp
left.push(-by_weight[next].size)
...
for _ in 0..minutes:
    if let s = left.pop():
        if -s >= limit: return false
left.len() == 0
```

Wanted:

```cp
var left = MaxHeap<Int>()
left.pop(minutes)                         // up to that many, as a list
if any(size >= limit for size in left.pop(minutes)): return false
left.is_empty
```

Proposed: `MaxHeap`, `peek`, `pop(k)`, `push_all`, and `len` and
`is_empty` spelled as on a list. btoi17p1 wants one more thing, a heap
whose key changes: today it pushes `(deg[v], v)` again and skips stale
entries on the way out (`if gone[u] or d != deg[u]: continue`). A
`MinHeap.by(P, key)` with `update(p)` would hide that. 11 corpus problems
use a default (max) `priority_queue`.

## K10. Sorting downwards (ioi13p5)

*Checked.* `sorted(small).rev` is "[0..b]Int[1..=2000000000] has no
`.rev`", although `for x in ys.rev` works when `ys` is a name, and
`sorted(a, desc)` is answered with the accepted forms: "sorted(xs),
sorted(xs, by: x => key) or sorted(xs, less: (a, b) => a_first)". Today
it is a name for the sorted list, or a negated key
(`sorted(a, by: x => -x)` builds):

```cp
let small_up = sorted(small)
for limit in small_up.rev:
```

Wanted: `for limit in sorted(small, desc):`, and `.rev` on any
sequence-valued expression. 1 of 10.

## K11. Vertices of several kinds (noi06p4)

A flow network has a source, a sink, and one vertex per station and per
user. Today they share one numbered domain and the offsets are kept by
hand:

```cp
domain Node = 0..n + m + 2
let source = Node(n + m)
let sink = Node(n + m + 1)
fn station(s: Station) -> Node: Node(s.label - 1)
fn user(u: User) -> Node: Node(n + u.label - 1)
```

*Checked:* an enum with payloads cannot index an array ("array dimension
must be a domain, a range, or a struct with `impl Index`, got Node"), and
`domain Node = A | B` is "domain needs a range". Wanted:

```cp
enum Node:
    Source
    Sink
    Build(s: Station)
    Serve(u: User)
var net = FlowGraph<Node>()
net.add(Serve(u), Build(users[u].a), inf)
```

Proposed: an enum whose payloads are domains gets `impl Index`
automatically, laid out variant after variant. This is the sum next to
batch 3's G18 (a record of domains as an index, the product), and the two
together cover "a node is this or that" and "a node is this and that".
1 of 10; 5 corpus problems build a flow network (pattern: `sink`), and
both that were read for this batch (noi06p4, dmopc20c7p5) number their
vertices by offset.

## K12. Project selection as one call (noi06p4)

With K11 the network is still 6 lines of arcs and a subtraction that has
to be remembered (`total pay - max flow`). Wanted:

```cp
fn gain(i: Item) -> Int: ...          // + for a user's pay, - for a station's cost
fn needs(i: Item) -> yields Item: ... // a served user needs both stations
print(max_closure(gain, needs).value)
```

Proposed: `std.flow.max_closure(gain, needs) -> (value: Int, chosen:
[Item]Bool)` on top of `FlowGraph.min_cut`. 1 of 10; dmopc20c7p5 in the
corpus builds the same network.

## K13. A monoid that is a record, and tables derived from the input (cco23p6; batch 1 F7)

*Checked.* The built-in tree takes only a lambda: `SegTree<Len,
Run>(Run(0, 0, 0), join)` is rejected with "SegTree<D, T>(identity, (a,
b) => combined)", there is no `.all` ("SegTree<I, Int> has no `.all`"),
and `query` takes two ends, not a range. Today:

```cp
var runs = SegTree<Len, Run>(Run(0, 0, 0), (x, y) => join(x, y))
let made = runs.query(Len.first, Len.last).made
```

`stdlib/segtree.cp` already has `fold(r: Range<D>)` and `.all`, so this
part is F1. What is new is the bookkeeping around the tree. A leaf
depends on `have[i]` and on a second table `ending`, which itself
depends on `have` at another index, and the port keeps all three in step
by hand:

```cp
fn count(i: Len, sign: Int):
    pairs += sign * (have[i] / 2)
    ending[reach(i)] += sign * (have[i] / 2)
    refresh(i)
    refresh(reach(i))
for (i, by) in changes:
    count(i, -1)
    have[i] += by
    count(i, 1)
```

The original has the same shape (`upd(ind, -1); arr[ind] += x; upd(ind,
+1)`), and it is where a mistake would go. Wanted: the monoid declared
with its record (`op join` and `const empty` inside `struct Run`), and

```cp
let runs = SegTree.of(Len, i => Run(0, have[i] % 2, 0).join(Run(0, 0, filed[i])))
have[i] += by         // runs refreshes the slots that read have[i]
```

Proposed, in two steps. The small one: `SegTree.of(D, leaf)` plus
`refresh(i, ...)`, as batch 1 F7 asks. The larger one, to decide and not
to build yet: a table declared as a function of other tables is kept up
to date by the compiler when an index it read is assigned. cco23p6 is a
good test, because one write to `have[i]` touches two leaves. 1 of 10,
and ccc21s4 in batch 1.

## K14. Windows of a grid and their sliding minimum (2 of 10; batch 1 F6)

ioi06p2, and cco22p1 for neighbouring pairs. *Checked:* `R.windows(2)` is
"type R has no method `windows`", `Deque` and `Queue` are unknown types,
and `sliding_min` is an unknown name. `today/ioi06p2.cp` gives up typed
positions altogether: a block is four plain numbers, converted at the one
place they meet the prefix sums,

```cp
fn block(r: Int, c: Int, rows: Int, cols: Int) -> Int:
    pre.sum(Row(r + 1)..=Row(r + rows), Col(c + 1)..=Col(c + cols))
```

the sliding minimum is 9 lines with a list used as a deque (`keep`,
`head`), run over rows and then over columns of plain nested lists, and
the answer is read back with the indices swapped and shifted
(`down[c + 1][r + 1]`). Every one of those `+ 1` is a chance to be off
by one, which is what domains are for. Wanted:

```cp
let chambers = [Block] (rows, cols) in windows(Row, ch) * windows(Col, cw) => pre.sum(rows, cols)
let lowest = chambers.sliding(min, rows: bh - ch - 1, cols: bw - cw - 1)
```

Proposed: `windows(D, k)` as an index type whose values are ranges
(F6), so a table of "every placing of a k-wide block" is indexed by the
block; `.inner` on a range (both ends moved in by one); `(x..=y).pairs`
for neighbours; and `sliding(op, k)` on a 1-D or 2-D table, by monotone
deque for `min` and `max`. 12 corpus problems use a `deque`; what they
use it for was not looked at.

## K15. A maximum over ranges is optional, over domains it is not (ioi06p2)

*Checked.* `let (v, i) = max((a[i], i) for i in I)` over a domain builds.
The same over plain ranges,

```cp
let (_, r, c) = max((block(r, c, bh, bw) - down[c + 1][r + 1], r, c) for r in 0..=h - bh for c in 0..=w - bw)
```

is "cannot destructure ?(f0: Int, f1: Int, f2: Int) into 3 names" and
needs a `!` at the end. The rule is sound (a range can be empty, a domain
cannot), but the message does not say so. Proposed: the error names the
reason and the fix ("this range may be empty; add `!` or `else`"). With
K14 the ranges become windows of a domain and the question goes away.
1 of 10.

## K16. A number that may or may not be a position (2 of 10)

yac6p3 and cco23p6, and `block` in the `today/` form of ioi06p2.
*Checked:* `X(v)` traps at run time when `v`
is outside ("domain value out of range"), and `X.of(v)` and `X.clamp(v)`
are "type X has no method". So a computed coordinate is tested by hand
before it is converted, in `today/yac6p3.cp`:

```cp
if (a + b) % 2 == 0 and 1 <= x <= n and 1 <= y <= m:
    if near(X(x), Y(y)) == 0: lower_pair = true
```

and cco23p6 clamps with `Len(min(n, 2 * i.label - 1))`. Wanted:

```cp
fn cell(sum: Int, diff: Int) -> ?Cell:
    if (sum + diff) % 2 != 0: return none
    (X.of((sum + diff) / 2)?, Y.of((sum - diff) / 2)?)
fn reach(i: Len) -> Len = Len.clamp(2 * i.label - 1)
```

Proposed: `D.of(n) -> ?D` and `D.clamp(n) -> D` on every domain; with
`?` (F8) the bounds test disappears. This is the same idea as the typed
step `i + k`, for a value that was computed and not stepped to. 12 corpus
problems define an in-bounds lambda (pattern: `auto ibs = [&](int x, int
y)` and similar names).

## K17. Points with real coordinates, and a mirror image (etsr23r3p2; batch 1 F15, batch 3 G15)

*Checked:* `F64` is "unknown type F64", so there is no `today/` form. The
original uses `complex<double>` and reflects with `c1 + v1 * (v1 *
conj(v2)) / norm(v1)`, which nobody reads at sight. Wanted:

```cp
let folds: [Fold](a: Pt<F64>, b: Pt<F64>) = read()
if cross(b - a, p - a) < 0: return 0
layers(done.prev, p) + layers(done.prev, reflect(p, a, b))
```

Proposed, once floats are decided: `Pt<T>` generic over the coordinate
(`stdlib/geometry.cp` has `Pt` over `Int` only), and `reflect(p, a, b)`,
`project(p, a, b)` and `dist` next to `cross` and `dot`. Exact rationals
do not rescue this problem: ten folds multiply ten denominators. 1 of 10;
46 corpus problems mention `double` or `long double`.

## K18. Every subset of a small list (btoi17p1; batch 4 B6)

*Checked:* `subsets` is an unknown name. Today the clique test is bit
arithmetic over list positions:

```cp
for mask in 0..(1 << near.len):
    let group = [near[i] for i in 0..near.len if ((mask >> i) & 1) == 1]
    if all((a, b) in pals for a in group for b in group if a != b): best max= group.len + 1
```

Wanted: `for group in near.subsets` and `group.pairs`. This is B6 of
batch 4 for a list rather than a domain; one `subsets` over any small
collection covers both. 1 of 10.

## K19. Inputs with a relation between fields (cco22p1, and btoi17p1 through K4)

cco22p1 reads queries `x < y`; today that is a `where` over the whole
array, and 83 to 88 % of generated inputs are thrown away:

```cp
let asks: [q](x: Spot, y: Spot) = read() where all(t.x < t.y for t in asks)
```

*Checked:* a `where` inside the record, `(x: S, y: S where x < y)`, does
not parse ("expected an expression (at 'where')"). Wanted: `let asks: [q]Span<Spot> = read()`, an ordered pair that
is read as two numbers and generated ordered. btoi17p1 needs a symmetric
relation and has no way to say so (and K4 blocks the `gen:` route).
Proposed: `Span<D>` (two positions, first not after second; a strict
variant for `<`), and `Symmetric<[P]List<P>>` or a generator helper for
undirected adjacency lists. `proto/ports/ccc21s5.cp` has the same
`where` for `l <= r`.

## K20. Grader-style originals (ioi13p5; batch 1 F20)

The original is `int putaway(int A, int B, int T, int X[], ...)` with no
`main`; `check.sh` appends one. Nothing new beyond F20: decision 6.8 is
still open.

## K21. The question budget next to the question (2 of 10)

cco18p4 and yac6p3. The limits (170 and 5) are in the judges only; the
original asserts `queries <= 170` by hand. Wanted: `ask("?", r, c, limit:
170)`, which traps locally on the question after the last allowed one
and gives `cpl stress` a number to report. This is the solution-side half
of the co-located `interactor` block: until that exists, the judge is a
Python class next to the port (`today/*_judge.py`, as in batch 3). 2 of
10; 8 interactive corpus problems.

## Smaller notes

- *Checked:* an array sized by a literal, `let a: [5]Int = read()`, is
  "expected a type", while `[m]Int` with `m` read from the input builds.
  No port needed it; three probes did.
- A set's membership test is `x in s`; `s.has(x)` is "no method has on
  Set<P>". Fine once known, but the message could name `in`.

## Seen again from earlier lists

- **F1** (ports cannot `use std`): 5 of 10 (noi06p4, cco23p6, noip10p3,
  cco22p1, etsr23r3p2). noi06p4's `today/` form runs only on the
  interpreter for this reason.
- **F5** (`let ... else`): etsr23r3p2, for the fold left of a boundary.
- **F15** (floats): etsr23r3p2, under K17. **F20** (graders): ioi13p5.
- **G21** (`zip`, `cycle`): cco22p1 alternates "up, down" with a flag
  flipped by hand.
- **G5** (a set straight from a list or a domain): btoi17p1.
- **G4** (a record prints its fields): yac6p3 prints two cells.
- **B6** (set algebra): btoi17p1, `foes[u] & alive`.

## Already good

- `@memo` in front of an asking function works: with it in place of the
  hand-kept map, `today/cco18p4.cp` passed 100 conversations of size 6
  and 30 of size 2000. K5 is only about size.
- `ask("?", x, y)` with a typed reply, and the Python judge protocol of
  `proto/interact.py`: two interactive ports were checked in minutes.
- `prefix(height)` with `pre.sum(rows, cols)` on a 2-D table.
- `max((value, r, c) for ...)` as argmax with the original's tie-break.
- `[P]List<P> = read()` reads count-prefixed lists in one line.
- `(a, b) in pals` on a set of pairs.
- `MinHeap<(Int, P)>` with `while let (d, u) = heap.pop()`.
- `std.flow.FlowGraph<Node>` on the interpreter: `add`, `max_flow`.
- `first_where` returning an optional: `?? -1` replaced a special case.

## Solutions that changed shape

See "Shape changes" in the batch README: cco18p4 (slope search instead of
golden section), cco22p1 (Kahn instead of recursive colouring), ioi13p5
(`none` from the search instead of a pre-test) and noi06p4's sketch
(closure instead of an explicit network).
