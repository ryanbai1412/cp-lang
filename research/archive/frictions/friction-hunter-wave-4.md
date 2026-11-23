# Friction Hunter, wave 4, and the minimal-core fix plan

Thirty more solved ports, none of them in waves
[1](friction-hunter-wave-1.md), [2](friction-hunter-wave-2.md) or
[3](friction-hunter-wave-3.md), which brings the audit to 120 ports. I read
every line of each. Each pattern was checked against `cpl` on `main` at
`f09d7b6` and labelled [STALE PORT], [REAL GAP] or [REJECTED] as before.
Every finding was deduplicated against the earlier waves and
`research/archive/frictions/friction-queue.md`.

The second half of this file is the **master synthesis**: every unique
verified real gap from waves 1 to 4, sorted into three ready-to-run batches.

## The numbers

Wave 4: the 30 ports have 851 non-blank lines. They hold 54
frictions covering 104 awkward lines: 20 real gaps (50
lines), 29 stale-port patterns and 5 rejected.

- Wave 4 real gaps per problem: 20 / 30 = **0.67**.
- Wave 4 awkward lines per problem: 104 / 30 = **3.47**.
- New real gaps: 7, in 6 clusters (X1 to X6).
- Ports with no friction at all: 13 (`cf2245a`, `cf2245b`, `cf2261a`, `cf2261d1`, `ioi00p1`, `ioi94p1`, `noip00p1`, `noip01p2`, `noip04p1`, `noip06p1`, `ccc03s4`, `sac22cc3p3`, `tle17c5p1`).

Cumulative over all 120 ports (wave 2 corrected to 31 real gaps, as wave 3
explains):

- Wave 1: 18 real gaps and 112 awkward lines, so 0.60 and 3.73 per problem.
- Wave 2: 31 real gaps and 182 awkward lines, so 1.03 and 6.07 per problem.
- Wave 3: 13 real gaps and 108 awkward lines, so 0.43 and 3.60 per problem.
- Wave 4: 20 real gaps and 104 awkward lines, so 0.67 and 3.47 per problem.
- **All 120 ports: 82 real gaps and 506 awkward lines, so 0.68 real gaps and 4.22 awkward lines per problem.**

Wave 4 drew more short, classic problems (NOIP, IOI 1994-2000, Codeforces
A/B). Thirteen of those are clean on HEAD, which is why its averages are
the lowest so far.

## The 30 ports

- `ecnerwala/cf2245a` (greedy strings): 0 frictions.
- `ecnerwala/cf2245b` (greedy, sorting): 0 frictions.
- `ecnerwala/cf2261a` (strings): 0 frictions.
- `ecnerwala/cf2261b` (bits): 1 frictions.
- `ecnerwala/cf2261d1` (constructive): 0 frictions.
- `ecnerwala/cf2268a` (greedy): 1 frictions.
- `ecnerwala/ucup5s05f` (suffix array, power series): 8 frictions.
- `wave2/ioi00p1` (interval DP): 0 frictions.
- `wave2/ioi94p1` (triangle DP): 0 frictions.
- `wave2/noip00p1` (negative base): 0 frictions.
- `wave2/noip01p2` (partition DP): 0 frictions.
- `wave2/noip04p1` (simulation): 0 frictions.
- `wave2/noip06p1` (cyclic interval DP): 0 frictions.
- `scout_batch_11/sdoi17r1p1` (Möbius, quotient blocks): 5 frictions.
- `scout_batch_29/ccc19s2` (sieve): 3 frictions.
- `scout_batch_39/dmpg18s5` (sqrt decomposition, divisors): 3 frictions.
- `scout_batch_8/noip14p6` (polynomial roots mod p): 2 frictions.
- `scout_batch_20/cco17p4` (knapsack, brute check): 3 frictions.
- `scout_batch_3/ccc03s4` (suffix array, LCP): 0 frictions.
- `scout_batch_46/sac22cc3p3` (radix sort): 0 frictions.
- `scout_batch_10/saspeed` (suffix array): 1 frictions.
- `scout_batch_33/cco20p4` (constructive tours): 3 frictions.
- `scout_batch_8/cco19p3` (tree DP, meet in the middle): 4 frictions.
- `sim3/p1_abc476_g` (bits, digit DP): 2 frictions.
- `sim3/p3_abc481_f` (layered BFS): 3 frictions.
- `scout_batch_40/tle17c5p1` (interactive): 0 frictions.
- `scout_batch_44/dmopc21c2p3` (interactive, bits): 2 frictions.
- `scout_batch_19/ioi07p1` (interactive, galloping search): 4 frictions.
- `scout_batch_28/ioi16p6` (Aliens trick, CHT): 6 frictions.
- `scout_batch_27/btoi18p1` (functional graphs, matching): 3 frictions.

## New real gaps in wave 4

- **X1. Two names for one function.** `std.math.bit_len` is defined as `bit_width`, which is already in the prelude, so `cf2261b` imports a duplicate. Fix: delete `bit_len`. One concept should have one name.
- **X2. `List<Mod<P>>` prints one value per line.** `print([1, 2])` prints `1 2`, but `print([Z(1), Z(2)])` prints two lines. `ucup5s05f` maps `.v` over its output to get one line. Fix: one print rule for every `List<T>`, regardless of `T`.
- **X3. `Sieve` has no `mu` or `phi` table.** `sdoi17r1p1` derives Möbius from `factor(x)[0].0`. Fix: `Sieve.mu(x)` and `Sieve.phi(x)`, filled by the same linear sieve.
- **X4. A huge decimal cannot be read into `Mod<P>`.** `let x: Mod<P> = read()` fails with "does not fit in an `Int`", so `noip14p6` (10,000-digit coefficients) and `p3_abc481_f` fold the digits by hand. Fix: reading `Mod<P>` reduces the decimal while parsing, and `Mod<P>.parse(s: Str)` does the same for a token already read (for two moduli).
- **X5. No search on an open range.** `bsearch_last(0.., p)` fails with "can't handle an unbounded range value yet", so `ioi07p1` doubles `hi` by hand first. Fix: `bsearch_first` / `bsearch_last` gallop on `lo..`: double until the predicate flips, then bisect.
- **X6. No get-or-insert on `Map`.** Interning names into dense ids takes a three-line `fn id`. Fix: `m.get_or_insert(k, v) -> V`, which returns the stored value or inserts `v` (the `v` is evaluated only on insert).

**The docs-gate `topo` / `peel` gap is confirmed on HEAD.** A `Graph` built
from a `List<(V, V)>` makes every algorithm on it fail:

```cp
domain V = 0..3
let g = Graph.directed([(V(0), V(1)), (V(1), V(2))])
print(topo(g)!)     // error[E0201]: `cpl run` doesn't know the name `V` here, at line 1:1
```

The same program with the edges in a named `[E](u: V, v: V)` array runs.
So the vertex domain is not carried through when `Graph.directed` infers
its edge domain from a plain list. The error lands at 1:1, the same
diagnostics bug as wave 3's B4.

## Real gaps already known, confirmed again

- Tropical order on `?Int` (`p3_abc481_f`), `last` as a place (`ucup5s05f`), the clamped prefix and from-end index (`ucup5s05f`, `ioi16p6`).
- Domain label crossings (`cco17p4`, `cco19p3`, `cco20p4`), cyclic rotation (`cco20p4`), quotient blocks (`sdoi17r1p1`), `LiChao` (`ioi16p6`), permutation cycles (`btoi18p1`), multipoint `Poly` (`ucup5s05f`), reading `Bool` (`ioi07p1`).

## Stale ports: rewrites that need no compiler work

- `ecnerwala/ucup5s05f` lines 25, 80: `[List<Z>() for _ in 0..4 * n]` is `List.filled(4 * n, List<Z>())`; each element is its own list (runs).
- `ecnerwala/ucup5s05f` lines 28: `while p.len < n + 1: p.push(Z(0))` is `p = p.resized(n + 1, Z(0))`.
- `ecnerwala/ucup5s05f` lines 58: `if xs.len > 0: xs[xs.len - 1] else: -1` is `xs.last ?? -1`.
- `ecnerwala/ucup5s05f` lines 82: `[Z(0) for _ in 0..n]` is `List.filled(n, Z(0))`.
- `scout_batch_11/sdoi17r1p1` lines 7-8: `use std.mod.{Mod}` and `use std.math.{Sieve}`: both are in the prelude.
- `scout_batch_11/sdoi17r1p1` lines 33-37: `var mult = d; while mult <= top: ..; mult += d` is `for mult in (d..=top).step(d)`.
- `scout_batch_29/ccc19s2` lines 5: `use std.math.{Sieve}`: it is in the prelude.
- `scout_batch_29/ccc19s2` lines 10: `max(ns[c] for c in Case)` is `max(ns)`.
- `scout_batch_29/ccc19s2` lines 12-15: a `for` / `break` search; `let a = first(p for p in sieve.primes if sieve.is_prime(2 * n - p))!`.
- `scout_batch_39/dmpg18s5` lines 10: `[0..=MAXV] _ => []` is `= []`.
- `scout_batch_39/dmpg18s5` lines 27-28, 38-39: `l - 1`, `idx - 1`; read `a` as `[1..=n]Int`.
- `scout_batch_8/noip14p6` lines 28: Horner with `% primes[t]` after each step; with `Mod<P>` it is `y = y * x + a[i]`.
- `scout_batch_20/cco17p4` lines 15: `sorted([x for x in height])` is `sorted(height)`.
- `scout_batch_20/cco17p4` lines 16: `[rising[n - 1 - i] for i in 0..n]` is `sorted(height, by: desc)`.
- `scout_batch_10/saspeed` lines 10: `suffix_array([c for c in s])`; pass `s`.
- `scout_batch_33/cco20p4` lines 36: `sum(1 for c in City if seen[c]) == n` is `all(seen)`.
- `scout_batch_8/cco19p3` lines 17, 19-23: child lists plus a `while at < order.len` BFS for an order; `Rooted.from_parents` and `.preorder`.
- `scout_batch_8/cco19p3` lines 42-43: `[parts[i] for i in 0..cut]` is `parts[..cut]`.
- `scout_batch_8/cco19p3` lines 49, 66: `best = max(best, ..)` is `best max= ..`.
- `sim3/p1_abc476_g` lines 29: `(x & ((1 << (k + 1)) - 1)) == 0` is `ctz(x) > k`; `ctz` is a prelude function (only the method form `x.ctz` fails).
- `sim3/p1_abc476_g` lines 62-64: `var ans = 0; for q ..: ans max= best[q]` is `max(best)`.
- `sim3/p3_abc481_f` lines 11-13: adjacency by hand; `Graph.directed(edges)` with `edges[a.e].w` (runs).
- `scout_batch_44/dmopc21c2p3` lines 7-9: `let d: Int = ask(..); d` is `fn probe(g: Int) -> Int = ask(g, limit: 31_999)` (runs).
- `scout_batch_19/ioi07p1` lines 6: `use std.seq.{bsearch_last}`: it is in the prelude.
- `scout_batch_19/ioi07p1` lines 13: `x < 1 or x > n or y < 1 or y > n` is `not (x in 1..=n and y in 1..=n)`.
- `scout_batch_28/ioi16p6` lines 16: `-max(r, c)` to sort one key downwards; `(min(r, c), desc(max(r, c)))`.
- `scout_batch_28/ioi16p6` lines 17: `spans.len == 0 or -far > spans[spans.len - 1].hi`; `(spans.last?.hi ?? -1) < hi` or `if let`.
- `scout_batch_28/ioi16p6` lines 29: `(a + fall - 1) / fall` is `ceil_div`.
- `scout_batch_28/ioi16p6` lines 54-58: a hand-written binary search for the Aliens-trick fee; `bsearch_first(0..=m * m, fee => cover(fee).photos <= k)!`.

## Rejected

- `ecnerwala/cf2268a` lines 8: `a[n - 1 - i]` for the mirror element; one plain index.
- `scout_batch_11/sdoi17r1p1` lines 39: a prefix-product loop over `Mod`; one obvious line.
- `scout_batch_39/dmpg18s5` lines 15-17: square-root blocks by hand; the algorithm, not friction.
- `scout_batch_44/dmopc21c2p3` lines 35-41: climbing an implicit heap tree in the interactor; specific to the problem.
- `scout_batch_27/btoi18p1` lines 30-41: leaf peeling with per-step side effects; `peel` cannot express the pairing rule.

## New detector rules for `scripts/friction_lint.py`

`scripts/friction_lint.py` landed on `main` in `b497b2c` while this wave
was running. It has 7 categories. Several earlier proposals map onto them:

- `positional_action` is wave 2's rule 26.
- `manual_cursor_loop` is wave 3's rule 28 (the worklist shape).
- `missing_compound_assign` covers wave 1's rule 3 and wave 3's rule 30.
- `manual_temp_swap` covers the swap findings.
- `comprehension_copy` covers wave 1's rule 11 and part of wave 3's rule 33.

The rest of rules 1 to 37 and the rules below are still to be added as new
categories. These continue after wave 3's rule 37.

38. `use std\.math\.\{[^}]*\bbit_len\b`: use `bit_width` (and delete `bit_len`, X1).
39. `print\(\[.*\.v for `: report X2 until lists of `Mod` print on one line.
40. `\w+ = \(\w+ \* 10 \+ \(\w+ - '0'\)\) % `: a hand-rolled decimal parse mod p, so read as `Mod<P>` (X4).
41. `hi \*= 2` in a `while` before a `bsearch_`: report X5.
42. `if (\w+) not in (\w+): \2\[\1\] = `: use `get_or_insert` (X6).
43. `Graph\.(directed|undirected)\(\[`: report the `topo` / `peel` inference gap until it is fixed; prefer a named edge array.

# Master minimal-core synthesis (waves 1 to 4)

Every unique verified real gap from 120 ports, deduplicated and filtered
for Ryan's minimal core: a gap is fixed by a general compiler rule or by a
cohesive `stdlib/` API, never by a helper for one problem. Each batch is
5 to 8 items, sized for one Fast-mode worker. Each item names the ports
that hit it, so its tests can start from real code.

The `friction-queue.md` Syntax Batch #3 (complete `read()`: `Bool`, `?T`
with `none:`, enums, `read_line`) and Syntax Batch #4 patterns (tuple
patterns over optionals, `Set.of`) are not repeated here. All four waves
confirmed them, and they should land first or alongside Batch #4.

## Batch #4: core rules that remove wrong answers and the most common friction

All are compiler work in `cpl-check` / `cpl-interp` / `cpl-codegen`, with
no new syntax.

1. **Slice bounds never lie.** A negative or reversed slice bound is a check-time error when it is constant and a trap otherwise, exactly like `a[-2]`. Today `a[..-1]` silently returns `[]`. (wave 2 N1; `ucup5s05e`)
2. **Tropical reductions.** `min`, `max`, `argmin`, `argmax` and `sum` over `?T` skip `none` and return `?T`, matching `min=` / `max=`; `?T` gets the order "`none` is worst" only inside these reductions. (wave 1 gap 1; `ceoi18p1`, `noip09p3`, `noip14p3`, `ucup5s06g`, `p3_abc481_f`, `cco21p5`)
3. **Range refinement for domain steps.** Guards (`l == r`, `l < r`) and loop bounds (`m in l..r`) refine domain values, so `l + 1` is `I`, not `?I`, where it is proven. `+=` / `-=` on a domain value becomes a checked step. (wave 2 N2; `dpl`, `dpn`, `cf2262d`; 10 corpus files)
4. **Every finite domain, `Range` and `Bool` is a `Seq`.** `sorted(0..n, by:)` and `for b in Bool` then work with no new function. (wave 2 N4; `cf2261d2`, `cf2268c`, `dmopc21c2p5`)
5. **`a op= b` means `a = a op b` for every type with `op`.** This covers lists, `Str` and user types. (wave 3 W4; `noip06p2`)
6. **Graphs built from plain lists keep their vertex domain.** `topo`, `peel` and the rest must work on `Graph.directed(List<(V, V)>)`. (docs gate, reproduced above)
7. **Errors inside stdlib generics point at the user's call** and name the user's types. Also fix the E0402 hint, which suggests `mut Curve(..)` that E0401 then rejects. (wave 3 B3, B4; `ioi10p6`, `cco19p4`)

## Batch #5: core rules, second tier

1. **By-value mutable parameters.** `fn f(var b: Int)` is a local copy with no borrow rules. It also makes `cco19p4`'s E0402 go away. (wave 1 gap 6; `noi14p4`, `cco19p4`)
2. **Block bodies after `=>`.** A lambda or array builder may continue onto an indented block. (wave 3 B1; `cco19p4` twice)
3. **Infer `none` from later uses.** A `none` in a `var` tuple takes its type from later assignments in the same scope. (wave 3 B2; `noi15p5`)
4. **`first` / `last` / `last(k)` on every `Seq`, usable as places.** This covers lists, `[D]T` and `Deque`: `runs.last!.head = x`, `a.last!` on `[0..n]Int`, `hull.last(1)!`. (wave 1 gap 2, wave 2 N1; `agc079c`, `cf2262c`, `ucup5s05e`, `dpz`, `ucup5s05f`, `ioi16p6`)
5. **Payload enums as index types.** `enum Node: Vert(V), Edge(E), Src, Snk` works as a `Dsu`, `FlowGraph` and `[Node]T` key. (wave 1 gap 3; `cf2245e`, `noi09p4`, `dmopc20c7p5`)
6. **One print rule for every `List<T>`**, including `Mod<P>`. (wave 4 X2; `ucup5s05f`)
7. **Read and parse decimals into `Mod<P>`** without going through `Int`. (wave 4 X4; `noip14p6`, `p3_abc481_f`)
8. **Open ranges in `cpl run` / `cpl build`**, so `bsearch_first(lo.., p)` can gallop. (wave 4 X5; `ioi07p1`)

## Batch #6: cohesive userland `stdlib/` abstractions

Each item extends an existing module or type. None needs the compiler,
except an entry in `STD_NAMES` for any new top-level `pub fn`.

1. **`std.seq` value forms.** Every in-place operation gets a value form: `xs.deduped() -> List<T>`, `xs.partition(p: T -> Bool) -> (List<T>, List<T>)`, `xs.prefix(k) -> List<T>` (clamped), `Str.filled(n, c) -> Str`. Add `p.inverse -> [B]A` for a bijective `[A]B`, which traps if `p` is not a bijection. (wave 1 gap 5, wave 2 N1/N6; `cf2262c`, `yac7p5`, `cf2261h`, `ioi08p4`, `nccc10s3`, `ucup5s02e`)
2. **`Int` bits and digits.** `x.bit(i) -> Bool` and `x.digits(base = 10) -> List<Int>` (least significant first; negative bases follow Euclidean `%`). Delete `std.math.bit_len`. (wave 1 gap 4, wave 3 W3, wave 4 X1; `cf2261g`, `cf2268e`, `noip21p1`, `cf2261b`)
3. **`std.deque.Worklist<T>`.** `of(xs)`, `push(x)`, `next() -> ?(Int, T)`, `items -> List<T>`. `peel` and `bfs` are rebuilt on it. (wave 3 W2; `cf2245d2`; 25 files loop `while i < xs.len`)
4. **Point-update and tree-edge verbs.** `SegTree.update(i, f)` to match `LazySegTree`, and `Rooted.lower(u, v) -> V` for the child end of an edge. (wave 3 W5, W6; `dmopc20c7p4`, `btoi17p3`; 4 files)
5. **`FlowGraph.add(u, v, cap: ?Int)`**, where `none` means an uncuttable edge. (wave 2 N3; `noi09p4`, `dmopc20c7p5`)
6. **`std.geometry.hull(pts) -> List<Pt>`** (counter-clockwise, collinear points dropped), plus `hull_chains` for the lower and upper halves. (wave 2 N5; `ucup5s05e`)
7. **`Sieve.mu(x)` and `Sieve.phi(x)`**, from the same linear sieve. (wave 4 X3; `sdoi17r1p1`)
8. **`Map.get_or_insert(k, v) -> V`**, for interning and memo tables. (wave 4 X6; `btoi18p1`)

## Carry-over: confirmed gaps owned by an existing plan

These were confirmed again by this audit, but already have a home, so they
are not re-batched:

- **Friction queue, Syntax Batch #3:** reading `Bool` and `?T` (`dpo`, `yac7p5`, `noip06p2`, `ioi07p1`).
- **Friction queue, Syntax Batch #4:** tuple patterns over optionals (`cf2261g`, `cf2261d2`, `noi18p4`) and `Set.of` / `Subset.of` in `cpl run` (`cf2245c`, `dwite10c3p3`).
- **Friction queue, Syntax Batch #5:** `zip` / `pairs` / `combinations(k)` (`ucup5s05e`) and cyclic views and rotation (C5: `dmpg16s5`, `cf2255a`, `btoi17p3`, `ucup5s05b`, `cco20p4`, `ucup5s05e`).
- **Friction queue, long tail:** `LiChao` (P12: `dpz`, `ioi16p6`), quotient blocks (G12: `ccc18s4`, `sdoi17r1p1`), a sliding-window minimum (L15: `dmpg16s5`), permutation cycles (H6: `btoi18p1`), multipoint `Poly` (MG6: `ucup5s05f`), a typed node arena (A10/A13: `cf2261g`).
- **Missing from the friction queue, and should be added:**
  - A `Matrix` with `pow` (backlog batch 11, N1: `dpr`).
  - Domain / `Int` label crossings such as `I(g.label + 1)` and `.to_idx`, which come from sub-domains like `Kid = 2..=n` inside `V` (backlog batches 13 and 17: `cf2268c`, `aac5p2`, `dmopc21c2p5`, `ioi08p4`, `noi15p2`, `cco17p4`, `cco20p4`, `cco19p3`; 477 `to_idx` / `from_idx` lines in the corpus). This is the largest unowned friction source in the audit, and it needs a design pass of its own (sub-domain coercion), not a quick batch item.

## Rejected one-off helpers

- `sign` (`ucup5s05b`), `mean`, `Dsu.unite_under`, `Rooted.at_centroid`: already rejected in the friction queue.
- A top-level `swap(a, i, j)`, `clone()` and a `Queue<T>` alias: they duplicate `a.swap(i, j)`, `.copy()` and `Deque`.
- A second name `find` next to `first(..)`: one concept, one name.
- A helper that widens a candidate window to five points (`ucup5s05e`), and an in-place multiply by `(d x + 1)` (`ucup5s01a`): both problem-specific.
- `has7` / digit-predicate helpers: `7 in x.digits` covers them once Batch #6 item 2 lands.
- A difference-array type (`cf2262a1`, `cf2262a2`): two lines with `+=` and an in-place prefix sum are already the obvious form.
- Square-root decomposition, union-length sweeps and leaf peeling with side effects (`dmpg18s5`, `ucup5s05b`, `btoi18p1`): these are algorithms, not missing abstractions.
