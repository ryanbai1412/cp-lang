# Friction Hunter, wave 3

Thirty more solved ports, none of them in [wave 1](friction-hunter-wave-1.md)
or [wave 2](friction-hunter-wave-2.md). They cover strings (KMP, Z-function,
suffix array), trees (LCA, HLD, path folds), number theory (sieve, CRT),
greedy and constructive problems, and bitmask and subset DP, plus the
remaining hard `ecnerwala/` and `wave2/` ports. I read every line of each.
Each pattern was checked against `cpl` on `main` at `e5daeb7`, using the same
labels as before ([STALE PORT], [REAL GAP], [REJECTED]). Every finding was
deduplicated against waves 1 and 2 and `research/archive/frictions/friction-queue.md`.

This wave also gives the root cause of each of the five hard ports that
wave 2 had to swap out.

## The numbers

The 30 ports have 755 non-blank lines. They hold 56 frictions
covering 108 awkward lines: 13 real gaps (24 lines),
38 stale-port patterns and 5 rejected.

- Real gaps per problem: 13 / 30 = **0.43** (wave 1: 0.60; wave 2: 1.03 after the correction below).
- Awkward lines per problem: 108 / 30 = **3.60** (wave 1: 3.73; wave 2: 6.07).
- Frictions of any kind per problem: 56 / 30 = 1.87.
- New real gaps from the port reading: 5, in 5 clusters (W2 to W6). The five broken ports add 4 more new gaps (B1 to B4).
- Ports with no friction at all: 3 (`noip14p2`, `noip18p2`, `mccc5j4`).

Most friction in this wave is stale. Three habits account for most of it:

- Re-reading 1-based input as `Int` and writing `a[i - 1]` (the pattern is in 58 port files).
- Building adjacency lists by hand next to a `Graph`.
- Importing names that are already in the prelude (`use std.graph.{Graph}` and similar, in 30 files).

**Correction to wave 2.** `first(x for x in xs if p)` runs on HEAD, and it
is the clean form of `ucup5s05e` lines 24-28. Wave 2 filed that as a real
gap ("`find` in `cpl run`"). It is a stale port, so wave 2 has 31 real gaps
(1.03 per problem), not 32. `find` itself is still unknown to `cpl run`,
which only matters if both names are meant to exist.

## The 30 ports

- `ecnerwala/cf2245d2` (graph peeling): 5 frictions.
- `ecnerwala/cf2255a` (cyclic strings): 1 frictions.
- `ecnerwala/cf2255b` (runs, combinatorics): 1 frictions.
- `ecnerwala/cf2255c` (interactive grid, number theory): 1 frictions.
- `ecnerwala/cf2262a1` (difference arrays): 2 frictions.
- `ecnerwala/cf2262a2` (counting DP): 3 frictions.
- `ecnerwala/cf2268b` (bits, popcount): 1 frictions.
- `ecnerwala/ucup5s03a` (EGF, power series): 1 frictions.
- `ecnerwala/ucup5s05b` (geometry, reflections): 2 frictions.
- `ecnerwala/ucup5s06a` (power series, Catalan): 2 frictions.
- `wave2/ccc20s2` (BFS on values): 1 frictions.
- `wave2/ccc21s2` (parity counting): 1 frictions.
- `wave2/noip06p2` (grouped knapsack): 2 frictions.
- `wave2/noip14p2` (trees, counting): 0 frictions.
- `wave2/noip18p2` (coin DP): 0 frictions.
- `wave2/yac1p1` (greedy): 1 frictions.
- `scout_batch_10/btoi19p5` (KMP strings): 2 frictions.
- `scout_batch_10/noip20p2` (Z-function strings): 2 frictions.
- `scout_batch_10/ctsc06p1` (KMP, Mod): 4 frictions.
- `scout_batch_10/nccc10s3` (suffix array): 3 frictions.
- `btoi17p3` (LCA, virtual tree): 2 frictions.
- `scout_batch_26/dmopc20c7p4` (trees, path folds): 3 frictions.
- `scout_batch_2/noi15p2` (HLD, lazy segment tree): 2 frictions.
- `hillclimb_dp_graph/sac22cc3p4` (LCA, offline DSU): 1 frictions.
- `scout_batch_11/noip21p1` (sieve): 2 frictions.
- `scout_batch_11/noi18p4` (CRT, number theory): 4 frictions.
- `scout_batch_42/mccc5j4` (greedy strings): 0 frictions.
- `btoi18p6` (bitmask DP on graphs): 1 frictions.
- `sim3/p4_abc480_f` (SOS / subset DP): 4 frictions.
- `scout_batch_28/dwite10c3p3` (broken-profile bitmask DP): 2 frictions.

## Root causes of the five broken ports

I fixed each one in a scratch copy, one error at a time, until `cpl check`
passed. The repository was not touched.

**`yac6p3`: stale port.** Its only error is `let m = read() where n * m >= 2`.
`where` guards were removed in 0.2.2. With `assert n * m >= 2` on the next
line it checks clean. It is listed in `PENDING` in
`crates/cpl-cli/tests/corpus.rs`, so the fix also lets it come off that list.

**`scout_batch_18/dpw`: stale port, three removed APIs.**
- Two `where` guards: one on a record read, `read() where all(x.l <= x.r ..)`, and one on a `@brute` function, `fn solve_brute() -> Int where n <= 16:`.
- `MaxOpt` is gone from `std.lazy`.
- The two-argument `LazySegTree.new(MaxOpt, AddMax)` became the one-argument form.

After `assert` replaces the guards and the tree is built with
`LazySegTree<Pos, ?Int, Int>(Action<?Int, Int>(id: 0, compose: +, apply: (f, x, _) => x + f, over: max))`,
it checks clean. The stdlib's `AddMax` is `Int`-only, so a "max over
`?Int` with range add" has to be spelled out inline. That is a small
stdlib coherence gap, not a compiler one.

**`scout_batch_10/noi15p5`: real gap (B2).** `var (pairs, best) = (0, none)`
fails with "can't tell which optional this `none` is". `best` is later
assigned `Int` values and compared with `max=`, but inference does not look
at later uses of a tuple-bound `none`. With `: (Int, ?Int)` it checks clean.

**`scout_batch_36/cco19p4`: real gaps (B1, B3).**
- Two array builders have a block body: `let score: [0..=n]F64 = [0..=n] d =>` followed by an indented `if`, and the same shape for `hulls`. The parser wants a value on the same line as `=>`.
- With both moved into named `fn`s, the next error is E0402 on `hulls[r].insert(Curve(..))`: "the argument must be passed as `mut Curve(..)`". Doing exactly that fails with E0401, "only a variable, a field, or an element can be assigned to". The hint is impossible to follow.
- The workaround is `var c = Curve(..); hulls[r].insert(mut c)`, after which it checks clean. The root cause is wave 1 gap 6: a by-value parameter cannot be mutated, so the method takes `mut c` by reference just to edit its own copy.

**`scout_batch_9/ioi10p6`: stale port plus a diagnostics bug (B4).**
- The port calls `Rooted(roads, City.first)` with an edge array. `Rooted` now wants a `Graph`, and with `Rooted(Graph.undirected(roads), City.first)` it checks clean.
- The error is reported at **line 1, column 1** (a comment), and it names an internal method the user never wrote: "a `[Road](u: City, v: City)` has no method `out`".
- The same happens for `Graph.undirected(e)` with an anonymous `[1..3]` edge domain: "doesn't know the name `E` here", again at 1:1.

Errors raised while instantiating a stdlib generic lose the user's call
site.

## New real gaps, clustered into general fixes

### W2. A worklist loop over a list that grows

Ports: `cf2245d2`.

Peeling and BFS orders want both a queue and the final order with each
vertex's position, so `cf2245d2` keeps a growing `order` list and walks it
by hand. `for v in order:` with a `push` inside is rejected by the borrow
rules, which is right for a general `for`. That leaves
`var z = 0; while z < order.len: ..; z += 1`, a shape 25 port
files use.

```cp
// today
var z = 0
while z < order.len:
    let v = order[z]
    ans[v] = if cnt[v][0] == 0: z - n else: n - z
    ...
        order.push(u)
    z += 1
// wanted: a worklist that remembers what it has handed out
var order = Worklist.of(starts)
while let (z, v) = order.next():
    ans[v] = if cnt[v][0] == 0: z - n else: n - z
    ...
        order.push(u)
```

Fix: a small `Worklist<T>` in `std.deque` (a list plus a read cursor)
with `of`, `push`, `next() -> ?(Int, T)` and `items`. `peel` and `bfs`
can be built on it. It is userland, so the borrow rules stay as they are.

### W3. Decimal digits of an `Int`

Ports: `noip21p1`.

A `% 10` / `/= 10` loop appears whenever a problem talks about digits
(3 port files end a loop with `x /= 10`). `x.digits` fails. Fix:
`x.digits(base = 10) -> List<Int>`, least significant first, next to
`popcount` and `bit_width`. Then `has7(x)` is `7 in x.digits`.

### W4. `+=` on every type that has `+`

Ports: `noip06p2`.

`a + b` joins lists, but `a += b` fails with "`+=` can't be applied to a
`List<Int>`". Compound assignment should mean `a = a + b` for every type
with `+` (lists, `Str`, user types with an `Add` impl), as it already does
for numbers and `Mod`. This is one rule in the checker, not a new method.

### W5. `SegTree.update`

Ports: `dmopc20c7p4`.

`LazySegTree` has `update(i, f)`, but `SegTree` does not, so a point
change is written `near.set(i, near.get(i) + d)` with the index twice.
Fix: add `update(mut self, i: D, f: T -> T)` to `SegTree` (and `Fenwick` if
it lacks it), so every point-update structure has the same three verbs:
`get`, `set` and `update`.

### W6. The child end of a tree edge

Ports: `btoi17p3`.

Edge-indexed tree problems need "the endpoint of edge `e` that is farther
from the root". The ports write `if tree.parent(u) == v: u else: v`
(4 port files). Fix: `Rooted.lower(u, v) -> V` (and `Rooted.edge_child(e)` when the
tree was built from an edge domain). It is a cohesive `Rooted` method, not
a problem helper: path sums, cut edges and virtual trees all use it.

### From the broken ports

- **B1. Block bodies after `=>`.** Array builders and lambdas cannot continue onto an indented block, so `cco19p4` needs two throwaway `fn`s.
- **B2. `none` inferred from later uses.** A tuple-bound `none` needs its type written out, even when the next assignment says it.
- **B3. A fix hint that cannot be followed.** E0402 tells the user to write `mut Curve(..)`, which E0401 then rejects. The hint should suggest `var c = ..; f(mut c)` until by-value mutable parameters land (wave 1, gap 6).
- **B4. Errors inside stdlib generics land at 1:1.** They should point at the user's call and name the user's types, not internal methods such as `out`.

## Real gaps already known, confirmed again

- Cyclic neighbours `x[(i + 1) % n]` (`btoi17p3`, `cf2255a`, `ucup5s05b`; C5).
- Reading `?T` where 0 means none (`noip06p2`; Syntax Batch #3).
- Tuple patterns over optionals (`noi18p4`; Syntax Batch #4).
- `Subset.of` / `Set.of` in `cpl run` (`dwite10c3p3`; Syntax Batch #4).
- The clamped prefix (`nccc10s3`; wave 2, N1).
- Domain label crossings such as `Pkg(d.label)` (`noi15p2`; backlog batches 13, 17, still missing from the friction queue).

## Stale ports: rewrites that need no compiler work

- `ecnerwala/cf2245d2` lines 7, 11, 13, 22, 24: `u: 1..=n` and `kind: 1..=2` read as `Int`, then `- 1` on every use; `domain V = 1..=n` and a `[V, 1..=2]Int` array index directly (1-based domain arrays run).
- `ecnerwala/cf2245d2` lines 8, 10-12, 22: adjacency by hand, and the other endpoint as `u - 1 + v - 1 - v`; `Graph.undirected(edges)` gives `a.to` and `a.e`.
- `ecnerwala/cf2245d2` lines 9: `[[0, 0] for _ in 0..n]` is `var cnt: [V, 1..=2]Int = 0`.
- `ecnerwala/cf2245d2` lines 14-15, 25: the same `cnt[v][0] == 0 or cnt[v][1] == 0` test written three times; a local `fn free(v)` names it.
- `ecnerwala/cf2262a1` lines 9-10: `a[i - 1]` for `i in 1..=n`; `let a: [1..=n]Int = read()` indexes by `i` (runs).
- `ecnerwala/cf2262a2` lines 14-16: `a[i - 1]` again; read `a` as `[1..=n]Int`.
- `ecnerwala/cf2262a2` lines 14: `need[j * i] = min(need[j * i], ..)` is `need[j * i] min= ..`.
- `ecnerwala/cf2262a2` lines 18: `need[i] = min(need[i], need[i + 1])` is `need[i] min= need[i + 1]`.
- `ecnerwala/cf2268b` lines 10-12: `a[p - 1]`; `var a: [1..=n]Int = read()` indexes by `p`.
- `ecnerwala/ucup5s03a` lines 19-20: `[Z(0) for _ in 0..n + 5]` is `List.filled(n + 5, Z(0))`.
- `ecnerwala/ucup5s06a` lines 11-13: a `var s` accumulation loop; `sum(Z(i) * f[i] * g[n - i] for i in 1..=n)`.
- `ecnerwala/ucup5s06a` lines 21: `[Z(0) for _ in 0..=n]` is `List.filled(n + 1, Z(0))`.
- `wave2/ccc20s2` lines 5: `use std.graph.{bfs}`: it is in the prelude.
- `wave2/ccc21s2` lines 9: `row_odd[at - 1]`; index a `[1..=rows]Bool` by `at`.
- `wave2/yac1p1` lines 16: `for i in ..: total += ..` is `sum(..)`, which works on `Mod`.
- `scout_batch_10/btoi19p5` lines 17, 24-25: `border[pat.len + 1..border.len]`, `s[cut..n]`, `s[0..cut]` are `border[pat.len + 1..]`, `s[cut..]`, `s[..cut]`.
- `scout_batch_10/btoi19p5` lines 32: `[c for c in s]` to make a `List<Char>`; `prefix_function` takes a `Str` directly (runs).
- `scout_batch_10/noip20p2` lines 12: `z_function([c for c in s])`; pass `s`.
- `scout_batch_10/ctsc06p1` lines 11: `use std.mod.{Mod}`: `Mod` needs no `use`.
- `scout_batch_10/ctsc06p1` lines 16, 28: a `digit` helper and four concatenations to zero-pad; `print("{w:04}")` runs.
- `scout_batch_10/ctsc06p1` lines 22: `[name[i] for i in I]`; `prefix_function` takes a domain array directly (runs).
- `scout_batch_10/ctsc06p1` lines 25: `wait = wait + ..` is `wait += ..`.
- `scout_batch_10/nccc10s3` lines 15: `suffix_array([c for c in s])`; pass `s`.
- `scout_batch_10/nccc10s3` lines 18-24: a hand-written `lo`/`hi` binary search; `bsearch_first(0..n, i => not ok(..)) ?? n`.
- `scout_batch_26/dmopc20c7p4` lines 8-10: `use std.graph.{Graph}`, `use std.tree.{Rooted}`, `use std.segtree.{SegTree}`: none is needed.
- `scout_batch_26/dmopc20c7p4` lines 16-17: `Graph<V, E>.undirected` plus `Rooted(u => g.adj(u), ..)` is `Rooted(Graph.undirected(edges), V.first)`.
- `scout_batch_2/noi15p2` lines 16, 18: a child list of `(to: Pkg)` records to feed `Rooted`; `Rooted.from_parents` takes the parent array.
- `hillclimb_dp_graph/sac22cc3p4` lines 21: `depth(u) + depth(v) - 2 * depth(lca(u, v))` is `tree.dist(u, v)` (runs).
- `scout_batch_11/noip21p1` lines 22-25: `var j = i; while j <= LIMIT: ..; j += i` is `for j in (i..=LIMIT).step(i)` (runs).
- `scout_batch_11/noi18p4` lines 26: `sorted([s for s in start])` is `sorted(start)`; `sorted` takes a domain array (runs).
- `scout_batch_11/noi18p4` lines 32-36: a linear scan for the largest sword at most `life`, then a full re-sort; `bsearch_last` on the sorted list, or a `SortedBag` with `remove`.
- `scout_batch_11/noi18p4` lines 37: `at_least = max(at_least, ..)` is `at_least max= ..`.
- `btoi18p6` lines 13-16: adjacency by hand; `Graph.undirected(edges)`.
- `sim3/p4_abc480_f` lines 8, 27: `sum(a[i] for i in I)` is `sum(a)`.
- `sim3/p4_abc480_f` lines 20: `if rem[ns] == 0: 1 else: 0` is `Int(rem[ns] == 0)`.
- `sim3/p4_abc480_f` lines 32-33, 40: `best = -1` as "unreachable"; `?Int` with `max=`.
- `sim3/p4_abc480_f` lines 35, 38: `s == Subset<I>.empty` is `s.is_empty`.
- `scout_batch_28/dwite10c3p3` lines 25: `(x + y) % MOD` by hand; `type Z = Mod<MOD>`.

## Rejected

- `ecnerwala/cf2255b` lines 10-11, 13-14: `[0, 0]` counters indexed by `c - '0'`; plain and short.
- `ecnerwala/cf2255c` lines 16, 28: `+ 1` when printing 1-based answers; an output format, not friction.
- `ecnerwala/cf2262a1` lines 11: a difference array summed in place; one obvious line.
- `ecnerwala/ucup5s05b` lines 9: a local `sign`; the friction queue already rejected a stdlib `sign` as one-off.
- `scout_batch_10/noip20p2` lines 17-25: two parity-toggle loops in opposite directions; clear as written.

## New detector rules for `scripts/friction_lint.py`

These continue the numbering after wave 2's 27 rules. The script is still
not on `main`.

28. `while (\w+) < (\w+)\.len:` where the body pushes to the same list: report W2 until `Worklist` lands. 25 files match the loop shape.
29. `(\w+) /= 10$` inside a `while \1 > 0` loop: report W3, then suggest `x.digits`. 3 files.
30. `(\w+) = \1 \+ \[`: report W4, then suggest `+=`.
31. `\.set\((\w+), (\w+)\.get\(\1\) [-+*]`: suggest `update(i, f)` once W5 lands.
32. `if tree\.parent\((\w+)\) == (\w+): \1 else: \2`: report W6. 4 files.
33. `\[c for c in (\w+)\]` passed to a `std.strings` function: pass the `Str` directly. 6 files.
34. `use std\.(graph|tree|segtree|mod|deque)\.\{(Graph|Rooted|SegTree|Mod|Deque)\}`: the name is in the prelude. 30 files.
35. `(\w+)\[(\w+) - 1\]` where `\2` loops over `1..=n`: read the array as `[1..=n]T` and index by `\2`. 58 files match the shape; review them, do not auto-fix.
36. `\bwhere\b` after `read()` or a `fn` signature: the guard was removed in 0.2.2, so use `assert`. This would have flagged `yac6p3` and `dpw`.
37. `=>\s*$` (a lambda or array builder ending its line): report B1.
