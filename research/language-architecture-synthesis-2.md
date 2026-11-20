# Language Architect Synthesis #2: what the three zero-tolerance friction audits add up to

*CP Lang 0.2.2. Written against `main` at `1634433`. Inputs: `research/ugly-code-friction-audit-1.md` (101 ports: I/O, formatting, casts, ad hoc), `research/ugly-code-friction-audit-2.md` (193 ports: DP, graphs, trees, data structures) and `research/ugly-code-friction-audit-3.md` (208 ports: geometry, math, strings, simulation, interactive), whose counts are `grep` totals over all ~1,000 `.cp` files under `proto/ports/`. This is a design document: nothing under `crates/`, `stdlib/`, `proto/` or `docs/` was changed, and every "after" block is a proposed spelling that has not been run in the prototype yet.*

## The short version

The three audits list 58 patterns between them. Deduplicated they are about 30, and they sort into four piles of very different cost:

1. **About half of the ugly lines are not a design problem at all.** 0.2.2 already has the short form (`Int(b)`, `xs.last`, `bsearch_first`, `compress`, typed `read()` of records, `MaxHeap`, `desc(x)`, tuple indices, `@multitest`, `Deque`), or `examples/local-lib/` already has the structure (`Cover`, `LiChao`, `Rope`, `Frac`, `Fixed`, `fold_others`, `kruskal_tree`, `functional`, `centroids`), and the port either predates it or was written for a prototype that could not run it. The fix is toolchain parity, a migration pass and lints.
2. **Seven kernel rules** remove whole classes of the remaining friction. Six of them extend a rule that already exists to the places it stops short; one is a deletion. Together they add no keyword, no decorator and no operator.
3. **One userland wave** of small `stdlib/` additions, each a few lines, each reusing a verb the cheat sheet already has.
4. **The remaining requests are rejected**, most of them because they revive something `decisions.md` §19 deleted on corpus evidence, or because a kernel rule above already covers the use.

Where the audits disagree with each other or with a locked decision, the call is made in the section "Collisions" below, before the ranked list, because several accepted changes depend on those calls.

## Collisions between the audits, and with locked decisions

**`min=` as a condition versus `.chmin()`.** Audit 1 (entry 11) and audit 3 (pattern 15) ask for `x min= y` to be usable as a `Bool`; audit 2 (pattern 3) offers `x.chmin(y)` as an alternative. Take the operator. It is the spelling 0.2.2 already teaches, and a method would be a second way to write the same update.

**`?Int += dx`.** Audit 2 asks for `+=` to lift over `?T`. `decisions.md` 19.5 deleted exactly that ("silent no-ops hide uninitialized DP"), and binary `+` still absorbs `none`, so the one site (`ioi04p2`) is `on_x[v] = on_x[v] + dx`. Rejected. The same audit's claim that `min(a, b)` does not work on two `?Int` in expression position is a prototype gap, not a design gap: the cheat sheet pins `min(?T, ?T) -> ?T`. With that running, `opt_min` in `cco21p5` is `Monoid(none, min)`.

**`for x in mut xs` and `.absorb(mut other)`.** Both are asked for by audit 2 (patterns 13 and 17) and both were deleted in 19.4 after hitting 0 of 642 ports, with stated reasons (a second iterator borrow mode; a hidden O(|other|) trap and same-container aliasing). Three new sites do not overturn that. Rejected; `for i in 0..xs.len: xs[i].f()` and `swap` + `.take()` stay the one idiom.

**`product(it)`.** All three audits ask for it, citing two ports between them (`noip19p4`, `ioi22p4`). 19.7 deleted `prod(gen)` for the same reason. Rejected: `var all = Z(1)` and a one-line loop.

**Scoped `defer` / `undo:` blocks, `Pool.free`, `I32`.** Audit 2 patterns 11 and 17. `defer` was deleted in 19.1, `Pool`, `I32` and `Journaled` in §18. Rejected. A sequence treap is `Rope` in `examples/local-lib/ds/rope.cp`; range types already pack to 4 bytes.

**`yes:` / `no:` on `print`.** Audit 3 (pattern 3) wants it; audit 1 (entry 8) argues against it. Side with audit 1: `print(if ok: "YES" else: "NO")` is one unambiguous line, judges use a dozen capitalisations, and the named arguments would be a concept that saves six tokens. What is accepted from audit 3 is the reading half: `Bool` reads `0`/`1`.

**`Char + Int` versus `Char.of_digit(d)`.** Audit 1 (entry 9) proposes the operator, audit 3 (pattern 2) a pair of methods. Take the operator: it is the inverse of `Char - Char -> Int`, which exists, and it covers letters as well as digits. `Char.digit(base)` for hex has one port behind it and is rejected.

**How much of `printf`.** Audit 1 (entry 1) wants width and zero-fill only and says "no hex"; audit 3 (pattern 2) adds `b`, `x`, `X` and a nested `{n}` width. One port prints binary and one parses hex. Take audit 1's three forms and stop.

**`Fixed<D>.ratio(...)` versus `fixed(num, den, k)`.** Same job, two audits. Take the function: the callers want a string once, not a numeric type. `Fixed` stays in `examples/local-lib/math/`.

**`sign(k)` for inclusion-exclusion.** Audit 3 (pattern 11). The prelude already has `sign(x, eps)` for floats with a different meaning. Rejected; the `if popcount(sub) % 2 == 0` form stays.

**`Rev<T>`, `.reversed()`, `in_preorder`, `fold_edge_path`, `Subset.full` / `.first`, `Rooted.from_parents`.** Audit 2 proposes each as new. Each exists under another name: `desc(x)` / `Desc<T>`, `Graph.transpose`, `Rooted.reorder`, `fold_path(a, b, ds, edges: true)`, and the last three verbatim. These are documentation and migration items, not design.

**Sub-domain widening versus reading into a slice.** Audit 2 (pattern 1) wants a core rule so that `domain Kid = V.tail` coerces to `V`, to kill `V(k.label)` in parent-array trees. The same friction disappears with no new rule once `read()` takes its type from any typed place (kernel rule 3 below): the parent array is `[V]?V`, its tail is read in place, and `Rooted.from_parents` already takes that shape. Widening is rejected.

**Flow-typed `!` removal.** Audit 2 (pattern 2) wants `m + 1` to have type `I` when `m` comes from `l..r`, and `xs.last` to be non-optional under `if not xs.is_empty`. That makes the *type* of an expression depend on a proof, so an innocent edit to the loop bound changes whether the next line compiles. This is the one request where the friction is real (postfix `!` is in 37% of ports) and the proposed cure is a large concept. It is not accepted into the kernel in this wave; the three most common sites are handled in userland (rule U6), and the rest goes to a prototype bakeoff with the rule stated as narrowly as possible.

## Accepted changes, ranked by contest impact against complexity

Rank 1 is the best ratio. "Kernel" means a rule in the checker or grammar; "userland" means `stdlib/` code only; "parity" means the design is already locked and a tool does not do it.

### 1. Parity first: decided features that do not run (parity, zero concepts)

Audit 2 pattern 18 and audit 3 pattern 20 measure the same thing from two sides: runnable ports are 47% longer than their sketches (1,979 lines against 1,344 over 47 pairs), and one clean 90-line draft became 210 lines of C-style code. Three of the gaps produce silent wrong answers and come before everything else:

- `proto/cpi` drops `{x:.3f}` and prints the bare float.
- A bare `print(x)` of an `F64` is accepted although the cheat sheet makes it an error.
- `proto/cpc` emits `1 << 32` as a 32-bit shift.

Then the features whose absence forces the longest workarounds: `(m[k] ??= []).push(x)`, nested `@memo`, multi-dimensional `[A, ?B, C]?Int` tables, `while let (a, b) = ...`, `Mod + Int` and a runtime modulus (which lets `ModR` and 216 `Z(...)` wrappers go), `U64` / `I128` with `Read` and `Show`, `Str * Int`, `Pt` deriving `Read`/`Show`/`Ord`/`Hash`, `adj4` / `step` / `Dir4`, tuple-spread indexing `dist[a, b]`, and the prelude wiring for `Fenwick`, `LazySegTree`, `SortedBag` and `Deque`.

```cp
// before (blind_critic_5/ioi19p5.cp)
if not mp.contains(key):
    var empty: List<Int> = []
    mp[key] = empty
mp[key].push(r * cols + c)

// after: already 0.2.2
(mp[key] ??= []).push(r * cols + c)
```

One process change goes with it: stop accepting a `today/` port that works around a decided feature. The workaround becomes a failing prototype test instead, so the gap is paid for once.

### 2. `min=` and `max=` are `Bool` expressions (kernel, extends an operator)

Raised by all three audits. `x == none or cand < x!` is on 49 to 59 lines in 38 to 45 files, and it is the reason hand-written Dijkstra, Floyd-Warshall with path counts and every "best plus companion state" DP spell out the comparison.

```cp
// before (scout_batch_27/noi07p1.cp)
if dist[s, t] == none or a + b < dist[s, t]!:
    dist[s, t] = a + b
    ways[s, t] = ways[s, k] * ways[k, t]
elif a + b == dist[s, t]!:
    ways[s, t] += ways[s, k] * ways[k, t]

// after
if dist[s, t] min= a + b: ways[s, t] = ways[s, k] * ways[k, t]
elif dist[s, t] == a + b: ways[s, t] += ways[s, k] * ways[k, t]
```

```cp
if dist[q] min= nd: pq.push((nd, q))     // the relaxation step of every shortest path
best min= (gap: g, id: id)               // records and tuples compare lexicographically
```

The rule: `x min= y` and `x max= y` have type `Bool`, true when the place changed. Used as a statement the value is discarded, as today. The 19.5 guardrail (a `none` that means failure must be checked explicitly) is unchanged. This also removes the need for `ArgMax` / `ArgMin` monoids: an arg-max segment tree is `SegTree<I, ?(Int, I)>(Monoid(none, max), ...)`.

### 3. `read()` and `ask()` take their type from any typed position (kernel, removes a special case)

Audit 3 pattern 4, and the root of audit 1 entry 4. Today the only legal form is `let x: T = read()`, which is why 13 files wrap `ask` in a three-line function, 12 define `ri()`, and three spellings of an explicit type are in use.

```cp
// before (yac6p3.cp, scout_batch_21/noi07p2.cp)
fn nearest(x: Int, y: Int) -> Int:
    let d: 0..=2e9 = ask("?", x, y)
    d

for d in Day:
    let (pa, pb, r): (F64, F64, F64) = read()
    (a[d], b[d], rate[d]) = (pa, pb, r)

// after
fn nearest(x: Int, y: Int) -> 0..=2e9 = ask("?", x, y)
for d in Day: (a[d], b[d], rate[d]) = read()
```

The rule is ordinary expected-type inference: a `let` annotation, a return type, a typed place on the left of `=`, or a parameter. Where there is no context the one explicit spelling is `read::<T>()`, which `decisions.md` already fixed for return-polymorphic calls; `read<Int>()` is deleted.

This rule is also what resolves the parent-array collision without sub-domain widening:

```cp
// before (hillclimb_dp_graph/noi15p2.cp)
let parent: [Dependent]Pkg = read()
var needed_by: [Pkg]List<Pkg> = []
for d in Dependent: needed_by[parent[d]].push(Pkg(d.label))

// after: no second domain, no conversion
var parent: [Pkg]?Pkg = none
parent[Pkg.tail] = read()
let tree = Rooted.from_parents(parent)
```

### 4. Two index conversions, each with one name; six spellings deleted (kernel, a subtraction)

Audit 1 entry 2, audit 2 patterns 1 and 8, audit 3 patterns 5 to 7: nine spellings and about 900 call sites, in 30% of ports. Some mean the 0-based position and some the written label, and the reader cannot tell which.

Keep exactly these:

```cp
V(x)             // from the written label, checked: V(1) is the first element of 1..=n
Int(v)           // the written label back; it is what print(v) and read() already use
v.to_idx         // the 0-based position, for row-major arithmetic only
V.from_idx(i)    // -> ?V, its inverse
```

Delete `.idx`, `.ord`, `.label`, `D.nth(i)`, `D.at(i)` and `D.try(i)`. `Int(v)` joins `Int(ch)` and `Int(b)`, so it is the existing cast rule applied to one more type, not a new one.

```cp
// before (scout_batch_52/noi02p2.cp, scout_batch_30/noi07p6.cp)
let ans: Int = ask("Ask", i.to_idx + 1, j.to_idx + 1, 0)
pq.push((-tree.depth(u), -u.idx))
let u = Node(-top.1 + 1)

// after
let ans: Int = ask("Ask", i, j, 0)             // a domain value crosses the wire as its label
var pq = MaxHeap<(Int, Desc<Node>)>()
while let (_, d) = pq.pop():
    let u = d.x
```

The same rule extends to the one boundary where it does not hold yet: `@grader fn` parameters and results of domain type marshal by label, exactly as `read` and `print` do (audit 3 pattern 6). `MakeMove(v.to_idx + 1)` becomes `MakeMove(v)`.

Two lints carry the rest. First, *a `domain` whose values are converted to `Int` in three or more places should be a structural range*: `var nxt: [0..=n, 0..=n]Z` makes `nxt[I(i + 1), I(j + 1)]` plain `nxt[i + 1, j + 1]`. Second, `D2(x.to_idx + 1)` where `D2` is `D.bounds` should say "write `x.r`"; the boundary vocabulary (`v.l`, `v.r`, `v.l(k)`, `b - a`, `(l..=r).len`) exists and most ports did not find it.

### 5. An `Int` next to an `F64` widens (kernel, relaxes one rule)

Audit 1 entry 7 and audit 3 pattern 10. Float formulas over integer input carry one cast per variable, in the problems where contestants are already slowest.

```cp
// before
total += F64(ways[s, v] * ways[v, t]) / F64(ways[s, t])
fn through(j: Int, i: Int) -> F64 = best[j] + F64(line(j, i)) ** F64(p)

// after: one cast marks the expression as floating
total += ways[s, v] * ways[v, t] / F64(ways[s, t])
fn through(j: Int, i: Int) -> F64 = best[j] + F64(line(j, i)) ** p
```

The rule: in `+ - * / **`, when one operand is `F64` an `Int` operand widens, as an integer literal already does. The two rules that protect against wrong answers stay: `Int / Int` is always integer division, and an `F64` never becomes an `Int` without `floor` / `round` / `trunc`. `Float` is deleted as a name for the type (19 files use it against 13 for `F64`).

### 6. Width and zero-fill in the format specifier (kernel, three forms)

Audit 1 entry 1 and audit 3 patterns 1 and 2. Eight hand-written zero-padders, a ten-arm `digit_char`, and eight spellings of "print k decimals".

```cp
// before (scout_batch_10/ctsc06p1.cp, scout_batch_29/wide.cp)
fn digit(x: Int) -> Str = "0123456789"[x % 10..x % 10 + 1]
print(digit(w / 1000) + digit(w / 100) + digit(w / 10) + digit(w))

var digits = "{lo}"
while digits.len < 18: digits = "0" + digits
"{hi}" + digits

// after
print("{w:04}")
"{hi}{lo:018}"
```

The whole grammar is `{v:W}` (minimum width), `{v:0W}` (zero-filled) and the existing `{v:.Df}`. No alignment, no hex or binary, no nested widths. `print(x, digits:)`, `print(x, prec:)` and `write` are deleted from `plan/16-io.md`.

For an exact fraction, one userland function in `std.math` replaces the sign-and-round logic three ports wrote by hand:

```cp
// before (scout_batch_29/sdoi17r1p6.cp)
fn slope(num: Int, den: Int) -> Str:
    let cut = abs(num) * 1_000_000 / den
    let units = (cut + 5) / 10
    var digits = "{units % 100_000}"
    while digits.len < 5: digits = "0" + digits
    (if num < 0 and units > 0: "-" else: "") + "{units / 100_000}." + digits

// after
print(fixed(num, den, 5))        // round half up, never "-0.00000"; fixed(t, 2, 1) for half-integers
```

### 7. Three one-line kernel completions

Each closes a hole in a rule that is already on the cheat sheet.

**`Char + Int -> Char`**, checked to stay in ASCII, the inverse of `Char - Char`:

```cp
// before
for row in rows: print(Str([Char(Int('0') + digit[Letter(c)]!) for c in row]))
// after
for row in rows: print(Str('0' + digit[Letter(c)]! for c in row))
```

**A list literal initialises `[D]T`** as it already does `[0..3]T`, with the length checked against `D.K`. This also closes an interpreter trap where the same line fills every cell with the whole list:

```cp
// before
let primes = [Small] p => [2, 3, 5][p.ord]
// after
let primes: [Small]Int = [2, 3, 5]
```

**`x!` on a `var` of type `?T` is a place**, so a stateful grader mutates in place instead of copying its state out and back:

```cp
// before (scout_batch_52/ioi13p3.cp)
var w = state!
w.h[p][q] = cost
state = w
// after
state!.h[p][q] = cost
```

### 8. Userland wave A: I/O shapes (userland)

**`print` rules written down, plus `sep:`.** 57 files print a count and then a list in five different ways, because nobody trusted what `print(xs)` does. State once in `plan/16-io.md` and on the cheat sheet: tuples, records and lists flatten to space-separated fields on one line; a 2D array prints one row per line; a `Char` grid prints rows with no spaces. Add one named argument next to `none:`.

```cp
// before
print(k)
for v in V: print(colour[v]!)
print("!", p.x, p.y, q.x, q.y)
for r in Row: print(out.row(r))

// after
print(k); print(colour, sep: "\n")
print("!", p, q)
print(out)
```

**`@tags` takes integers and strings, and a one-character-tag enum reads from an unbroken string.** 44 files read a kind tag and branch with `if`; the character case is already on the cheat sheet and nobody knew numbers worked.

```cp
// before (scout_batch_39/dmpg18s5.cp)
let kind: 1..=2 = read()
if kind == 1:
    let (l, r, x): (1..=n, 1..=n, 1..=MAXV) = read()
    let (li, ri) = (l - 1, r - 1)

// after
@tags(1, 2)
enum Query: Count(l: I, r: I, x: 1..=MAXV) | Set(i: I, x: 1..=MAXV)
let queries: [0..q]Query = read()
```

The same enum, shared by both sides, is what makes an `@interactor` a `match` instead of a 26-line `recv` chain; the only addition is `sol.recv_all::<Msg>()`.

**`Bool` reads `0` / `1`.** `let lit: [V]Bool = read()` replaces `[V] _ => read<Int>() == 1`.

### 9. Userland wave B: sequences, bits and grids (userland)

Each item reuses a verb that exists elsewhere on the cheat sheet.

- **`.get(i) -> ?T` on lists and arrays**, as on `Map`; an out-of-range `Int` or a `none` index gives `none`. `carry + (if i < a.len: a[i] else: 0)` becomes `carry + (a.get(i) ?? 0)`.
- **`xs.adjacent(cyclic: true)`** for polygons and tours: `for (a, b) in tour.adjacent(cyclic: true)` replaces `tour[(i + 1) % tour.len]` in 14 files.
- **`x.bit(i) -> Bool`** beside `popcount`; three spellings on 35 lines, each leaning on precedence. `if y >> i & 1 == 1` becomes `if y.bit(i)`. `<=` on `Subset<D>` as subset-of completes the set operators.
- **`scan` accepts a 2D array**, so a difference table becomes its running sum without three guarded terms: `let tint = scan(diff, +)`.
- **`xs.rank(x)` on a sorted sequence**, the verb `Set` already has, replaces the six-line "count below p" binary search.
- **`v.back(k)` and `v.ahead(k)`**, clamped steps, and **`D.tail` / `D.init`** as ranges: `val_st.fold(p.back(k - 1)..p)` replaces `max(0, pi - k + 1)` plus `I.nth(lo)`, and `for pen in Pen.tail` replaces `Pen.first.next!..`.
- **`p.dist1(q)`** in `std.grid` for Manhattan distance between cells.
- **`Subset<D>(it)`**, the constructor rule every other container follows, replaces `Cells.empty.with(Cell(0)).with(Cell(1))`.
- **`Window<D>` has non-optional `.l` / `.r` / `.first` / `.last`**, since `windows(k)` with `k >= 1` is non-empty by construction (see rule U6 below).

### 10. Userland wave C: graphs, trees and range structures (userland)

**Graph construction without a throwaway edge domain.** A hand-built `var adj: [V]List<...>` is in 162 ports, twice as many as use `Graph`, because `Graph.undirected` insists on `[E](u: V, v: V)` and iteration yields `(to, e)`.

```cp
// before (btoi17p3.cp)
var adj: [City]List<City> = []
for (u, v) in tracks:
    adj[u].push(v)
    adj[v].push(u)
let tree = Rooted(adj, City.first)

// after
let tree = Rooted.from_edges(tracks, root: City.first)
let g = Graph.undirected(tracks)       // any list or array of (V, V): g[u] yields V
let wg = Graph.undirected(roads)       // of (V, V, W): wg[u] yields (V, W)
```

**`bfs01`.** `dijkstra` in `stdlib/graph.cp` is already generic over any `Index` state and takes several sources (`next`, `from: List<(V, W)>`), so layered and multi-source searches can call it today; audit 2's 35 manual heap loops are partly a documentation gap. What is truly missing is 0-1 BFS. Parent tracking stays in `examples/local-lib/graph/shortest.cp`, and with rule 2 a custom relaxation is one line anyway.

**`SegTree.update(i, f)`**, which `LazySegTree` already has, and a fill constructor matching `var a: [D]T = 0`:

```cp
// before
near.set(t.pos(p), near.get(t.pos(p)) + (if safe[u]: 1 else: -1))
var st = LazySegTree<Val, Int>(AddMin, [Val] _ => 0)
// after
near.update(t.pos(p), x => x + (if safe[u]: 1 else: -1))
var st = LazySegTree<Val>(AddMin, 0)
```

**`Rooted.reroot(id, merge, lift)`.** The forward-and-backward sibling sweep is 15 to 20 lines every time and a known off-by-one source; it is built on `fold_others`, which exists in `examples/local-lib/seq/`.

```cp
// before (hillclimb_dp_graph/dpv.cp): 18 lines of down / earlier / later sweeps
// after
let ans = tree.reroot(id: 1, merge: (a, b) => a * b % m, lift: (v, x) => (x + 1) % m)
```

**`Dsu.unite_into(parent, child)`**, a directed union for Kruskal reconstruction trees and next-alive skipping; six lines, and it retires hand-written `find` loops.

**`Compressed.span(lo, hi)` and `Compressed.segments`**, the elementary intervals of a compressed axis, which remove the three-hop `from_idx(cy.at(y).to_idx - 1)!` cascade and make `Cover` usable for area-of-union sweeps.

**Make `t.Pre` a first-class domain everywhere.** `Fenwick<t.Pre, Int>` and arrays must accept `t.pos(u)` directly; `Pre(tree.pos(u).idx)` and reads of `t.at` / `t.heavy` are leaks of private fields.

### 11. Local-lib additions and discoverability (userland, off the cheat sheet)

Audit 2 patterns 10, 11, 14 and 15 ask for `CoverTree`, `LiChao`, `Treap`, `JumpTable` and Kruskal trees in `stdlib/`. All five exist in `examples/local-lib/` (`ds/cover.cp`, `ds/li_chao.cp`, `ds/rope.cp`, `graph/functional.cp`, `graph/kruskal_tree.cp`), demoted there by 19.7 on usage counts. The ports hand-rolled `o * 2` trees because the authors did not know, not because the structures are missing. Three structures are genuinely absent and belong beside them: a segment tree over time for offline rollback (`ds/time_segtree.cp`), an interval map for assign-heavy problems (`ds/intervals.cp`), and `Top2`. An `AssignAdd` action goes in the same place. The cheap fix with the largest effect is an index of `examples/local-lib/` by problem shape in its README, and a lint that recognises `o * 2` / `o * 2 + 1` indexing and points at it.

### 12. Migration pass and lints (tooling, zero concepts)

The patterns below need no design. Each is a mechanical rewrite of `proto/ports/` plus a lint so the long form is not written again:

- `if c: 1 else: 0` to `Int(c)` (80 lines in 62 files).
- `xs[xs.len - 1]` to `xs.last!` (86 lines in 61 files).
- A `while lo < hi` loop with a `mid` to `bsearch_first` / `bsearch_last` (54 files).
- A `read()` followed by a `push` in a loop to a typed array `read()` (1,587 scalar reads in 707 files).
- A list with a `head` index to `Deque` (40 sites); `for _ in 0..tests` to `@multitest` (113 files).
- A negated sort or heap key to `by: desc` / `desc(x)` / `MaxHeap` (29 lines).
- Index packing by arithmetic (`2 * f.to_idx + Int(flag)`) to a tuple index `(F, Bool)`.
- `if name not in ids: ids[name] = ...` to `ids[name] ??= P(ids.len)`.
- Private copies of `compress`, `Sieve`, `gcd`, `starts_with`, `swap` and `next_permutation` to the prelude names.
- `mut fn` on nested helpers, which 0.2.2 does not require, removed.

## U6: where the `!` goes without flow typing

The three sites audit 2 quotes for forced `!` each have a userland answer, and they should be tried before any narrowing rule is prototyped:

```cp
// windows: Window<D> carries non-optional ends
for rw in R.windows(k).rev: down[rw.l] = ...

// monotone stack: the stack's own optional does the work, no double unwrap
let la = if let top = st_a.last: top.r else: I.first.l      // a cut, not (st_a.last! + 1)!

// interval DP: split a range into its two non-empty halves in the library
min(cost(a) + cost(b) for (a, b) in (l..=r).splits)! + pre.sum(l..=r)
```

`(l..=r).splits` yields each pair of adjacent non-empty sub-ranges, so `m.next!` never appears. The `!` after `min(...)` stays: an aggregate over a generator is optional by the uniform rule, and the unwrap is elided at run time when proven.

## Rejected, with the reason

Already covered above: `.chmin()`, `?T +=`, `for x in mut xs`, `.absorb`, `product`, `defer` / `undo:`, `Pool.free`, `I32`, `yes:` / `no:`, `Char.of_digit` / `Char.digit`, hex and binary format flags, `Fixed<D>.ratio`, `sign(k)`, sub-domain widening, and flow-typed `!` removal (deferred to a bakeoff, not refused).

The rest:

- **`[k: v, ..default]` array literal and `Rolling<D, T>`** (audit 2 pattern 5). `var ways: [C]Z = 0` then one assignment is two clear lines, and a rolling step is `swap(mut cur, mut nxt)` plus `nxt.fill(none)`. New literal syntax for a two-line saving fails the budget.
- **`[K]T` as an `Index` / `@memo` key** (audit 2 pattern 16). One port; a tuple works. v2.2 cut array-shaped indices and one site does not bring them back.
- **Optional chaining `xs.last?.next`** (audit 2 pattern 2). A new operator; `if let` says the same thing.
- **`@interactor(limit: K)`** (audit 3 pattern 14). 19.3 has just cut decorator parameters; a `const LIMIT` used in the `ask` wrapper states the budget once.
- **`read(base: 1)`** (audit 1 entry 4). A 1-based statement is `domain V = 1..=n`; the label rule in change 4 makes the subtraction unnecessary.
- **`ArgMax` / `ArgMin` / `AddMinCount` stock monoids** (audit 2 pattern 9). `max` over a tuple with rule 2 covers the first two; the third is one `Action` line.
- **`v.adj` on a 1D index, `swap_remove`, `quotient_blocks` in `std`, `Int.digits`, `next_pow2`, `bit_reverse`, `Int.from_bits`, `dist_inf`, `adj4_and_self`, `angle_deg`, `Box`, `reflect`, `cyc_to`, `Perm.swap` with inverse tracking.** One or two ports each. Those worth keeping go to `examples/local-lib/`; none earns a prelude name.
- **Bitwise `|` on `Bool` to force both calls** (audit 1 entry 20). Two statements say it better; no rule is added and the trick is not blessed.

## Concept budget ledger

Added to the kernel: nothing with a name. The seven rules are `min=` / `max=` as `Bool`, expected-type flow into `read` / `ask`, `Int(v)` and label marshalling at `@grader`, `Int` widening beside `F64`, two format-spec forms, `Char + Int`, and the two place/literal completions. Keywords stay at 34 and decorators at 11.

Removed: six index-conversion spellings (`.idx`, `.ord`, `.label`, `.nth`, `.at`, `.try`), `read<T>()`, the type name `Float`, the library type `ModR`, `print(x, digits:)`, `print(x, prec:)`, `write`, and the "only on the right of a typed `let`" restriction.

Added to the prelude and `std`: `fixed`, `print(sep:)`, `.get`, `adjacent(cyclic:)`, `.bit`, `.rank` on sequences, `.back` / `.ahead`, `D.tail` / `D.init`, `.splits`, `dist1`, `bfs01`, `SegTree.update`, `Rooted.from_edges`, `Rooted.reroot`, `Dsu.unite_into`, `Compressed.span` / `.segments`, `recv_all`. Seventeen names, most of which reuse a verb or argument shape already on the cheat sheet.

## Order of work

1. The three silent-wrong-answer bugs in change 1, today.
2. Changes 2 to 7 in the Python prototype, each with a red-team test, then one cheat-sheet edit and a `decisions.md` §20 entry. Changes 4 and 5 touch locked rows (the index-conversion vocabulary and the float rules of 1.5) and need those rows revised, not appended.
3. The rest of change 1 (parity), in the Rust front end, so that changes 8 to 10 can be written as real `stdlib/*.cp` and run.
4. Changes 8 to 11 as one `stdlib/` and `examples/local-lib/` pass.
5. Change 12: migrate `proto/ports/`, land the lints, and re-run the three audits' `grep` counts as the acceptance test. The targets are the numbers quoted here: index-conversion escapes from 30% of ports toward the row-major cases only, `x == none or` from 49 lines to none, and the sketch-to-runnable gap from 47% to under 10%.
6. A bakeoff for flow-typed `!` on the interval-DP and monotone-stack ports, only if U6 leaves the `!` rate above one port in five.
