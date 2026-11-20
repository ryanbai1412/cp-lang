# Cold-writer benchmark #3

Base: `main` at `b2c22c2` (after Syntax Batches #6 and #7). Ten standard
contest problems, each written once from `docs/one-page-cheat-sheet.md` alone,
without reading `stdlib/`, `crates/`, or `research/cold-writer-bench-2.md`
first. All ten first drafts were saved before any tool ran, then checked with
`cpl check`. The drafts that checked (and the fixed versions of the others)
went through `cpl build`, `g++ -std=c++17 -O2 -fsanitize=address,undefined
-fno-sanitize-recover=all`, and `cpl run`. Both outputs were compared with a
Python reference solution on 40 random small inputs per problem, plus one
max-N input for five of them.

Unlike Bench #2, which ran only the samples, every result here is stress
tested against an independent reference: 10 problems × 40 cases × (`cpl run` +
ASan binary).

## Result

- **First-try `cpl check` pass rate: 7/10**, the same as Bench #2. The three
  failures differ from Bench #2's: grid BFS and `find_all` fail for new reasons,
  and the third is Dijkstra.
- **First-try end-to-end pass rate: 7/10.** Every draft that checked also
  built, compiled cleanly under ASan/UBSan, and matched the reference on all 40
  random cases in both `cpl run` and the binary. There were no wrong answers
  and no differences between run and build.
- **After one edit: 9/10.** Grid BFS and string matching each needed one
  token-level change. Dijkstra needed two rewrites, and the middle attempt
  produced an error located at line 1, column 1.
- Tuple vertices now work: the grid BFS over `(Row, Col)` with `(r, c).adj4`
  and `[Row, Col]?Int` relaxation, which needed a full rewrite in Bench #2,
  failed only on an optional unwrap.

The ten problems: grid BFS on a `#`/`.` maze, string matching (all
occurrences), 0/1 knapsack, tree rerooting (eccentricity of every vertex),
max flow, binomials mod 1e9+7, greedy interval scheduling by end time, a
constructive permutation (CSES "Permutations"), `Mod<P>` pair-product sum, and
Dijkstra.

## Passed first try (7)

These came out the way a contestant would write them. Lines worth keeping as
the reference idiom:

```cp
// knapsack: runtime-sized structural array, reverse range, max= as a statement
var dp: [0..=cap]Int = 0
for c in (w..=cap).rev:
    dp[c] max= dp[c - w] + v

// rerooting: excl_fold gives each child the fold over its siblings, so the
// "up" value needs no prefix/suffix arrays
let best = Monoid<Int>(id: 0, op: (x, y) => max(x, y))
fn dfs2(v: V):
    for (c, rest) in t.excl_fold(v, best, c => down[c] + 1):
        up[c] = 1 + max(up[v], rest)
        dfs2(c)
print([max(down[v], up[v]) for v in V])

// greedy: read a whole record array, sort by a field
type Iv = (l: Int, r: Int)
let ivs: [0..n]Iv = read()
for x in sorted(ivs, by: x => x.r):
    if free_at == none or x.l >= free_at!: ...

// constructive: stepped ranges in comprehensions, += on lists
var p = [i for i in (2..=n).step(2)]
p += [i for i in (1..=n).step(2)]

// Mod<P>: a 0..=1e18 range type converts with Z(x); / 2 is a modular inverse
type Z = Mod<1_000_000_007>
s += Z(x)
print((s * s - sq) / 2)
```

Max flow (`FlowGraph<V>()`, `.add`, `.max_flow(s, t)`) and binomials
(`let c = Comb<P>()` with no size argument, `c.choose(n, k)` is 0 when `k > n`)
passed unchanged as well.

One weak spot in rerooting: the cheat sheet gives no way to list a vertex's
children, so the down pass loops over `t.excl_fold(v, best, c => 0)` and
throws away the fold. It works, but nobody would guess it. `t.children(v)`,
`t.parent(v)`, `t.preorder` and `t.postorder` all exist in `stdlib/tree.cp`
(Bench #2 used them), and the cheat sheet shows none of them.

## The three first-draft fights

### 1. Grid BFS: `Deque.pop_front()` returns `?T`, and the cheat sheet hides it (1 edit)

```cp
while not q.is_empty:
    let (r, c) = q.pop_front()
```

```
error[E0302]: a `?(Row, Col)` can't be taken apart into 2 names
  --> p01_grid_bfs.cp:13:9
```

Three more errors followed, all caused by the first (`r`/`c` unknown, then
`g[nr, nc]` "indexed by `Row`, but the index is a `Int`"). Adding `!` fixed all
four. The cheat sheet's §8 example is `print(q.pop_front()) // 1`, which reads
as if it returns `T`, while §8 says outright that `List.pop` returns `?T`. The
rest of the grid draft was right the first time: `[Row, Col]Char = read()`,
`(r, c).adj4` from `std.grid`, and `if dist[nr, nc] min= dist[r, c] + 1:
q.push(...)`.

What would make it pass on try #1:

- Cheat sheet: write `.pop_front -> ?T` in §8 and change the example to
  `while let x = q.pop_front(): ...`.
- Diagnostic: when the right-hand side of a destructuring is `?T` and `T`
  would match, say "`pop_front` returns `?(Row, Col)`: add `!`, or use
  `while let (r, c) = q.pop_front():`". Errors caused by the first one (unknown
  `r`, `Int` index) should not be printed.
- The 1000×1000 grid takes 8.9 s under `cpl run` and 0.13 s compiled. That is
  far over the 200 ms interpreter budget; the cause is untriaged.

### 2. String matching: `find_all` is a free function, not a `Str` method (1 edit)

```cp
let occ = s.find_all(p)
```

```
error[E0203]: a `Str` has no method `find_all` that `cpl run` knows
```

Since Bench #2 `find_all` has been in the prelude, but only as the free
generator `find_all(text, pat)`. The cheat sheet lists `.find -> ?Int` as a
method in §7 and never mentions `find_all`, so the method form is the natural
guess. The fix was `let occ = [i for i in find_all(s, p)]`.

What would make it pass on try #1: since `s.find(p)` is a method, make
`s.find_all(p)` a method too (one `pub fn find_all(self: Str, p: Str) ->
yields Int` beside `find` in `stdlib/strings.cp`), and list it in §7. Failing
that, E0203 should say "did you mean the free function `find_all(s, p)`?"
whenever a prelude free function has the method's name and takes the receiver
type first.

### 3. Dijkstra: arc `.e` exists only for `[E]` record edges (2 edits, one with a 1:1 error)

The draft followed §10 to the letter (`w: a => weights[a.e]`), keeping edges
and weights in two lists:

```cp
var edges: List<(V, V)> = []
var wt: List<Int> = []
...
let g = Graph.directed(edges)
let d = dijkstra(g, from: [(V.first, 0)], w: a => wt[a.e])
```

```
error[E0203]: a `V` has no member `e` that `cpl run` knows
  --> p10_dijkstra.cp:12:56
```

`Graph.directed` takes a `List<(V, V)>` without complaint but returns plain
adjacency lists, whose arcs are bare `V` with no edge id. The second attempt
used weighted triples and `a.1`:

```cp
var edges: List<(V, V, Int)> = []
let d = dijkstra(g, from: [(V.first, 0)], w: a => a.1)
```

```
error[E0301]: this is indexed by `V`, but the index is a `(V, Int)`
  --> p10_dijkstra.cp:1:1
error[E0302]: expected `V`, found `(V, Int)`
  --> p10_dijkstra.cp:1:1
```

Both errors come from inside the generic `dijkstra` and point at line 1,
column 1 of the user's file. That is the same bad-span bug Bench #2 reported
for `bfs`, so it is still open for `dijkstra`. The version that works reads an
edge-domain record array, which only the docstring at the top of
`stdlib/graph.cp` shows:

```cp
domain E = 0..m
let edges: [E](u: V, v: V, w: 1..=1_000_000_000) = read()
let g = Graph.directed(edges)
let d = dijkstra(g, from: [(V.first, 0)], w: a => edges[a.e].w)
```

What would make it pass on try #1:

- Cheat sheet §10: show the edge declaration that `a.e` needs
  (`let edges: [E](u: V, v: V, w: Int) = read()`), not just the call.
- Library: a `List<(V, V, W)>` graph should give arcs that `dijkstra` accepts
  directly (a `(to, w)` record, `w: a => a.w`), or `dijkstra` should take the
  triples form, so the natural "push `(u, v, w)`" spelling works.
- Diagnostic: point generic-instantiation errors at the user's call
  (`p10_dijkstra.cp:11`), not 1:1. Bench #2 asked for the same thing (fix #4).
  For `a.e` on a bare vertex, the hint should say "arcs carry `.e` only when
  the graph is built from an `[E]` edge array".

## Other observations from the stress runs

- **ASan stack overflow on deep recursion is silent.** The rerooting ASan
  binary on a 100,000-vertex path exits with code 1, no output and nothing on
  stderr under the default 8 MB stack. It works with `ulimit -s unlimited`, and
  the `-O2` binary and `cpl run` work as is. The recursion depth is the
  program's choice, but a silent exit 1 is a bad local experience. The emitted
  `main` could raise the stack limit (or run on a large thread stack), as many
  CP templates do.
- **Max-N timings**, `cpl run` vs `g++ -O2` binary: grid BFS 1000×1000 8.9 s
  vs 0.13 s; rerooting n = 1e5 path 1.3 s vs 0.05 s; 1e5 binomials up to 1e6
  1.7 s vs 0.05 s; Dijkstra n = 1e5, m = 2e5 1.6 s vs 0.11 s; permutation
  n = 1e6 0.20 s vs 0.07 s. All outputs matched across `cpl run`, `-O2` and
  ASan, apart from the rerooting stack case above.
- Emitted C++ ranges from 2.9 KB (knapsack) to 17.6 KB (rerooting), all well
  under 64 KB.

## Proposed fixes, in order

1. **Generic-instantiation errors at the user's call site** (Dijkstra; open
   since Bench #2). A 1:1 location on a generic graph call is the one error a
   cold writer cannot act on.
2. **Weighted edge lists that work with `dijkstra` out of the box**, plus the
   `[E]` edge declaration in cheat sheet §10.
3. **`s.find_all(p)` as a `Str` method** beside `s.find(p)`, and listed in §7.
4. **Cheat sheet accuracy for optional returns**: `Deque.pop_front`/`.pop ->
   ?T` with a `while let` example, and a destructuring diagnostic that suggests
   `!`/`while let` and drops the errors it causes.
5. **Rooted traversal on the cheat sheet**: `t.children(v)`, `t.parent(v)`,
   `t.preorder`, `t.postorder` exist but are not listed, so rerooting drafts
   misuse `excl_fold` just to walk children.
6. **Interpreter max-N latency** on grid BFS (8.9 s) and a raised stack limit
   in emitted `main`.

## What was not covered

There was no interactive, segment tree, or interval DP problem this round. The
categories followed the ticket (grid BFS, `find_all`, DP, rerooting, max flow,
`Comb`, greedy, constructive, `Mod<P>`), with Dijkstra in the tenth slot. Random
tests used small inputs; max-N inputs were run for five problems only (grid
BFS, rerooting, binomials, permutation, Dijkstra). The `cpl stress`,
`@brute`/`@gen` workflow and the trace debugger were not exercised.
