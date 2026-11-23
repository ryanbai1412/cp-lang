# Friction Hunter, wave 1

Thirty solved ports, read line by line, looking for every awkward line,
not only the ones with a `// WISH` comment. Each pattern was then checked
against `cpl` on `main` at `eca47e3`:

- **[STALE PORT]**: HEAD already has a clean form (checked with `cpl run`); the port just needs rewriting.
- **[REAL GAP]**: still awkward on HEAD, with the error quoted.
- **[REJECTED]**: awkward-looking, but the current form is already the obvious one.

## The numbers

The 30 ports have 702 non-blank lines. They hold 50 frictions
covering 112 awkward lines: 18 real gaps (45 lines),
28 stale-port patterns and 4 rejected.

- Real gaps per problem: 18 / 30 = **0.60**.
- Awkward lines per problem: 112 / 30 = **3.73**.
- Frictions of any kind per problem: 50 / 30 = 1.67.
- Ports with no friction at all: 7 (`cf2261c`, `dpk`, `dpu`, `dpx`, `ioi04p4`, `noip04p3`, `ccc21s5`).

Every one of the 30 checks clean with `cpl check` on HEAD.

## The 30 ports

- `ecnerwala/agc079a` (combinatorics, stack): 2 frictions.
- `ecnerwala/agc079c` (deque simulation): 4 frictions.
- `ecnerwala/cf2245c` (constructive, bits): 2 frictions.
- `ecnerwala/cf2245e` (trees, DSU): 2 frictions.
- `ecnerwala/cf2255d` (greedy, bits): 3 frictions.
- `ecnerwala/cf2261c` (games): 0 frictions.
- `ecnerwala/cf2261e` (strings, binary search): 1 frictions.
- `ecnerwala/cf2261g` (binary trie, bits): 4 frictions.
- `ecnerwala/cf2261h` (Hamming code, strings): 2 frictions.
- `ecnerwala/cf2262c` (math, counting): 3 frictions.
- `wave2/dpk` (game DP): 0 frictions.
- `wave2/dpm` (window DP): 2 frictions.
- `wave2/dpt` (permutation DP): 2 frictions.
- `wave2/dpu` (subset DP): 0 frictions.
- `wave2/dpv` (rerooting DP): 1 frictions.
- `wave2/dpx` (knapsack): 0 frictions.
- `wave2/ioi04p4` (memo DP): 0 frictions.
- `wave2/noip04p3` (LIS DP): 0 frictions.
- `wave2/noip14p5` (graphs, BFS): 1 frictions.
- `wave2/ccc18s4` (number theory, memo): 1 frictions.
- `ccc21s5` (segment tree, gcd): 0 frictions.
- `ccc23s5` (recursion, number theory): 3 frictions.
- `cco17p1` (constructive graphs): 2 frictions.
- `ceoi18p1` (knapsack, sweep): 2 frictions.
- `coci13c1p3` (2-D prefix sums): 1 frictions.
- `dmpg16s5` (sliding window): 2 frictions.
- `ioi10p3` (binary search, 2-D prefix): 1 frictions.
- `noi14p4` (KMP strings): 2 frictions.
- `noip09p3` (graphs, SPFA): 3 frictions.
- `yac7p5` (interactive): 4 frictions.

## Real gaps, clustered into general fixes

Each cluster is a general mechanism or a cohesive library API, not a
helper for one problem. "New" marks gaps that are not in
`research/archive/frictions/friction-queue.md` yet.

### 1. Tropical reductions over optionals (new)

The README says `min` and `max` skip `none`, and `min=` / `max=` do. But
a reduction over a `[D]?Int` or a generator of `?Int` fails: "a `?Int` has
no order to compare by". `ceoi18p1` and `noip09p3` fall back to a
`var profit = 0` loop.

```cp
// today
var profit = 0
for b in best: profit max= b
// wanted: `max` and `min` skip `none`, as `max=` does
print(max(best) ?? 0)
```

Fix: give the `min` / `max` / `argmin` / `argmax` reductions the same
`none`-skipping rule as `min=` and `max=`, returning `?T`.

### 2. Ends of a sequence as places and on domain arrays (new)

`a.last!` works on a `List` but not on a `[0..n]Int` ("a `[Int]Int` has no
member `last`"), and `xs.last!.f = v` cannot be assigned ("has no field
`last` to assign to"). Ports write `runs[runs.len - 1].head` and
`a[n - 1]` instead (`agc079c`, `cf2262c`).

```cp
// today
runs[runs.len - 1].head = succ[top.head]
let top = a[n - 1]
// wanted
runs.last!.head = succ[top.head]
let top = a.last!
```

Fix: `first` / `last` on every `Seq` (lists and `[D]T`), and allow them
as places the way `a[i]` is.

### 3. Payload enums as index types (new)

The README says enums implement `Index`, and the cheat sheet pairs
`FlowGraph<Node>` with a payload enum. But `enum Node: Vert(V), Edge(E)`
used as a `Dsu<Node>` key fails in `cpl run`: "`cpl run` doesn't know the
name `V` here". `cf2245e` packs vertices and edges into
`domain X = 0..2 * n - 1` and decodes with `x.ord >= n`.

```cp
// today
domain X = 0..2 * n - 1
pieces.unite(X(n + e), X(u - 1))
let extra = count(x.ord >= n for x in g) + 1 - count(x.ord < n for x in g)
// wanted
enum Node:
    Vert(V)
    Edge(E)
pieces.unite(Node.Edge(e), Node.Vert(u))
let extra = count(x is Edge for x in g) + 1 - count(x is Vert for x in g)
```

### 4. Bit tests on `Int` (new)

`x >> z & 1 == 0` appears three times in `cf2261g` alone, and the
precedence of `>>`, `&` and `==` is exactly where contest bugs come from.
`x.bit(z)` fails: "a `Int` has no method `bit`". Fix: `x.bit(i) -> Bool`
next to `popcount` and `ctz`. A plain `Int` method, no new syntax.

### 5. Value forms that mirror the mutating ones (new)

The library has `sort` / `sorted` and `reverse` / `.rev`, but `dedup` only
mutates, `partition` does not exist, and a `Str` cannot be made by
repetition (`Str.filled` and `"0" * n` both fail). Ports write
`var s = sorted(..); dedup(mut s)` (`cf2262c`), a two-list loop
(`yac7p5`), and `Str('0' for _ in 0..size)` (`cf2261h`).

```cp
// wanted
let seconds = sorted([a[n - 2], top - a[n - 2] / (half - 1)]).deduped()
let (lower, upper) = eggs[..-1].partition(e => breaks(e, mid - 1))
var s = Str.filled(size, '0')
```

Fix: one rule for the sequence API, that every in-place operation has a
value form, plus `partition` and `Str.filled`. No problem-specific names.

### 6. Mutable parameters (new)

`noi14p4` writes `var b = b` as the first line of `grow` because
`fn grow(var b: Int, i: Int)` fails: "expected a parameter name, found
`var`". Fix: accept `var` on a by-value parameter. It is a local copy,
so it adds no borrow rules.

### 7. Already in the friction queue, confirmed again

- Tuple patterns over optionals, `let (p, q) = (x, y) else:` (`cf2261g`; Syntax Batch #4).
- `Set.of(xs)` in `cpl run` (`cf2245c`; Syntax Batch #4).
- `ask` / `read()` of a `Bool` (`yac7p5`; Syntax Batch #3).
- A typed node arena with recycled ids (`cf2261g`; A10, A13).
- The quotient-block loop `w / (w / k)` (`ccc18s4`; G12).
- A sliding-window minimum and a cyclic view (`dmpg16s5`; L15, C5).

## Stale ports: rewrites that need no compiler work

- `ecnerwala/agc079a` lines 13: `blocks.len > 0 and blocks.last!.r >= p[i] - 1` is `while let b = blocks.last and b.r >= p[i] - 1` (runs on HEAD).
- `ecnerwala/agc079a` lines 16-17: a hand-rolled factorial loop; `Comb<P>().fact(n - 1)` exists.
- `ecnerwala/agc079c` lines 13-14, 16: `List.filled(n, -1)` and `to: -1` sentinels where `?Int` says it.
- `ecnerwala/agc079c` lines 77: `if front[b] == none: front[b] = x` is `front[b] ??= x` (runs).
- `ecnerwala/cf2245c` lines 20-21: push loop after a comprehension; `xs + tail.rev` runs.
- `ecnerwala/cf2245e` lines 5, 9-10, 14-15: `u - 1` on every use of a 1-based vertex; `domain V = 1..=n` and `[V]Bool` read and index directly (runs).
- `ecnerwala/cf2255d` lines 15, 38: `least[k] = min(least[k], x)` is `least[k] min= x`.
- `ecnerwala/cf2255d` lines 12: `1 << B` as "no value yet" where `?Int` says it.
- `ecnerwala/cf2255d` lines 40: a dead `false` after `while true:`; HEAD accepts the loop as the last statement.
- `ecnerwala/cf2261e` lines 4: `use std.seq.{bsearch_first}`: it is in the prelude.
- `ecnerwala/cf2261g` lines 35: `kept.len > 0 and kept.last!.mask == v` is `if let k = kept.last and k.mask == v` (runs).
- `ecnerwala/cf2261h` lines 15: `if c == '1': 1 else: 0` is `Int(c == '1')`.
- `ecnerwala/cf2262c` lines 19-21: push loop, then `merged = sorted(merged)`; `sorted(xs + ys)` runs.
- `wave2/dpm` lines 12-16: a hand-slid window sum; `prefix(ways).sum(c - cap..=c)` runs.
- `wave2/dpt` lines 23-24: `for w in ways: total += w` is `sum(ways)`, which works on `Mod`.
- `wave2/dpv` lines 19-27: the earlier/later sibling scans are `tree.excl_fold(v, mul, c => down[c] + 1)`.
- `wave2/noip14p5` lines 3: `use std.graph.{bfs}`: it is in the prelude.
- `ccc23s5` lines 1: a stale `// cp: 0.1 ...+dirty` prototype stamp.
- `ccc23s5` lines 18: a dead `true` after `while true:`.
- `ccc23s5` lines 27: `(a + b - 1) / b` by hand; `ceil_div` is in `std.math`.
- `cco17p1` lines 1: stale prototype stamp.
- `cco17p1` lines 8: `c * (c - 1) / 2` is `choose(c, 2)` from `std.math`.
- `coci13c1p3` lines 1: stale prototype stamp.
- `ioi10p3` lines 3: `use std.seq.{bsearch_first}`: it is in the prelude.
- `noip09p3` lines 10-17: two adjacency lists built by hand; `Graph.directed(one_way + two_way)` and `.transpose()` say it.
- `noip09p3` lines 28: `best[v] != pick(best[v] ?? via, via)` as "did it improve"; `if best[v] min= via:` already returns whether it changed (runs).
- `yac7p5` lines 1: stale prototype stamp.
- `yac7p5` lines 27, 29: `eggs.len - 1` and `eggs[eggs.len - 1]` on a `List`; `eggs.last!` runs.

One more finding: `if best[v] min= via:` already returns whether the
value changed (checked on HEAD). That is the relaxation step of BFS, SPFA
and Dijkstra, and it should be on the cheat sheet.

## Rejected

- `wave2/dpm` lines 8-9: `var ways: [D]Z = 0` then `ways[0] = 1`; the two-line form is the obvious one.
- `wave2/dpt` lines 9-10: same one-hot start as dpm.
- `ceoi18p1` lines 15-16: same one-hot start as dpm.
- `noi14p4` lines 11-12: same one-hot start as dpm.

## Detector rules for `scripts/friction_lint.py`

The script does not exist on `main` yet. These rules would have caught
every stale-port finding above. Each is a regex over a port's code lines,
with the rewrite it suggests:

1. `(\w+)\[\1\.len - 1\]`: use `xs.last!`.
2. `(\w+)\.len > 0 and \1\.last!`: use `while let x = xs.last and ...` or `if let`.
3. `^(\s*)(\S+) = (min|max)\(\2, `: use `x min= v` / `x max= v`.
4. `^// cp: \d`: delete the prototype stamp.
5. `use std\.\w+\.\{[^}]*\b(bsearch_first|bsearch_last|bfs|topo|peel|resized)\b`: the name is in the prelude.
6. `List\.filled\([^,]+, -1\)`, or a record field defaulting to `-1`: use `?Int` and `none`.
7. `if (\S+) == none: \1 = `: use `x ??= v`.
8. `\((\w+) \+ (\w+) - 1\) / \2`: use `ceil_div`.
9. `if (.+): 1 else: 0`: use `Int(cond)`.
10. `while true:` whose function ends with a lone `true` or `false` line: delete the dead tail.
11. `var (\w+) = \[.*\]` followed by `for .*: \1\.push\(`: use `+` to join the lists.
12. `for (\w+) in (\w+): (\w+) \+= \1` on its own: use `sum(xs)`.
13. `\b(\w+) - 1\]` on a value read from a `1..=n` range: declare `domain V = 1..=n` and index by it.
14. `var (\w+) = \1$` inside a function: report the mutable-parameter gap.
15. `>> \w+ & 1`: report the bit-test gap until `x.bit(i)` lands, then suggest it.
