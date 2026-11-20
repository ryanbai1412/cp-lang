# Wave 1 blind critic: 30 ports, CP Lang 0.2.2

Reviewed at `733b152`. Every "Fixable" rewrite below is applied in `wave1-critic.patch` and all 30 ports pass
`proto/ports/wave1/gate.sh <name> 100` with the patch applied. Every "Blocked" spelling was tried against the
release `cpl`; the error quoted is the exact one it printed. Line numbers refer to the files at `733b152`.

Totals: **30 fixable** (F1..F30, in 20 ports), **39 blocked** (B1..B39), **1 correctness doubt with a failing input** (dpg).

Recurring blockers worth fixing once, in the order they hurt most:

- Helpers the cheat sheet advertises need a `use`: `bsearch_first`, `choose`, `desc`, `cross`, `scan`, `topo`, `divisors` (B1, B17, B19, B21).
- `?Int` is half-supported: no `+=` (B6), no `max(it)` / `min(it)` (B20, B39), no `<=` and no `if let` in a comprehension filter (B24, B25).
- A domain index can't be compared with an `Int`, so `Int(i)` escapes survive (B12, B38).
- `a.items` is missing on domain arrays, optional arrays and `Map` (B8, B11, B27).
- `Graph.directed(read())` needs explicit type arguments and fails with an unrelated error without them (B22).
- `@memo` recursion overflows an 8 MB stack near depth 5e4 in emitted C++ (dpg).

## aac4p2

Fixable:
- **F1** line 5: `const CAP = 2_000_000_000` -> `const CAP = 2e9`.

Blocked:
- **B1** line 3: `use std.seq.{bsearch_first}` should not be needed (cheat sheet lists it as core). Without it: `error[E0201]: no `bsearch_first` in scope` / `help: `bsearch_first` is in the standard library: add `use std.seq.{bsearch_first}``. Same in ccc21s3 line 4 and ioi10p3 line 3.
- **B2** line 19: wanted `bsearch_first(I, i => ...)`. `error[E0302]: `bsearch_first` takes a Range<I> for `r`, but `I` is a domain I`. `I.first..` is the clunky stand-in.
- **B3** line 19: wanted `print(bsearch_first(...), none: -1)` instead of `if let i = ...: print(i) else: print(-1)`. `error[E0310]: `print` has no argument named `none``.
- **B4** line 19: wanted `print(bsearch_first(...) ?? -1)`. `error[E0302]: expected `I`, found `Int``.
- **B5** lines 10-14: the 5-line running lcm should be `let pre: [I]Int = scan(a, (x, y) => min(CAP, lcm(x, y)))`. `error[E0302]: expected `[I]Int`, found `List<Int>`` (`scan` drops the domain, and also needs `use std.seq.{scan}`).

## apio09p3

Fixable:
- **F2** line 14: `Graph<Junction, Road>.directed(roads)` -> `Graph.directed(roads)` (the type arguments are inferred).

Blocked:
- **B6** line 23: `best[d] = best[d] + loot[d]` should be `best[d] += loot[d]`. `error[E0309]: `+=` can't be applied to an ?Int`.
- **B7** line 22: `for e in onward[d]: ... best[e.to]`: `condense` yields one-field records `(to: C)`; wanted `for e in onward[d]: best[d] max= best[e]`. Stdlib shape (`condense -> [C]List<(to: C)>`), no compiler error to quote.

## btoi17p3

Fixable:
- **F3** line 7: `Rooted(Graph<City, Track>.undirected(tracks), City.first)` -> `Rooted(Graph.undirected(tracks), City.first)`.
- **F4** line 16: `sorted(plan, by: c => tree.pos(c))` -> `sorted(plan, by: tree.pos)`.
- **F5** lines 17-21 and 26: every mark was doubled for no reason. `let (a, b) = ...; marks[a] += 1; marks[b] += 1; marks[tree.lca(a, b)] -= 2` and `>= 2 * k` -> `marks[tour[i]] += 1; marks[tree.lca(tour[i], tour[(i + 1) % s])] -= 1` and `>= k` (comment updated).
- **F6** line 25: `fn lower(t: Track) -> City = if tree.parent(tracks[t].u) == tracks[t].v: tracks[t].u else: tracks[t].v` (four `tracks[t].`) -> `fn lower(u: City, v: City) -> City = if tree.parent(u) == v: u else: v`, called as `lower(tracks[t].u, tracks[t].v)`.

Blocked:
- **B8** lines 25-26: wanted `[t for (t, (u, v)) in tracks.items if ...]`. `error[E0203]: a `[Track](u: City, v: City)` has no member `items` that `cpl run` knows` (cheat sheet lists `a.items`).
- **B9** line 25: wanted `fn lower((u, v): (City, City))` so `lower(tracks[t])` works. `error[E0108]: expected a parameter name, found `(``.
- **B10** lines 14-15: `let s = read(); let plan: [0..s]City = read()` should be one length-prefixed read, `let plan: List<City> = read()`. `error[E0501]: `read()` has no input format for a `List<Int>``.

## ccc10s4

Fixable:
- **F7** lines 9-25: sort all sides, then compare each with both neighbours by index (`sides[j + 1].a`, `sides[j - 1].b`, `j == 0`, ...). Replaced with a `Map<(Int, Int), (cost, pen)>` of sides seen once: `if let other = open.remove(side): fences.push(...) else: open[side] = ...`, then `for (_, (cost, pen)) in open: fences.push((cost, pen, Pen.first))`. 17 lines -> 13, no index arithmetic.
- **F8** line 10: `for pen in Pen if pen != Pen.first:` -> `for pen in Pen.first.r..:`.

Blocked:
- **B11** (F7's last line): wanted `open.items` / `open.values`. `error[E0203]: a `Map<(Int, Int), (cost: Int, pen: Pen)>` has no member `items` that `cpl run` knows`. Bare `for (k, v) in open` works, so this is only an inconsistency with arrays.

## ccc17s4

Fixable:
- **F9** line 11: `sorted([e for e in Pipe], by: e => (pipes[e].cost, is_new(e)))` -> `sorted(Pipe, by: e => pipes[e].cost)`: `sorted` takes a domain and is stable, so old pipes already come first among equals.

Blocked:
- **B12** line 10: `fn is_new(e: Pipe) -> Bool = Int(e) >= n - 1`. Wanted `e >= n - 1`. `error[E0302]: can't compare a `Pipe` with a `Int``. (No stdlib way to split a domain into "first n - 1" and "rest" either.)

Remaining wart, no better spelling found: line 25 `- Int(saved)`.

## ccc18s5

Fixable:
- **F10** lines 14-15: `var links = [... for f in flights]` then `for g in portals: links.push(...)` -> one `let links: List<(cost: Int, link: Link)> = [...] + [...]`.

Blocked:
- **B13** line 22: wanted `sorted(links)` (cost is the first field). `error[E0309]: a `(Int, Link)` has no order to sort by` (enums derive no order).
- **B14** line 14: the `List<(cost: Int, link: Link)>` annotation exists only to name the fields for `by: l => l.cost`. Without it, `by: l => l[0]`: `error[E0305]: a `(Int, Link)` can't be indexed`; and `by: (cost, _) => cost` is taken as a comparator: `error[E0302]: expected `Bool`, found `(Int, Link)``.

## ccc19s5

No fixable findings.

Blocked:
- **B15** lines 6-7: wanted `let (n, k): (1..=3000, 1..=n) = read()`. `error[E0201]: `cpl run` doesn't know the name `n` here`. Same in dmpg16s5 lines 5-6.

## ccc21s3

No fixable findings. Blocked: the `use` on line 4 (see B1; not counted again).

## ccc21s5

Fixable:
- **F11** lines 9-14, 20, 22: two arrays `opens` / `closes` -> one difference array over boundaries, `var delta: [0..=16, I.bounds]Int`, `delta[x, l.l] += 1; delta[x, r.r] -= 1`, and a single `open += delta[x, i.l]` in the sweep.
- **F12** lines 6, 21, 25: `domain X = 1..=16` bought nothing but two `Int(x)` escapes. Read `x: 1..=16` as a plain range and both `Int(x)` go away (`lcm(a[i], x)`, `== x`).

Blocked:
- **B16** F12 wastes row 0 because the natural `[1..=16, I.bounds]Int` fails: `error[E0501]: `cpl run` and `cpl build` can't handle array dimensions that don't start at 0 yet`.

## ccc22s4

Fixable:
- **F13** lines 12-13, 20-21: a second domain `Twice`, a `Prefix1<Twice>` and three escapes (`Spot(Int(t) % c)`, `Twice(Int(p) + 1)`, `next.l..next.r(c / 2)`) just to sum the next `c / 2` spots. Replaced with a sliding count: `var ahead = sum(at[Spot.first.cyc(k)] for k in 1..=c / 2)` and, per spot, `ahead += at[p.cyc(c / 2 + 1)] - at[p.cyc(1)]`.
- **F14** line 26: `for p in Spot.first..Spot(c / 2):` -> `for p in Spot:` with `if p > q: break` after `let q = p.cyc(c / 2)`.

Blocked:
- **B17** line 4: `use std.math.{choose}` should not be needed. Without it: `error[E0201]: no `choose` in scope`.
- **B18** line 13: the unrolled ring without a named domain, `prefix1([0..2 * c] t => at[t % c])`: `error[E0201]: `cpl run` doesn't know the name `D.bounds` here` (reported at 1:1, on the `use` line).

## ceoi18p1

Fixable:
- **F15** lines 10-14 and 19: `type Deal`, a comprehension plus a push loop, and `sort(mut deals, by: d => (-d.rate, -d.cores))` -> `var deals = [...] + [...]`, `sort(mut deals)`, `for (_, cores, gain) in deals.rev`. The tuple order (rate, cores) already is the wanted order, reversed.

Blocked:
- **B19** line 14: wanted `sorted(..., by: desc)` in one expression. `error[E0201]` with `help: `desc` is in the standard library: add `use std.algebra.{desc}`` (`Desc` is in the prelude, `desc` is not). Same in dmopc21c2p1 line 4.
- **B20** lines 25-27: `var profit = 0; for b in best: profit max= b; print(profit)` should be `print(max(best)!)`. `error[E0309]: a `?Int` has no order to compare by` (same for `max(b for b in best)`).

## cheerio1s3

Fixable:
- **F16** lines 5-8, 12-17: a hand-rolled `Slope` tuple and cross-multiplying `steeper` -> read `[0..n]Pt`, `let cur = pts[j] - pts[i]`, `cross(top ?? cur, cur) >= 0` and `cross(cur, low ?? cur) >= 0`.

Blocked:
- **B21** F16 needs `use std.geometry.{cross}` although `Pt` is in the prelude, and `a.cross(b)` fails: `error[E0203]: Pt has no field or method `cross``.

## coi06p2

Fixable:
- **F17** lines 11, 14, 21, 32: four `Int(tree.pos(..))` escapes. `var low = [Town] v => tree.pos(v)` keeps the positions typed; `min=`, `<` and `<=` all work on them.

Blocked:
- **B22** line 6: wanted `Graph.undirected(read())` with `Town` / `Road` in scope. `error[E0203]: a `Int` has no member `to_idx` that `cpl run` knows`, reported at 1:1 on an unrelated line. Same in dpg after F19.

## dmopc21c2p1

Fixable:
- **F18** lines 4, 12: `use std.algebra.{desc}` + `sorted(heights, by: desc)` -> `sorted(heights).rev`, no import.

Blocked: `by: desc` without the `use` (see B19; not counted again).

## dmpg16s5

No fixable findings.

Blocked:
- **B23** lines 9-17: a hand-written monotone deque with `(cut, pre)` pairs, `window.first!` twice and an expiry test `window.first!.cut < cut + 1 - k`. The stdlib has no sliding-window minimum (no `window`/`Mono*` in `stdlib/`); wanted something like `for (cut, low) in sliding_min(pre, k)`.
- The dependent read on lines 5-6 (see B15; not counted again).

## dpe

No fixable findings.

Blocked, all on line 11 (`if lightest[total] != none and lightest[total]! <= cap`):
- **B24** wanted `if lightest[total] <= cap`. `error[E0308]: an optional has no order: `none` is neither small nor large`.
- **B25** wanted `if let w = lightest[total] if w <= cap`. `error[E0501]: `cpl run` and `cpl build` can't handle more than one `if`, or an `if let`, in a comprehension yet`. (`if let w = lightest[total] and w <= cap` in a comprehension: `error[E0307]: `and` needs a Bool, but `lightest[total]` is an ?Int` (probe used the name `best[t]`).)
- **B26** wanted `(lightest[total] ?? cap + 1) <= cap`. `error[E0110]: `??` mixed with `+ - * / %` needs parentheses`.
- **B27** wanted `for (total, w) in lightest.items`. `error[E0203]: a `[Int]?Int` has no member `items` that `cpl run` knows`.

## dpg

**Correctness doubt, with a failing input.** The original port is `@memo fn longest(u)` recursing along edges. On the chain `n = 100000`, `m = 99999`, edges `i -> i + 1` (within constraints), the emitted C++ (`cpl build`, `g++ -O2`, default 8 MB stack) dies with a segmentation fault; `n = 40000` survives, `n = 60000` crashes. The interpreter and the original C++ both print `99999`. Each level costs a memo wrapper, a body and two lambdas.

Fixable:
- **F19** whole file: replaced the recursion with `for u in topo(dag)!.rev: for (v, _) in dag.out(u): longest[u] max= longest[v] + 1` and `print(max(longest))`. Passes the gate and the chain input.

Blocked:
- **B28** the `@memo` form is the one a contestant will type; it must not overflow the stack at `n = 1e5` (error above: `Segmentation fault` at run time in the emitted binary). F19 also needs `use std.graph.{topo}`, an edge domain and explicit `Graph<V, E>` (B22).

## dph

No fixable findings.

Blocked:
- **B29** lines 11-12: wanted `for (r, c) in (Row, Col) if grid[r, c] == '.':`. `error[E0302]: a `for` can't walk over a `(domain, domain)``.
- **B30** line 3: wanted `Mod<1e9 + 7>`. `error[E0105]: expected a type`. Same in dps line 3 and noi14p4 line 4.

## dps

Fixable:
- **F20** line 19: `below[Rem.first] + Z(Int(prefix == Rem.first)) - 1` -> `below[Rem.first] - Int(own != Rem.first)`.
- **F21** lines 9, 11, 16, 18: the variable `prefix` (the digit sum of K's prefix, and the name of a stdlib helper) -> `own`.

Blocked:
- **B31** line 19: a Bool into a `Mod` directly, `Z(prefix == Rem.first)`: `error[E0302]: expected `Int`, found `Bool``.

## dpz

Fixable:
- **F22** lines 17, 20: `hull.get(1)`, `hull.get(0)`, `hull.get(hull.len - 2)` -> `hull[1]`, `hull[0]`, `hull[hull.len - 2]`.

Blocked:
- **B32** line 20: wanted `hull[-2]` for the second from the back. Compiles, then `error[R0001]: assertion failed: 0 <= i and i < n` at run time, reported at 1:1.

## ioi10p3

Fixable:
- **F23** line 12: `Prefix2<Row, Col>(...)` -> `Prefix2(...)`.
- **F24** line 13: `low.sum(i.l..i.r(h), j.l..j.r(w)) ... for i in Row.first..=Row(r - h) for j in Col.first..=Col(c - w)` -> `low.sum(rows, cols) ... for rows in Row.windows(h) for cols in Col.windows(w)`.

Blocked: the `use` on line 3 (see B1; not counted again).

## ioi11p4

Fixable:
- **F25** line 9: `Graph<Chamber, Corridor>.undirected([Corridor] c => (corridors[c].u, corridors[c].v))` -> `Graph.undirected(corridors)`; the extra `time` field is accepted.

No blocked findings.

## noi14p4

No fixable findings.

Blocked:
- **B33** lines 16-17: `fn grow(b: Int, i: Int)` then `var b = b`. Wanted `fn grow(var b: Int, i: Int)`. `error[E0108]: expected a parameter name, found `var``.

Not a rewrite I could make shorter: `prefix_function(s)` (std.strings) gives `border`, but `grow` is still needed for `half`.

## noip04p2

No fixable findings.

Blocked:
- **B34** lines 6-7: `var heap = MinHeap<Int>()` + `for p in piles: heap.push(p)` should be `MinHeap(piles)`. `error[E0303]: `MinHeap<Int>` takes 0 arguments, but 1 was given`.

## noip07p1

No findings.

## noip07p2

No fixable findings.

Blocked:
- **B35** line 11: `'A' + (c - 'a')` should be `c.upper`. `error[E0203]: a `Char` has no member `upper` that `cpl run` knows`. (`c - 'a' + 'A'` also fails: `error[E0309]: `+` can't be applied to `Int` and `Char``.)
- **B36** line 21: `for _ in 0..repeat: out.push(shown(c))`; a repeat operator fails: `error[E0309]: `*` can't combine a Str and an Int`.

## noip09p2

Fixable:
- **F26** whole file (36 lines -> 7): hand sieve, `vp`, per-prime exponent counting and a leftover-prime case -> `print(count(gcd(x, a0) == a1 and lcm(x, b0) == b1 for x in divisors(b1)))` under `@!multitest`. Cost: at 2000 tests with `b1` near 2e9 the emitted C++ takes 0.22 s against 0.03 s for the old port and 0.12 s for the original C++. If that is too slow for the perf track, the smaller verified fix is to keep the algorithm and replace lines 5-9 with `let primes = Sieve(LIM).primes` (also passes the gate).

No blocked findings beyond the `use std.math.{divisors}` (B17 family; not counted again).

## noip09p3

Fixable:
- **F27** lines 19-36: `spread` took a `key` and the caller negated prices (`p => -p`, then `-dearest[c] - cheapest[c]`). It now takes `pick` and is called as `spread(fwd, City.first, min)` / `spread(back, City.last, max)`; the update test `if best[v] == none or via < best[v]!` -> `if best[v] != pick(best[v] ?? via, via)`; the answer is `dearest[c] - cheapest[c]`.

No blocked findings.

## noip10p3

No fixable findings.

Blocked:
- **B37** lines 9-16: the `foe` array with two `if let ... else` lines stands in for a union-find over (criminal, side). Wanted `Dsu<(V, Bool)>()` with `unite((a, true), (b, false))`. `error[E0301]: this is indexed by `Int`, but the index is a `(V, Bool)``, reported at 1:1.

## noip14p3

Fixable:
- **F28** lines 6, 9, 11, 23: `domain X = 0..n` forced `gap[Int(x) + 1]`. With `domain X = 1..=n` (the column being arrived at), `at: X` and `gap: [X]`, it is `gap[x]`.
- **F29** lines 15, 25, 28: the running `passed` counter and its `any(t != none for t in next)` test -> `print(count(p.at < x for p in pipes))` at the point of death.
- **F30** lines 31-34: `var fewest: ?Int = none; for t in taps: fewest min= t; print(fewest!)` -> `print(min(t! for t in taps if t != none))`.

Blocked:
- **B38** line 24: `Int(h) <= low or Int(h) >= high`. Wanted `h <= low or h >= high`. `error[E0302]: can't compare a `H` with a `Int``.
- **B39** F30 should be `print(min(taps)!)`. `error[E0309]: a `?Int` has no order to compare by`.
